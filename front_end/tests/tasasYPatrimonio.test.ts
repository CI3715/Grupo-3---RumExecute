import test, { describe } from "node:test";
import assert from "node:assert";

// Mock de entorno de almacenamiento para pruebas en Node.js
if (typeof globalThis.window === "undefined") {
  const store = new Map<string, string>();
  const mockStorage = {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, val: string) => { store.set(key, val); },
    removeItem: (key: string) => { store.delete(key); },
    clear: () => { store.clear(); },
  };
  (globalThis as any).window = globalThis;
  (globalThis as any).localStorage = mockStorage;
}

import {
  MONEDAS_FABRICA,
  resolverTasaVigente,
  registrarTasa,
  listarHistorialTasas,
  consultarPatrimonio,
  cambiarMonedaBase,
  obtenerMonedaBase,
} from "../src/services/tasasService";
import { formatearMonto } from "../src/utils/formato";
import { TasaCambioItem } from "../src/types/tasas";

describe("Entrega 4: Manejo de Monedas, Tasas de Cambio y Patrimonio Consolidado", () => {

  test("1. Catálogo inicial contiene exactamente las 4 monedas de fábrica", () => {
    assert.strictEqual(MONEDAS_FABRICA.length, 4);
    const codigos = MONEDAS_FABRICA.map(m => m.codigo);
    assert.deepStrictEqual(codigos.sort(), ["EUR", "USD", "USDT", "VES"]);
    assert.ok(MONEDAS_FABRICA.every(m => m.decimales === 2 && m.es_fabrica === true));
  });

  test("2. Resolución temporal histórica selecciona la tasa más reciente <= T", () => {
    const historial: TasaCambioItem[] = [
      { id: "1", origen: "USD", destino: "VES", valor: 35.0, fecha_vigencia: "2026-09-01", momento_registro: "2026-09-01T10:00:00Z" },
      { id: "2", origen: "USD", destino: "VES", valor: 40.0, fecha_vigencia: "2026-10-01", momento_registro: "2026-10-01T10:00:00Z" },
    ];

    // Consulta antes de la primera tasa registrada
    const anterior = resolverTasaVigente(historial, "USD", "VES", "2026-08-30");
    assert.strictEqual(anterior, null);

    // Consulta en fecha intermedia toma la tasa previa de septiembre (35.0)
    const intermedia = resolverTasaVigente(historial, "USD", "VES", "2026-09-15");
    assert.strictEqual(intermedia?.tasa, 35.0);
    assert.strictEqual(intermedia?.esInversa, false);

    // Consulta posterior a octubre toma la tasa de octubre (40.0)
    const posterior = resolverTasaVigente(historial, "USD", "VES", "2026-10-15");
    assert.strictEqual(posterior?.tasa, 40.0);
  });

  test("3. Tasa inversa automática calcula 1 / valor cuando solo existe el par opuesto", () => {
    const historial: TasaCambioItem[] = [
      { id: "1", origen: "USD", destino: "VES", valor: 40.0, fecha_vigencia: "2026-10-01", momento_registro: "2026-10-01T10:00:00Z" },
    ];

    // Buscamos VES -> USD cuando sólo tenemos USD -> VES
    const resInversa = resolverTasaVigente(historial, "VES", "USD", "2026-10-15");
    assert.ok(resInversa !== null);
    assert.strictEqual(resInversa.esInversa, true);
    assert.strictEqual(resInversa.tasa, 1 / 40.0); // 0.025
  });

  test("4. Prohibición estricta de triangulación (no inventa tasas puente)", () => {
    const historial: TasaCambioItem[] = [
      { id: "1", origen: "USD", destino: "VES", valor: 40.0, fecha_vigencia: "2026-10-01", momento_registro: "2026-10-01T10:00:00Z" },
      { id: "2", origen: "EUR", destino: "USD", valor: 1.10, fecha_vigencia: "2026-10-01", momento_registro: "2026-10-01T10:00:00Z" },
    ];

    // Intentar convertir EUR -> VES cuando no existe tasa directa ni inversa EUR-VES
    const res = resolverTasaVigente(historial, "EUR", "VES", "2026-10-15");
    assert.strictEqual(res, null);
  });

  test("5. Valuación de patrimonio consolidado con conversión exacta a moneda base", async () => {
    localStorage.clear();
    await cambiarMonedaBase("USD");
    await registrarTasa({ origen: "USD", destino: "VES", valor: 40.0, fecha_vigencia: "2026-10-01" });

    const cuentas = [
      { id: "c1", nombre: "Banesco", tipo: "banco", moneda: "VES", saldo: 400.0 }, // 400 / 40 = 10 USD
      { id: "c2", nombre: "Zelle", tipo: "banco", moneda: "USD", saldo: 100.0 },   // 100 USD = 100 USD
    ];

    const patrimonio = await consultarPatrimonio("2026-10-05", cuentas);
    assert.strictEqual(patrimonio.moneda_base, "USD");
    assert.strictEqual(patrimonio.total_consolidado, 110.0);
    assert.strictEqual(patrimonio.tasa_faltante, null);

    // Preserva subtotales por moneda original
    assert.strictEqual(patrimonio.subtotales.length, 2);
    const subVES = patrimonio.subtotales.find(s => s.moneda === "VES");
    const subUSD = patrimonio.subtotales.find(s => s.moneda === "USD");
    assert.strictEqual(subVES?.subtotal, 400.0);
    assert.strictEqual(subUSD?.subtotal, 100.0);
  });

  test("6. Bloqueo estricto del total consolidado cuando falta una tasa con saldo activo", async () => {
    localStorage.clear();
    await cambiarMonedaBase("USD");
    await registrarTasa({ origen: "USD", destino: "VES", valor: 40.0, fecha_vigencia: "2026-10-01" });

    // Cuenta en EUR con saldo sin tasa hacia USD
    const cuentas = [
      { id: "c1", nombre: "Banesco", tipo: "banco", moneda: "VES", saldo: 400.0 },
      { id: "c2", nombre: "Banco Europa", tipo: "banco", moneda: "EUR", saldo: 50.0 },
    ];

    const patrimonio = await consultarPatrimonio("2026-10-05", cuentas);
    assert.strictEqual(patrimonio.total_consolidado, null);
    assert.ok(patrimonio.tasa_faltante !== null);
    assert.strictEqual(patrimonio.tasa_faltante?.moneda_origen, "EUR");
    assert.strictEqual(patrimonio.tasa_faltante?.moneda_destino, "USD");

    // Los subtotales por moneda de origen siguen mostrándose siempre
    assert.strictEqual(patrimonio.subtotales.length, 2);
  });

  test("7. Cuenta con saldo 0 sin tasa de cambio no bloquea el patrimonio consolidado", async () => {
    localStorage.clear();
    await cambiarMonedaBase("USD");
    await registrarTasa({ origen: "USD", destino: "VES", valor: 40.0, fecha_vigencia: "2026-10-01" });

    // Cuenta en EUR pero con saldo exactamente 0.0
    const cuentas = [
      { id: "c1", nombre: "Banesco", tipo: "banco", moneda: "VES", saldo: 400.0 },
      { id: "c2", nombre: "Cuenta Vacía", tipo: "banco", moneda: "EUR", saldo: 0.0 },
    ];

    const patrimonio = await consultarPatrimonio("2026-10-05", cuentas);
    assert.strictEqual(patrimonio.total_consolidado, 10.0);
    assert.strictEqual(patrimonio.tasa_faltante, null);
  });

  test("8. Manejo algebraico de saldo negativo (sobregiro) en el total consolidado", async () => {
    localStorage.clear();
    await cambiarMonedaBase("USD");
    await registrarTasa({ origen: "USD", destino: "VES", valor: 40.0, fecha_vigencia: "2026-10-01" });

    const cuentas = [
      { id: "c1", nombre: "Zelle", tipo: "banco", moneda: "USD", saldo: 100.0 },
      { id: "c2", nombre: "Tarjeta Sobregirada", tipo: "tarjeta_credito", moneda: "VES", saldo: -200.0 }, // -200 / 40 = -5 USD
    ];

    const patrimonio = await consultarPatrimonio("2026-10-05", cuentas);
    assert.strictEqual(patrimonio.total_consolidado, 95.0);
  });

  test("9. Cambio dinámico de moneda base recalcula la unidad de valuación", async () => {
    localStorage.clear();
    await registrarTasa({ origen: "USD", destino: "VES", valor: 40.0, fecha_vigencia: "2026-10-01" });
    await cambiarMonedaBase("VES");

    const base = await obtenerMonedaBase();
    assert.strictEqual(base, "VES");

    const cuentas = [
      { id: "c1", nombre: "Efectivo USD", tipo: "efectivo", moneda: "USD", saldo: 50.0 }, // 50 * 40 = 2000 VES
      { id: "c2", nombre: "Pago Móvil", tipo: "banco", moneda: "VES", saldo: 500.0 },     // 500 VES
    ];

    const patrimonio = await consultarPatrimonio("2026-10-05", cuentas);
    assert.strictEqual(patrimonio.moneda_base, "VES");
    assert.strictEqual(patrimonio.total_consolidado, 2500.0);
  });

  test("10. Formato de montos y soporte de emojis asignados a categorías", () => {
    const formateadoVES = formatearMonto(1500.75, "VES");
    assert.ok(formateadoVES.includes("Bs."));
    assert.ok(formateadoVES.includes("1.500,75"));

    const categoriaConEmoji = "🍔 Hamburguesas";
    const regexEmoji = /\p{Extended_Pictographic}/u;
    assert.ok(regexEmoji.test(categoriaConEmoji));
  });

});
