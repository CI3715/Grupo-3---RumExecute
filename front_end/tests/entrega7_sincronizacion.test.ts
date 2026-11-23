import test, { describe, beforeEach } from "node:test";
import assert from "node:assert";

// Mock de entorno de navegador y almacenamiento para pruebas en Node.js
const store = new Map<string, string>();
const mockStorage = {
  getItem: (key: string) => store.get(key) ?? null,
  setItem: (key: string, val: string) => {
    store.set(key, val);
  },
  removeItem: (key: string) => {
    store.delete(key);
  },
  clear: () => {
    store.clear();
  },
};

(globalThis as any).window = globalThis;
(globalThis as any).localStorage = mockStorage;
if (typeof (globalThis as any).window.dispatchEvent !== "function") {
  (globalThis as any).window.dispatchEvent = () => true;
}
if (typeof (globalThis as any).CustomEvent !== "function") {
  (globalThis as any).CustomEvent = class CustomEvent {
    type: string;
    constructor(type: string) {
      this.type = type;
    }
  };
}

import {
  elegirCuentaGrupo,
  obtenerCuentaGrupo,
  listarPartesPendientes,
  sincronizarCambios,
  reiniciarCursorSincronizacion,
  encolarGastoOffline,
  listarColaOffline,
  despacharColaOffline,
  obtenerCursorLocal,
  obtenerRechazosColaLocal,
} from "../src/services/syncService";
import { calcularTotalesCuenta } from "../src/utils/formato";

function inicializarLibroBetoConEfectivoUSD(saldoInicial = 200) {
  localStorage.setItem(
    "cuentas_claras_session_user",
    JSON.stringify({ id: "user-beto", nombre_usuario: "beto", nombre_visible: "Beto Gómez" })
  );
  localStorage.setItem("cuentas_claras_session_token", "token-beto-valido");
  localStorage.setItem(
    "cuentas_claras_libro_local_v1",
    JSON.stringify({
      cuentas: [
        {
          id: "cta-efectivo-usd",
          nombre: "Efectivo USD",
          tipo: "efectivo",
          moneda: "USD",
          saldo_formateado: "$ 200,00",
        },
      ],
      saldosIniciales: { "cta-efectivo-usd": saldoInicial },
      movimientosPorCuenta: { "cta-efectivo-usd": [] },
      categoriasGasto: [],
      categoriasIngreso: [],
    })
  );
}

function leerMovimientosYCalcularSaldo(cuentaId = "cta-efectivo-usd") {
  const libro = JSON.parse(localStorage.getItem("cuentas_claras_libro_local_v1") || "{}");
  const movs = libro.movimientosPorCuenta?.[cuentaId] || [];
  const saldoIni = libro.saldosIniciales?.[cuentaId] ?? 0;
  const { saldoActual } = calcularTotalesCuenta(saldoIni, movs);
  return { movs, saldoActual };
}

describe("Entrega 7: Sincronización de Gastos de Grupo, Idempotencia y Cola sin Conexión", () => {
  beforeEach(() => {
    store.clear();
  });

  test("1. Sincronizar baja la parte del gasto a la cuenta elegida y marca el movimiento como gasto de grupo (RF-COM-10)", async () => {
    inicializarLibroBetoConEfectivoUSD(200);
    await elegirCuentaGrupo("grp-merida", "cta-efectivo-usd", "USD");

    const asignada = await obtenerCuentaGrupo("grp-merida");
    assert.strictEqual(asignada?.cuenta_id_personal, "cta-efectivo-usd");

    (globalThis as any).fetch = async (url: string) => {
      assert.ok(url.includes("/cambios?desde=0"));
      return {
        ok: true,
        status: 200,
        json: async () => ({
          eventos: [
            {
              id: "ev-hotel-beto",
              posicion: 1,
              usuario_id: "user-beto",
              grupo_id: "grp-merida",
              nombre_grupo: "Viaje a Mérida",
              gasto_id: "gst-hotel",
              concepto: "Hotel",
              fecha: "2026-11-22T20:00:00Z",
              moneda: "USD",
              monto_parte: "33.33",
              incluye_redondeo: false,
            },
          ],
          nuevo_cursor: 1,
          quedan_mas: false,
        }),
      };
    };

    const res = await sincronizarCambios();
    assert.strictEqual(res.eventos_aplicados, 1);
    assert.strictEqual(res.nuevo_cursor, 1);
    assert.strictEqual(obtenerCursorLocal(), 1);

    const { movs, saldoActual } = leerMovimientosYCalcularSaldo();
    assert.strictEqual(movs.length, 1);
    assert.strictEqual(movs[0].nota, "Viaje a Mérida · Hotel");
    assert.strictEqual(movs[0].es_gasto_grupo, true);
    assert.strictEqual(Math.round(saldoActual * 100) / 100, 166.67);
  });

  test("2. Idempotencia estricta: aplicar el mismo lote dos veces o reiniciar el cursor a cero no duplica movimientos ni altera saldos (INV-13, INV-15)", async () => {
    inicializarLibroBetoConEfectivoUSD(200);
    await elegirCuentaGrupo("grp-merida", "cta-efectivo-usd", "USD");

    const eventoHotel = {
      id: "ev-hotel-beto",
      posicion: 1,
      usuario_id: "user-beto",
      grupo_id: "grp-merida",
      nombre_grupo: "Viaje a Mérida",
      gasto_id: "gst-hotel",
      concepto: "Hotel",
      fecha: "2026-11-22T20:00:00Z",
      moneda: "USD",
      monto_parte: "33.33",
      incluye_redondeo: false,
    };

    (globalThis as any).fetch = async () => ({
      ok: true,
      status: 200,
      json: async () => ({
        eventos: [eventoHotel],
        nuevo_cursor: 1,
        quedan_mas: false,
      }),
    });

    // Sincronización 1
    await sincronizarCambios();
    // Sincronizaciones 2 y 3
    await sincronizarCambios();
    await sincronizarCambios();

    // Reiniciar cursor a 0 y sincronizar de nuevo (Sección 2.6 y 2.8 Paso 3)
    await reiniciarCursorSincronizacion();
    assert.strictEqual(obtenerCursorLocal(), 0);
    const resReinicio = await sincronizarCambios();

    assert.strictEqual(resReinicio.eventos_aplicados, 0);
    assert.strictEqual(obtenerCursorLocal(), 1);

    const { movs, saldoActual } = leerMovimientosYCalcularSaldo();
    assert.strictEqual(movs.length, 1);
    assert.strictEqual(Math.round(saldoActual * 100) / 100, 166.67);
  });

  test("3. Partes de un grupo sin cuenta elegida quedan en pendientes de aplicar y se aplican al elegirla (Sección 2.2, INV-18)", async () => {
    inicializarLibroBetoConEfectivoUSD(200);
    // No se elige cuenta antes de sincronizar

    (globalThis as any).fetch = async () => ({
      ok: true,
      status: 200,
      json: async () => ({
        eventos: [
          {
            id: "ev-hotel-carla",
            posicion: 2,
            usuario_id: "user-beto",
            grupo_id: "grp-merida",
            nombre_grupo: "Viaje a Mérida",
            gasto_id: "gst-hotel",
            concepto: "Hotel",
            fecha: "2026-11-22T20:00:00Z",
            moneda: "USD",
            monto_parte: "33.33",
            incluye_redondeo: false,
          },
        ],
        nuevo_cursor: 2,
        quedan_mas: false,
      }),
    });

    const resSync = await sincronizarCambios();
    assert.strictEqual(resSync.eventos_aplicados, 0);
    assert.strictEqual(resSync.partes_pendientes, 1);

    const pendientesAntes = await listarPartesPendientes();
    assert.strictEqual(pendientesAntes.length, 1);
    assert.strictEqual(pendientesAntes[0].concepto, "Hotel");

    // Ahora el usuario elige la cuenta personal en USD -> se aplica de inmediato
    const resEleccion = await elegirCuentaGrupo("grp-merida", "cta-efectivo-usd", "USD");
    assert.strictEqual(resEleccion.partes_aplicadas, 1);

    const pendientesDespues = await listarPartesPendientes();
    assert.strictEqual(pendientesDespues.length, 0);

    const { movs, saldoActual } = leerMovimientosYCalcularSaldo();
    assert.strictEqual(movs.length, 1);
    assert.strictEqual(Math.round(saldoActual * 100) / 100, 166.67);
  });

  test("4. Cola de gastos sin conexión envía en orden con Idempotency-Key y una operación rechazada no bloquea las demás (RNF-17, ESC-09, INV-16)", async () => {
    inicializarLibroBetoConEfectivoUSD(166.67);

    // Encolar 2 operaciones sin conexión: la primera inválida (400) y la segunda válida (Cena 60.00)
    const opInvalida = await encolarGastoOffline({
      grupo_id: "grp-merida",
      concepto: "Gasto con porcentajes rotos",
      monto: "50.00",
      pagador_id: "user-beto",
      regla_division: "PORCENTAJES",
      participantes: ["user-ana", "user-beto"],
      valores: { "user-ana": "40", "user-beto": "40" },
    });

    const opCena = await encolarGastoOffline({
      grupo_id: "grp-merida",
      concepto: "Cena",
      monto: "60.00",
      pagador_id: "user-beto",
      regla_division: "PORCENTAJES",
      participantes: ["user-ana", "user-beto", "user-carla"],
      valores: { "user-ana": "50", "user-beto": "25", "user-carla": "25" },
    });

    assert.ok(opInvalida.idempotency_key.length > 5);
    assert.ok(opCena.idempotency_key.length > 5);
    assert.notStrictEqual(opInvalida.idempotency_key, opCena.idempotency_key);
    assert.strictEqual((await listarColaOffline()).length, 2);

    const peticionesRecibidas: Array<{ headers: Record<string, string>; body: any }> = [];

    (globalThis as any).fetch = async (_url: string, init: any) => {
      const bodyParsed = JSON.parse(init.body);
      peticionesRecibidas.push({ headers: init.headers, body: bodyParsed });

      // INV-16: Verificar que ninguna petición contiene cuentas, saldos ni movimientos del libro personal
      assert.strictEqual(bodyParsed.cuentas, undefined);
      assert.strictEqual(bodyParsed.saldosIniciales, undefined);
      assert.strictEqual(bodyParsed.movimientosPorCuenta, undefined);

      if (bodyParsed.concepto === "Gasto con porcentajes rotos") {
        return {
          ok: false,
          status: 400,
          json: async () => ({ detail: { mensaje: "Los porcentajes suman 80%, faltan 20%." } }),
        };
      }
      return {
        ok: true,
        status: 201,
        json: async () => ({ id: "gst-cena" }),
      };
    };

    const despacho = await despacharColaOffline();
    assert.strictEqual(despacho.exitosas, 1);
    assert.strictEqual(despacho.rechazos.length, 1);
    assert.ok(despacho.rechazos[0].motivo.includes("faltan 20%"));
    assert.strictEqual((await listarColaOffline()).length, 0);
    assert.strictEqual(obtenerRechazosColaLocal().length, 1);

    // Ambas enviaron su encabezado Idempotency-Key
    assert.strictEqual(peticionesRecibidas.length, 2);
    assert.strictEqual(peticionesRecibidas[0].headers["Idempotency-Key"], opInvalida.idempotency_key);
    assert.strictEqual(peticionesRecibidas[1].headers["Idempotency-Key"], opCena.idempotency_key);
  });
});
