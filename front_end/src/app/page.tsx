"use client";

import { useState, useEffect, useMemo } from "react";
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

  const [categoriasGasto, setCategoriasGasto] = useState<string[]>(CATEGORIAS_GASTO);
  const [categoriasIngreso, setCategoriasIngreso] = useState<string[]>(CATEGORIAS_INGRESO);

  const [modalNuevaCuenta, setModalNuevaCuenta] = useState(false);
  const [modalDiagnostico, setModalDiagnostico] = useState(false);
  const [errorValidacion, setErrorValidacion] = useState<ErrorValidacion | null>(null);

  const cargarCuentasTauri = async () => {
    if (!enTauri()) return;
    try {
      const res = await invoke<ResumenCuentasRust>("listar_cuentas");
      if (!res?.cuentas) return;

      const mapaSaldos: Record<string, number> = {};
      const mapaMovs: Record<string, MovimientoItem[]> = {};

      // Consultar el detalle proyectado de cada cuenta (saldo_actual y movimientos de SQLite)
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

      // Calcular los subtotales usando los saldos actuales reales proyectados
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
    } catch (err) {
      console.warn("Error al cargar cuentas desde Tauri IPC:", err);
    }
  };

  const cargarCategoriasTauri = async () => {
    if (!enTauri()) return;
    try {
      const cats = await invoke<
        Array<{ id: string; nombre: string; tipo: "gasto" | "ingreso" }>
      >("listar_categorias");
      if (cats && Array.isArray(cats)) {
        const gastos = cats
          .filter((c) => c.tipo === "gasto")
          .map((c) => c.nombre);
        const ingresos = cats
          .filter((c) => c.tipo === "ingreso")
          .map((c) => c.nombre);

        setCategoriasGasto(Array.from(new Set([...CATEGORIAS_GASTO, ...gastos])));
        setCategoriasIngreso(Array.from(new Set([...CATEGORIAS_INGRESO, ...ingresos])));
      }
    } catch (err) {
      console.warn("Error al cargar categorías desde Tauri IPC:", err);
    }
  };

  useEffect(() => {
    cargarCuentasTauri();
    cargarCategoriasTauri();
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
            (typeof err === "string" ? err : "Error al crear la categoría."),
        };
        setErrorValidacion(errorObj);
        throw errorObj;
      }
    }

    // Modo fallback sin Tauri
    const listaActual =
      entrada.tipo === "gasto" ? categoriasGasto : categoriasIngreso;
    const yaExiste = listaActual.some(
      (c) => c.trim().toLowerCase() === nombreLimpio.toLowerCase()
    );
    if (yaExiste) {
      const err = {
        campo: "categoria",
        mensaje: `Ya existe una categoría de ${entrada.tipo} con el nombre '${nombreLimpio}'. Elige un nombre distinto.`,
      };
      setErrorValidacion(err);
      throw err;
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

    // Si el usuario indicó crear una nueva categoría, primero la creamos
    if (entrada.esNuevaCategoria) {
      try {
        await handleCrearCategoria({
          nombre: entrada.categoria.trim(),
          tipo: entrada.tipo,
        });
      } catch (errCat: any) {
        // Si el backend o fallback reporta que ya existe, no es un error bloqueante para el movimiento
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
        await cargarCategoriasTauri();
        await abrirDetalleCuenta(entrada.id_cuenta);
        return true;
      } catch (err: any) {
        const errorObj = {
          campo: err?.campo || "categoria",
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
    const movsBrutos = movimientosPorCuenta[cuenta.id] || [];

    // Sanitización exhaustiva para asegurar tipos primitivos a prueba de fallos de React
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
        onOpenNuevaCuenta={() => setModalNuevaCuenta(true)}
        onOpenDiagnostico={() => setModalDiagnostico(true)}
        onNavigateHome={() => {
          setVista("cuentas");
          setCuentaSeleccionadaId(null);
          cargarCuentasTauri();
          cargarCategoriasTauri();
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
