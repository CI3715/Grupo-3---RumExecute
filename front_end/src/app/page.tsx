"use client";

import { useState, useEffect, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import Navbar from "../components/Navbar";
import SubtotalesGrid, { SubtotalMoneda } from "../components/SubtotalesGrid";
import ListaCuentas, { CuentaItem } from "../components/ListaCuentas";
import ModalNuevaCuenta, { EntradaCuentaForm, ErrorValidacion } from "../components/ModalNuevaCuenta";
import ModalDiagnostico from "../components/ModalDiagnostico";
import DetalleCuenta, { DetalleCuentaData } from "../components/DetalleCuenta";

export interface EntradaNuevaCategoriaForm {
  nombre: string;
  tipo: "gasto" | "ingreso";
}
import { EntradaMovimientoForm } from "../components/ModalNuevoMovimiento";
import { EntradaCorreccionForm, MovimientoItem } from "../components/ModalCorregirMovimiento";
import {
  CATEGORIAS_GASTO,
  CATEGORIAS_INGRESO,
  formatearMonto,
  formatearMontoConSigno,
  calcularTotalesCuenta,
} from "../utils/formato";

// Importaciones de Entrega 4 (The Exchange Flow & Patrimonio)
import {
  MonedaItem,
  TasaCambioItem,
  ResumenPatrimonioData,
  EntradaRegistroTasa,
  EntradaCrearMoneda,
} from "../types/tasas";
import {
  obtenerFechaHoyISO,
  listarMonedas,
  obtenerMonedaBase,
  cambiarMonedaBase,
  registrarTasa,
  eliminarTasa,
  listarHistorialTasas,
  consultarPatrimonio,
  crearMonedaPropia,
} from "../services/tasasService";
import VistaPatrimonio from "../components/VistaPatrimonio";
import VistaTasas from "../components/VistaTasas";

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
  // Navegación de pestañas (Entrega 4)
  const [tabActivo, setTabActivo] = useState<"cuentas" | "patrimonio" | "tasas">("cuentas");

  // Vistas internas de Cuentas
  const [vista, setVista] = useState<"cuentas" | "detalle">("cuentas");
  const [cuentaSeleccionadaId, setCuentaSeleccionadaId] = useState<string | null>(null);

  // Estado de Cuentas y Movimientos
  const [cuentas, setCuentas] = useState<CuentaItem[]>([]);
  const [subtotales, setSubtotales] = useState<SubtotalMoneda[]>([]);
  const [saldosIniciales, setSaldosIniciales] = useState<Record<string, number>>({});
  const [movimientosPorCuenta, setMovimientosPorCuenta] = useState<Record<string, MovimientoItem[]>>({});

  const [categoriasGasto, setCategoriasGasto] = useState<string[]>(CATEGORIAS_GASTO);
  const [categoriasIngreso, setCategoriasIngreso] = useState<string[]>(CATEGORIAS_INGRESO);

  // Estado de Tasas, Monedas y Patrimonio (Entrega 4)
  const [fechaConsultaPatrimonio, setFechaConsultaPatrimonio] = useState<string>(obtenerFechaHoyISO());
  const [monedaBaseActual, setMonedaBaseActual] = useState<string>("USD");
  const [monedasDisponibles, setMonedasDisponibles] = useState<MonedaItem[]>([]);
  const [tasasHistorial, setTasasHistorial] = useState<TasaCambioItem[]>([]);
  const [patrimonioData, setPatrimonioData] = useState<ResumenPatrimonioData | null>(null);
  const [filtroParHistorial, setFiltroParHistorial] = useState<{ origen?: string; destino?: string } | null>(null);

  // Modales
  const [modalNuevaCuenta, setModalNuevaCuenta] = useState(false);
  const [modalDiagnostico, setModalDiagnostico] = useState(false);
  const [errorValidacion, setErrorValidacion] = useState<ErrorValidacion | null>(null);

  // ==========================================
  // CARGA DE DATOS DE CUENTAS (TAURI / LOCAL)
  // ==========================================
  const cargarCuentasTauri = async () => {
    if (!enTauri()) return;
    try {
      const res = await invoke<ResumenCuentasRust>("listar_cuentas");
      if (!res?.cuentas) return;

      const mapaSaldos: Record<string, number> = {};
      const mapaMovs: Record<string, MovimientoItem[]> = {};

      const detalles = await Promise.all(
        res.cuentas.map(async (c) => {
          try {
            const d = await invoke<any>("obtener_detalle_cuenta", { idCuenta: c.id })
              .catch(() => invoke<any>("obtener_detalle_cuenta", { id_cuenta: c.id }));
            return { id: c.id, detalle: d };
          } catch {
            return { id: c.id, detalle: null };
          }
        })
      );

      const nuevasCuentas: CuentaItem[] = res.cuentas.map((c) => {
        const saldoIniNum = Number(c.saldo_inicial?.cantidad ?? 0);
        mapaSaldos[c.id] = saldoIniNum;

        const info = detalles.find((d) => d.id === c.id)?.detalle;
        let saldoActualNum = saldoIniNum;

        if (info?.saldo_actual) {
          saldoActualNum = Number(info.saldo_actual.cantidad ?? saldoIniNum);
        }

        if (info && Array.isArray(info.movimientos)) {
          const movs: MovimientoItem[] = info.movimientos.map((m: any) => {
            const montoNum = Number(m.monto?.cantidad ?? m.monto ?? 0);
            const tipoNorm = (typeof m.tipo === "string" ? m.tipo : m.tipo?.tipo || "gasto").toLowerCase();
            const catTexto = typeof m.categoria === "object" && m.categoria !== null
              ? (m.categoria.nombre || m.categoria.id || "Varios")
              : (typeof m.categoria === "string" ? m.categoria : "Varios");

            return {
              id: m.id,
              id_cuenta: m.id_cuenta || c.id,
              tipo: tipoNorm as "gasto" | "ingreso" | "ajuste",
              monto_num: montoNum,
              monto_formateado: formatearMontoConSigno(montoNum, c.moneda, tipoNorm),
              fecha: m.fecha || new Date().toISOString().split("T")[0],
              categoria: catTexto,
              nota: m.descripcion || m.nota || "",
              estado: (tipoNorm === "ajuste" ? "ajuste" : "vigente") as "vigente" | "corregido" | "ajuste",
              id_movimiento_referenciado: m.id_movimiento_referenciado,
            };
          });
          mapaMovs[c.id] = movs;
        }

        return {
          id: c.id,
          nombre: c.nombre,
          tipo: c.tipo,
          moneda: c.moneda,
          saldo_formateado: formatearMonto(saldoActualNum, c.moneda),
        };
      });

      const monedas = Array.from(new Set(nuevasCuentas.map((c) => c.moneda)));
      const nuevosSubtotales: SubtotalMoneda[] = monedas.map((codMoneda) => {
        const cuentasMoneda = nuevasCuentas.filter((c) => c.moneda === codMoneda);
        const total = cuentasMoneda.reduce((acum, c) => {
          const saldo = detalles.find((d) => d.id === c.id)?.detalle?.saldo_actual?.cantidad;
          return acum + Number(saldo ?? mapaSaldos[c.id] ?? 0);
        }, 0);

        return {
          moneda: codMoneda,
          subtotal_formateado: formatearMonto(total, codMoneda),
          cantidad_cuentas: cuentasMoneda.length,
        };
      });

      setSaldosIniciales((prev) => ({ ...mapaSaldos, ...prev }));
      setMovimientosPorCuenta((prev) => ({ ...prev, ...mapaMovs }));
      setCuentas(nuevasCuentas);
      setSubtotales(nuevosSubtotales);
    } catch (e) {
      console.error("Error al cargar cuentas de Tauri:", e);
    }
  };

  const cargarCategoriasTauri = async () => {
    if (!enTauri()) return;
    try {
      const res = await invoke<any>("listar_categorias");
      if (res) {
        if (Array.isArray(res.gastos) && res.gastos.length > 0) {
          setCategoriasGasto(res.gastos.map((c: any) => typeof c === "string" ? c : c.nombre));
        }
        if (Array.isArray(res.ingresos) && res.ingresos.length > 0) {
          setCategoriasIngreso(res.ingresos.map((c: any) => typeof c === "string" ? c : c.nombre));
        }
      }
    } catch {}
  };

  // ==========================================
  // CARGA DE TASAS, MONEDAS Y PATRIMONIO
  // ==========================================
  const cargarMonedasYBase = useCallback(async () => {
    try {
      const mons = await listarMonedas();
      setMonedasDisponibles(mons);
      const base = await obtenerMonedaBase();
      setMonedaBaseActual(base);
    } catch (e) {
      console.error("Error al cargar monedas:", e);
    }
  }, []);

  const cargarTasas = useCallback(async () => {
    try {
      const lista = await listarHistorialTasas();
      setTasasHistorial(lista);
    } catch (e) {
      console.error("Error al cargar tasas:", e);
    }
  }, []);

  const actualizarPatrimonio = useCallback(async () => {
    try {
      const cuentasSimples = cuentas.map((c) => {
        const saldoIni = saldosIniciales[c.id] ?? 0;
        const movs = movimientosPorCuenta[c.id] || [];
        const { saldoActual } = calcularTotalesCuenta(saldoIni, movs);
        return {
          id: c.id,
          nombre: c.nombre,
          tipo: c.tipo,
          moneda: c.moneda,
          saldo: saldoActual,
        };
      });

      const res = await consultarPatrimonio(fechaConsultaPatrimonio, cuentasSimples);
      setPatrimonioData(res);
    } catch (e) {
      console.error("Error al consultar patrimonio:", e);
    }
  }, [cuentas, saldosIniciales, movimientosPorCuenta, fechaConsultaPatrimonio]);

  useEffect(() => {
    cargarCuentasTauri();
    cargarCategoriasTauri();
    cargarMonedasYBase();
    cargarTasas();
  }, [cargarMonedasYBase, cargarTasas]);

  useEffect(() => {
    if (tabActivo === "patrimonio") {
      actualizarPatrimonio();
    }
  }, [tabActivo, fechaConsultaPatrimonio, monedaBaseActual, cuentas, actualizarPatrimonio]);

  // ==========================================
  // HANDLERS DE TASAS Y MONEDA BASE
  // ==========================================
  const handleCambiarMonedaBase = async (nuevaBase: string) => {
    try {
      await cambiarMonedaBase(nuevaBase);
      setMonedaBaseActual(nuevaBase);
      await actualizarPatrimonio();
    } catch (err: any) {
      alert(err?.message || "Error al cambiar moneda base.");
    }
  };

  const handleGuardarTasa = async (entrada: EntradaRegistroTasa) => {
    await registrarTasa(entrada);
    await cargarTasas();
    await actualizarPatrimonio();
  };

  const handleEliminarTasa = async (idTasa: string) => {
    await eliminarTasa(idTasa);
    await cargarTasas();
    await actualizarPatrimonio();
  };

  const handleCrearMoneda = async (entrada: EntradaCrearMoneda) => {
    await crearMonedaPropia(entrada);
    await cargarMonedasYBase();
    await actualizarPatrimonio();
  };

  // ==========================================
  // HANDLERS DE CUENTAS Y MOVIMIENTOS
  // ==========================================
  const handleCrearCuenta = async (entrada: EntradaCuentaForm): Promise<boolean> => {
    setErrorValidacion(null);

    if (enTauri()) {
      try {
        await invoke("crear_cuenta", { entrada });
        await cargarCuentasTauri();
        setModalNuevaCuenta(false);
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

    const nuevaId = `cta-${Date.now()}`;
    const saldoNum = parseFloat(entrada.saldo_inicial.replace(",", "."));
    const nuevaCuenta: CuentaItem = {
      id: nuevaId,
      nombre: entrada.nombre,
      tipo: entrada.tipo,
      moneda: entrada.moneda,
      saldo_formateado: formatearMonto(saldoNum, entrada.moneda),
    };

    setSaldosIniciales((prev) => ({ ...prev, [nuevaId]: saldoNum }));
    setCuentas((prev) => [...prev, nuevaCuenta]);
    setModalNuevaCuenta(false);
    return true;
  };

  const handleCrearCategoria = async (
    entrada: EntradaNuevaCategoriaForm
  ): Promise<boolean> => {
    setErrorValidacion(null);
    const nombreLimpio = entrada.nombre.trim();
    if (!nombreLimpio) {
      const err = {
        campo: "categoria",
        mensaje: "El nombre de la categoría no puede estar vacío.",
      };
      setErrorValidacion(err);
      throw err;
    }

    if (enTauri()) {
      try {
        await invoke("crear_categoria", {
          entrada: {
            nombre: nombreLimpio,
            tipo: entrada.tipo,
          },
        });
        await cargarCategoriasTauri();
        return true;
      } catch (err: any) {
        const errorObj = {
          campo: err?.campo || "categoria",
          mensaje:
            err?.mensaje ||
            (typeof err === "string" ? err : "Error al crear la categoría"),
        };
        setErrorValidacion(errorObj);
        throw errorObj;
      }
    }

    if (entrada.tipo === "gasto") {
      setCategoriasGasto((prev) => [...prev, nombreLimpio]);
    } else {
      setCategoriasIngreso((prev) => [...prev, nombreLimpio]);
    }
    return true;
  };

  const handleRegistrarMovimiento = async (
    entrada: EntradaMovimientoForm
  ): Promise<boolean> => {
    setErrorValidacion(null);
    const hoy = new Date().toISOString().split("T")[0];
    const fechaMovimiento = entrada.fecha?.trim() || hoy;

    if (fechaMovimiento > hoy) {
      const errorObj = {
        campo: "fecha",
        mensaje: `No se permiten movimientos con fechas futuras (${fechaMovimiento} es posterior a la fecha actual ${hoy}).`,
      };
      setErrorValidacion(errorObj);
      throw errorObj;
    }

    if (entrada.esNuevaCategoria) {
      try {
        await handleCrearCategoria({
          nombre: entrada.categoria.trim(),
          tipo: entrada.tipo,
        });
      } catch (errCat: any) {
        const msg = (
          errCat?.mensaje ||
          (typeof errCat === "string" ? errCat : "")
        ).toLowerCase();
        if (!msg.includes("ya existe")) {
          throw errCat;
        }
      }
    }

    if (enTauri()) {
      try {
        const payloadRust = {
          id_cuenta: entrada.id_cuenta,
          tipo: entrada.tipo,
          monto: entrada.monto,
          categoria: entrada.categoria,
          fecha: fechaMovimiento,
          descripcion: entrada.nota?.trim() || "Sin descripción",
        };
        await invoke("registrar_movimiento", { entrada: payloadRust });
        await cargarCuentasTauri();
        if (cuentaSeleccionadaId) {
          await abrirDetalleCuenta(cuentaSeleccionadaId);
        }
        return true;
      } catch (err: any) {
        const errorObj = {
          campo: err?.campo || "monto",
          mensaje:
            err?.mensaje ||
            (typeof err === "string" ? err : "Error al registrar el movimiento"),
        };
        setErrorValidacion(errorObj);
        throw errorObj;
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
      fecha: fechaMovimiento,
      categoria: entrada.categoria,
      nota: entrada.nota,
      estado: "vigente",
    };

    setMovimientosPorCuenta((prev) => ({
      ...prev,
      [entrada.id_cuenta]: [nuevoMov, ...(prev[entrada.id_cuenta] || [])],
    }));
    return true;
  };

  const handleCorregirMovimiento = async (entrada: EntradaCorreccionForm): Promise<boolean> => {
    setErrorValidacion(null);
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
        return true;
      } catch (err: any) {
        const errorObj = {
          campo: err?.campo || "monto_correcto",
          mensaje: err?.mensaje || (typeof err === "string" ? err : "Error al corregir el movimiento"),
        };
        setErrorValidacion(errorObj);
        throw errorObj;
      }
    }

    const cuenta = cuentas.find((c) => c.id === cuentaSeleccionadaId);
    if (!cuenta) return false;

    const montoNuevoNum = parseFloat(entrada.monto_correcto.replace(",", "."));
    const movsActuales = movimientosPorCuenta[cuenta.id] || [];
    const movOriginal = movsActuales.find((m) => m.id === entrada.id_movimiento_original);
    if (!movOriginal) return false;

    let delta = 0;
    if (movOriginal.tipo === "gasto") {
      delta = movOriginal.monto_num - montoNuevoNum;
    } else {
      delta = montoNuevoNum - movOriginal.monto_num;
    }

    const movCorregido: MovimientoItem = {
      ...movOriginal,
      estado: "corregido",
    };

    const movAjuste: MovimientoItem = {
      id: `adj-${Date.now()}`,
      id_cuenta: cuenta.id,
      tipo: "ajuste",
      monto_num: Math.abs(delta),
      monto_formateado: formatearMontoConSigno(Math.abs(delta), cuenta.moneda, "ajuste"),
      fecha: new Date().toISOString().split("T")[0],
      categoria: "Ajuste por corrección",
      nota: entrada.motivo?.trim() || `Corrección del movimiento #${movOriginal.id}`,
      estado: "ajuste",
      id_movimiento_referenciado: movOriginal.id,
    };

    setMovimientosPorCuenta((prev) => ({
      ...prev,
      [cuenta.id]: [
        movAjuste,
        ...prev[cuenta.id].map((m) => (m.id === movOriginal.id ? movCorregido : m)),
      ],
    }));

    return true;
  };

  const abrirDetalleCuenta = (id: string) => {
    setCuentaSeleccionadaId(id);
    setVista("detalle");
  };

  const cuentaActivaDetalle = useMemo<DetalleCuentaData | null>(() => {
    if (!cuentaSeleccionadaId) return null;
    const cuenta = cuentas.find((c) => c.id === cuentaSeleccionadaId);
    if (!cuenta) return null;

    const saldoIni = saldosIniciales[cuenta.id] ?? 0;
    const movsBrutos = movimientosPorCuenta[cuenta.id] || [];

    const movs: MovimientoItem[] = movsBrutos.map((m: any) => {
      const catTexto = typeof m.categoria === "object" && m.categoria !== null
        ? (m.categoria.nombre || m.categoria.id || "Varios")
        : (typeof m.categoria === "string" ? m.categoria : "Varios");

      const tipoNorm = (typeof m.tipo === "string" ? m.tipo : m.tipo?.tipo || "gasto").toLowerCase();
      const montoNum = Number(m.monto_num ?? m.monto?.cantidad ?? m.monto ?? 0);

      return {
        ...m,
        id: String(m.id || ""),
        id_cuenta: String(m.id_cuenta || cuenta.id),
        tipo: tipoNorm as "gasto" | "ingreso" | "ajuste",
        monto_num: montoNum,
        monto_formateado: m.monto_formateado || formatearMontoConSigno(montoNum, cuenta.moneda, tipoNorm),
        fecha: String(m.fecha || new Date().toISOString().split("T")[0]),
        categoria: String(catTexto),
        nota: String(m.nota || m.descripcion || ""),
        estado: (tipoNorm === "ajuste" ? "ajuste" : m.estado || "vigente") as "vigente" | "corregido" | "ajuste",
        id_movimiento_referenciado: m.id_movimiento_referenciado ? String(m.id_movimiento_referenciado) : undefined,
      };
    });

    const { totalIngresos, totalGastos, totalAjustes, saldoActual } = calcularTotalesCuenta(saldoIni, movs);

    return {
      id: cuenta.id,
      nombre: String(cuenta.nombre),
      tipo: String(cuenta.tipo),
      moneda: String(cuenta.moneda),
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
        tabActivo={tabActivo}
        onCambiarTab={(tab) => {
          setTabActivo(tab);
          if (tab === "cuentas") {
            setVista("cuentas");
            setCuentaSeleccionadaId(null);
          }
        }}
        onOpenNuevaCuenta={() => setModalNuevaCuenta(true)}
        onOpenDiagnostico={() => setModalDiagnostico(true)}
        onNavigateHome={() => {
          setTabActivo("cuentas");
          setVista("cuentas");
          setCuentaSeleccionadaId(null);
          cargarCuentasTauri();
          cargarCategoriasTauri();
        }}
      />

      <main className="app-main">
        {/* ======================================================== */}
        {/* TAB 1: MIS CUENTAS Y MOVIMIENTOS                         */}
        {/* ======================================================== */}
        {tabActivo === "cuentas" && (
          vista === "cuentas" ? (
            <>
              <div className="hero-section">
                <div>
                  <h1 className="hero-title">Tu dinero, en orden.</h1>
                  <p className="hero-subtitle">
                    Todas tus cuentas en un solo lugar, sin mezclar monedas.
                  </p>
                </div>
                <div className="security-pill">
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
                categoriasGasto={categoriasGasto}
                categoriasIngreso={categoriasIngreso}
                onCrearCategoria={handleCrearCategoria}
                onVolver={() => {
                  setVista("cuentas");
                  setCuentaSeleccionadaId(null);
                  cargarCuentasTauri();
                  cargarCategoriasTauri();
                }}
                onRegistrarMovimiento={handleRegistrarMovimiento}
                onCorregirMovimiento={handleCorregirMovimiento}
              />
            )
          )
        )}

        {/* ======================================================== */}
        {/* TAB 2: PATRIMONIO CONSOLIDADO                            */}
        {/* ======================================================== */}
        {tabActivo === "patrimonio" && (
          patrimonioData ? (
            <VistaPatrimonio
              data={patrimonioData}
              monedasDisponibles={monedasDisponibles}
              monedaBaseActual={monedaBaseActual}
              fechaConsulta={fechaConsultaPatrimonio}
              onCambiarFecha={(f) => setFechaConsultaPatrimonio(f)}
              onCambiarMonedaBase={handleCambiarMonedaBase}
              onGuardarTasa={handleGuardarTasa}
              onIrATasas={(origen, destino) => {
                if (origen && destino) {
                  setFiltroParHistorial({ origen, destino });
                }
                setTabActivo("tasas");
              }}
              onOpenCrearMoneda={() => {
                setTabActivo("tasas");
              }}
            />
          ) : (
            <div style={{ padding: "40px", textAlign: "center", color: "var(--text-muted)" }}>
              Cargando valuación de patrimonio...
            </div>
          )
        )}

        {/* ======================================================== */}
        {/* TAB 3: TASAS DE CAMBIO E HISTORIAL                       */}
        {/* ======================================================== */}
        {tabActivo === "tasas" && (
          <VistaTasas
            tasas={tasasHistorial}
            monedas={monedasDisponibles}
            filtroPar={filtroParHistorial}
            onRegistrarTasa={handleGuardarTasa}
            onEliminarTasa={handleEliminarTasa}
            onCrearMoneda={handleCrearMoneda}
          />
        )}
      </main>

      <footer className="app-footer">
        <div style={{ display: "flex", alignItems: "center", gap: "8px", fontWeight: 500 }}>
          <span>Datos guardados localmente</span>
        </div>
        <div style={{ color: "var(--text-muted)", fontWeight: 500 }}>
          Sin nube. Sin conexión. Siempre tuyos.
        </div>
      </footer>

      {/* Modales de Entrega 2 y 3 */}
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
