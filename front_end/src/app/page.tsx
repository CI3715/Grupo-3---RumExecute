"use client";

import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import Navbar from "../components/Navbar";
import SubtotalesGrid, { SubtotalMoneda } from "../components/SubtotalesGrid";
import ListaCuentas, { CuentaItem } from "../components/ListaCuentas";
import ModalNuevaCuenta, { EntradaCuentaForm, ErrorValidacion } from "../components/ModalNuevaCuenta";
import ModalDiagnostico from "../components/ModalDiagnostico";
import DetalleCuenta, { DetalleCuentaData } from "../components/DetalleCuenta";
import { EntradaMovimientoForm } from "../components/ModalNuevoMovimiento";
import { EntradaCorreccionForm, MovimientoItem } from "../components/ModalCorregirMovimiento";
import {
  formatearMonto,
  formatearMontoConSigno,
  calcularTotalesCuenta,
} from "../utils/formato";

interface CuentaRust {
  id: string;
  nombre: string;
  tipo: string;
  moneda: string;
  saldo_inicial: { cantidad: string | number; moneda: string };
}

interface ResumenCuentasRust {
  cuentas: CuentaRust[];
  subtotales_por_moneda: Array<{ cantidad: string | number; moneda: string }>;
}

const enTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export default function HomePage() {
  const [vista, setVista] = useState<"cuentas" | "detalle">("cuentas");
  const [cuentaSeleccionadaId, setCuentaSeleccionadaId] = useState<string | null>(null);

  const [cuentas, setCuentas] = useState<CuentaItem[]>([]);
  const [subtotales, setSubtotales] = useState<SubtotalMoneda[]>([]);
  const [saldosIniciales, setSaldosIniciales] = useState<Record<string, number>>({});
  const [movimientosPorCuenta, setMovimientosPorCuenta] = useState<Record<string, MovimientoItem[]>>({});

  const [modalNuevaCuenta, setModalNuevaCuenta] = useState(false);
  const [modalDiagnostico, setModalDiagnostico] = useState(false);
  const [errorValidacion, setErrorValidacion] = useState<ErrorValidacion | null>(null);

  const cargarCuentasTauri = async () => {
    if (!enTauri()) return;
    try {
      const res = await invoke<ResumenCuentasRust>("listar_cuentas");
      if (!res?.cuentas) return;

      const mapaSaldos: Record<string, number> = {};
      const nuevasCuentas: CuentaItem[] = res.cuentas.map((c) => {
        mapaSaldos[c.id] = Number(c.saldo_inicial?.cantidad ?? 0);
        return {
          id: c.id,
          nombre: c.nombre,
          tipo: c.tipo,
          moneda: c.moneda,
          saldo_formateado: formatearMonto(c.saldo_inicial?.cantidad ?? "0", c.moneda),
        };
      });

      const nuevosSubtotales: SubtotalMoneda[] = (res.subtotales_por_moneda || []).map((s) => ({
        moneda: s.moneda,
        subtotal_formateado: formatearMonto(s.cantidad, s.moneda),
        cantidad_cuentas: res.cuentas.filter((c) => c.moneda === s.moneda).length,
      }));

      setSaldosIniciales((prev) => ({ ...mapaSaldos, ...prev }));
      setCuentas(nuevasCuentas);
      setSubtotales(nuevosSubtotales);
    } catch (err) {
      console.warn("Error al cargar cuentas desde Tauri IPC:", err);
    }
  };

  useEffect(() => {
    cargarCuentasTauri();
  }, []);

  const sincronizarSaldosYSubtotales = (
    listaCuentas: CuentaItem[],
    mapaInicial: Record<string, number>,
    mapaMovs: Record<string, MovimientoItem[]>
  ) => {
    const cuentasActualizadas = listaCuentas.map((c) => {
      const { saldoActual } = calcularTotalesCuenta(mapaInicial[c.id] ?? 0, mapaMovs[c.id]);
      return { ...c, saldo_formateado: formatearMonto(saldoActual, c.moneda) };
    });

    const monedas = Array.from(new Set(cuentasActualizadas.map((c) => c.moneda)));
    const nuevosSubtotales: SubtotalMoneda[] = monedas.map((codMoneda) => {
      const delGrupo = cuentasActualizadas.filter((c) => c.moneda === codMoneda);
      const total = delGrupo.reduce((acum, c) => {
        const { saldoActual } = calcularTotalesCuenta(mapaInicial[c.id] ?? 0, mapaMovs[c.id]);
        return acum + saldoActual;
      }, 0);
      return {
        moneda: codMoneda,
        subtotal_formateado: formatearMonto(total, codMoneda),
        cantidad_cuentas: delGrupo.length,
      };
    });

    setCuentas(cuentasActualizadas);
    setSubtotales(nuevosSubtotales);
  };

  const abrirDetalleCuenta = async (id: string) => {
    setCuentaSeleccionadaId(id);
    setVista("detalle");

    if (!enTauri()) return;
    try {
      // Intenta primero obtener el detalle completo proyectado por Rust (PR #10)
      const detalle = await invoke<any>("obtener_detalle_cuenta", { idCuenta: id })
        .catch(() => invoke<any>("obtener_detalle_cuenta", { id_cuenta: id }))
        .catch(() => null);

      const listaMovs = detalle?.movimientos ?? detalle;

      if (Array.isArray(listaMovs)) {
        const moneda = cuentas.find((c) => c.id === id)?.moneda || "VES";
        const movs: MovimientoItem[] = listaMovs.map((m: any) => {
          const montoNum = Number(m.monto?.cantidad ?? m.monto ?? 0);
          const tipoNorm = (typeof m.tipo === "string" ? m.tipo : m.tipo?.tipo || "gasto").toLowerCase();
          
          // En Rust m.categoria es un struct { id, nombre, tipo }
          const catTexto = typeof m.categoria === "object" && m.categoria !== null
            ? (m.categoria.nombre || m.categoria.id || "Varios")
            : (typeof m.categoria === "string" ? m.categoria : "Varios");

          return {
            id: m.id,
            id_cuenta: m.id_cuenta || id,
            tipo: tipoNorm as "gasto" | "ingreso" | "ajuste",
            monto_num: montoNum,
            monto_formateado: formatearMontoConSigno(montoNum, moneda, tipoNorm),
            fecha: m.fecha || new Date().toISOString().split("T")[0],
            categoria: catTexto,
            nota: m.descripcion || m.nota || "",
            estado: (tipoNorm === "ajuste" ? "ajuste" : "vigente") as "vigente" | "corregido" | "ajuste",
            id_movimiento_referenciado: m.id_movimiento_referenciado,
          };
        });
        setMovimientosPorCuenta((prev) => ({ ...prev, [id]: movs }));
      }
    } catch (err) {
      console.warn("No se pudo cargar detalle desde Tauri, usando almacén local:", err);
    }
  };

  const handleCrearCuenta = async (entrada: EntradaCuentaForm): Promise<boolean> => {
    setErrorValidacion(null);

    if (enTauri()) {
      try {
        await invoke("crear_cuenta", { entrada });
        await cargarCuentasTauri();
        return true;
      } catch (err: any) {
        setErrorValidacion({
          campo: err?.campo || "general",
          mensaje: err?.mensaje || (typeof err === "string" ? err : "Error al registrar la cuenta"),
        });
        return false;
      }
    }

    if (!entrada.nombre.trim()) {
      setErrorValidacion({ campo: "nombre", mensaje: "El nombre de la cuenta no puede estar vacío." });
      return false;
    }
    if (cuentas.some((c) => c.nombre.trim().toLowerCase() === entrada.nombre.trim().toLowerCase())) {
      setErrorValidacion({
        campo: "nombre",
        mensaje: `Ya existe una cuenta con el nombre '${entrada.nombre}'. Elige un nombre distinto.`,
      });
      return false;
    }

    const saldoNorm = entrada.saldo_inicial.replace(",", ".");
    const numeroSaldo = Number(saldoNorm);
    if (isNaN(numeroSaldo) || numeroSaldo < 0) {
      setErrorValidacion({
        campo: "saldo_inicial",
        mensaje: "El saldo inicial no puede ser negativo ni contener letras.",
      });
      return false;
    }

    const nuevoId = `cta-${Date.now()}`;
    const nuevaCuenta: CuentaItem = {
      id: nuevoId,
      nombre: entrada.nombre.trim(),
      tipo: entrada.tipo,
      moneda: entrada.moneda,
      saldo_formateado: formatearMonto(numeroSaldo, entrada.moneda),
    };

    const nuevasCuentas = [...cuentas, nuevaCuenta];
    const nuevosIniciales = { ...saldosIniciales, [nuevoId]: numeroSaldo };
    const nuevosMovs = { ...movimientosPorCuenta, [nuevoId]: [] };

    setSaldosIniciales(nuevosIniciales);
    setMovimientosPorCuenta(nuevosMovs);
    sincronizarSaldosYSubtotales(nuevasCuentas, nuevosIniciales, nuevosMovs);
    return true;
  };

  const handleRegistrarMovimiento = async (entrada: EntradaMovimientoForm): Promise<boolean> => {
    if (enTauri()) {
      try {
        const payloadRust = {
          id_cuenta: entrada.id_cuenta,
          tipo: entrada.tipo,
          monto: entrada.monto,
          categoria: entrada.categoria,
          fecha: entrada.fecha || new Date().toISOString().split("T")[0],
          descripcion: entrada.nota?.trim() || "Sin descripción",
        };
        await invoke("registrar_movimiento", { entrada: payloadRust });
        await cargarCuentasTauri();
        await abrirDetalleCuenta(entrada.id_cuenta);
      } catch (err) {
        console.warn("Usando almacén local para registrar_movimiento:", err);
      }
    }

    const cuenta = cuentas.find((c) => c.id === entrada.id_cuenta);
    if (!cuenta) return false;

    const montoNum = parseFloat(entrada.monto.replace(",", "."));
    const nuevoMov: MovimientoItem = {
      id: `mov-${Date.now()}`,
      id_cuenta: entrada.id_cuenta,
      tipo: entrada.tipo,
      monto_num: montoNum,
      monto_formateado: formatearMontoConSigno(montoNum, cuenta.moneda, entrada.tipo),
      fecha: entrada.fecha || new Date().toISOString().split("T")[0],
      categoria: entrada.categoria,
      nota: entrada.nota?.trim(),
      estado: "vigente",
    };

    const nuevosMovs = {
      ...movimientosPorCuenta,
      [entrada.id_cuenta]: [nuevoMov, ...(movimientosPorCuenta[entrada.id_cuenta] || [])],
    };
    setMovimientosPorCuenta(nuevosMovs);
    sincronizarSaldosYSubtotales(cuentas, saldosIniciales, nuevosMovs);
    return true;
  };

  const handleCorregirMovimiento = async (entrada: EntradaCorreccionForm): Promise<boolean> => {
    if (enTauri()) {
      try {
        const payloadRust = {
          id_movimiento_original: entrada.id_movimiento_original,
          nuevo_monto: entrada.monto_correcto,
          fecha: new Date().toISOString().split("T")[0],
          descripcion: entrada.motivo?.trim() || "Ajuste de corrección",
        };
        await invoke("corregir_movimiento", { entrada: payloadRust });
        await cargarCuentasTauri();
        if (cuentaSeleccionadaId) {
          await abrirDetalleCuenta(cuentaSeleccionadaId);
        }
      } catch (err) {
        console.warn("Usando almacén local para corregir_movimiento:", err);
      }
    }

    if (!cuentaSeleccionadaId) return false;
    const cuenta = cuentas.find((c) => c.id === cuentaSeleccionadaId);
    const movsActuales = movimientosPorCuenta[cuentaSeleccionadaId] || [];
    const original = movsActuales.find((m) => m.id === entrada.id_movimiento_original);
    if (!cuenta || !original) return false;

    const montoCorrectoNum = parseFloat(entrada.monto_correcto.replace(",", "."));
    const diff =
      original.tipo === "gasto"
        ? original.monto_num - montoCorrectoNum
        : montoCorrectoNum - original.monto_num;

    const movAjuste: MovimientoItem = {
      id: `mov-ajuste-${Date.now()}`,
      id_cuenta: cuentaSeleccionadaId,
      tipo: "ajuste",
      monto_num: diff,
      monto_formateado: formatearMontoConSigno(diff, cuenta.moneda, "ajuste"),
      fecha: new Date().toISOString().split("T")[0],
      categoria: original.categoria,
      nota: entrada.motivo?.trim()
        ? `Ajuste: ${entrada.motivo.trim()}`
        : `Ajuste por corrección de monto (${original.monto_num} → ${montoCorrectoNum})`,
      estado: "ajuste",
      id_movimiento_referenciado: original.id,
    };

    const listaActualizada = [
      movAjuste,
      ...movsActuales.map((m) => (m.id === original.id ? { ...m, estado: "corregido" as const } : m)),
    ];

    const nuevosMovs = { ...movimientosPorCuenta, [cuentaSeleccionadaId]: listaActualizada };
    setMovimientosPorCuenta(nuevosMovs);
    sincronizarSaldosYSubtotales(cuentas, saldosIniciales, nuevosMovs);
    return true;
  };

  const cuentaActivaDetalle = useMemo<DetalleCuentaData | null>(() => {
    if (!cuentaSeleccionadaId) return null;
    const cuenta = cuentas.find((c) => c.id === cuentaSeleccionadaId);
    if (!cuenta) return null;

    const saldoIni = saldosIniciales[cuenta.id] ?? 0;
    const movs = movimientosPorCuenta[cuenta.id] || [];
    const { totalIngresos, totalGastos, totalAjustes, saldoActual } = calcularTotalesCuenta(saldoIni, movs);

    return {
      id: cuenta.id,
      nombre: cuenta.nombre,
      tipo: cuenta.tipo,
      moneda: cuenta.moneda,
      saldo_inicial_num: saldoIni,
      saldo_actual_num: saldoActual,
      total_ingresos_num: totalIngresos,
      total_gastos_num: totalGastos,
      total_ajustes_num: totalAjustes,
      movimientos: movs,
    };
  }, [cuentaSeleccionadaId, cuentas, saldosIniciales, movimientosPorCuenta]);

  return (
    <div className="app-wrapper">
      <Navbar
        onOpenNuevaCuenta={() => setModalNuevaCuenta(true)}
        onOpenDiagnostico={() => setModalDiagnostico(true)}
        onNavigateHome={() => {
          setVista("cuentas");
          setCuentaSeleccionadaId(null);
        }}
      />

      <main className="app-main">
        {vista === "cuentas" ? (
          <>
            <div className="hero-section">
              <div>
                <h1 className="hero-title">Tu dinero, en orden.</h1>
                <p className="hero-subtitle">
                  Todas tus cuentas en un solo lugar, sin mezclar monedas.
                </p>
              </div>
              <div className="security-pill">
                <span>🛡️</span>
                <span>Solo en este dispositivo</span>
              </div>
            </div>

            <SubtotalesGrid subtotales={subtotales} />

            <ListaCuentas
              cuentas={cuentas}
              onCrearPrimeraCuenta={() => setModalNuevaCuenta(true)}
              onSeleccionarCuenta={abrirDetalleCuenta}
            />
          </>
        ) : (
          cuentaActivaDetalle && (
            <DetalleCuenta
              cuenta={cuentaActivaDetalle}
              onVolver={() => {
                setVista("cuentas");
                setCuentaSeleccionadaId(null);
              }}
              onRegistrarMovimiento={handleRegistrarMovimiento}
              onCorregirMovimiento={handleCorregirMovimiento}
            />
          )
        )}
      </main>

      <footer className="app-footer">
        <div style={{ display: "flex", alignItems: "center", gap: "8px", fontWeight: 500 }}>
          <span>💾</span>
          <span>Datos guardados localmente</span>
        </div>
        <div style={{ color: "var(--text-muted)", fontWeight: 500 }}>
          Sin nube. Sin conexión. Siempre tuyos.
        </div>
      </footer>

      <ModalNuevaCuenta
        isOpen={modalNuevaCuenta}
        onClose={() => {
          setModalNuevaCuenta(false);
          setErrorValidacion(null);
        }}
        onSubmit={handleCrearCuenta}
        errorExterno={errorValidacion}
      />

      <ModalDiagnostico
        isOpen={modalDiagnostico}
        onClose={() => setModalDiagnostico(false)}
      />
    </div>
  );
}
