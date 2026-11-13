"use client";

import { useState, useEffect, useCallback } from "react";
import Navbar from "../components/Navbar";
import SubtotalesGrid from "../components/SubtotalesGrid";
import ListaCuentas from "../components/ListaCuentas";
import ModalNuevaCuenta from "../components/ModalNuevaCuenta";
import ModalDiagnostico from "../components/ModalDiagnostico";
import DetalleCuenta from "../components/DetalleCuenta";
import VistaPatrimonio from "../components/VistaPatrimonio";
import VistaTasas from "../components/VistaTasas";
import VistaComunidad from "../components/VistaComunidad";
import ModalAuth from "../components/ModalAuth";

import { useCuentas } from "../hooks/useCuentas";
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
import {
  DatosUsuario,
  obtenerPerfil,
  obtenerUsuarioCacheado,
  cerrarSesion,
  verificarEstadoServidor,
} from "../services/authService";

export default function HomePage() {
  const [tabActivo, setTabActivo] = useState<"cuentas" | "patrimonio" | "tasas" | "comunidad">("cuentas");

  // Estado de Sesión y Autenticación (Entrega 5)
  const [usuario, setUsuario] = useState<DatosUsuario | null>(null);
  const [servidorOffline, setServidorOffline] = useState(false);
  const [avisoSesion, setAvisoSesion] = useState<string | null>(null);
  const [modalAuth, setModalAuth] = useState(false);
  const [modoAuth, setModoAuth] = useState<"login" | "registro">("login");

  // Hook desacoplado de cuentas, movimientos y categorías locales del dispositivo (INV-16 / RES-02)
  const {
    cuentas,
    subtotales,
    saldosIniciales,
    movimientosPorCuenta,
    categoriasGasto,
    categoriasIngreso,
    errorValidacion,
    setErrorValidacion,
    vista,
    setVista,
    cuentaActivaDetalle,
    abrirDetalleCuenta,
    setCuentaSeleccionadaId,
    cargarCuentasTauri,
    cargarCategoriasTauri,
    handleCrearCuenta,
    handleCrearCategoria,
    handleRegistrarMovimiento,
    handleCorregirMovimiento,
  } = useCuentas();

  // Estado de Tasas y Patrimonio Consolidado (Entrega 4)
  const [fechaConsultaPatrimonio, setFechaConsultaPatrimonio] = useState<string>(obtenerFechaHoyISO());
  const [monedaBaseActual, setMonedaBaseActual] = useState<string>("USD");
  const [monedasDisponibles, setMonedasDisponibles] = useState<MonedaItem[]>([]);
  const [tasasHistorial, setTasasHistorial] = useState<TasaCambioItem[]>([]);
  const [patrimonioData, setPatrimonioData] = useState<ResumenPatrimonioData | null>(null);
  const [filtroParHistorial, setFiltroParHistorial] = useState<{ origen?: string; destino?: string } | null>(null);

  // Modales adicionales
  const [modalNuevaCuenta, setModalNuevaCuenta] = useState(false);
  const [modalDiagnostico, setModalDiagnostico] = useState(false);

  // Carga inicial de catálogo de monedas y tasas
  const cargarMonedasYBase = useCallback(async () => {
    try {
      const lista = await listarMonedas();
      const base = await obtenerMonedaBase();
      setMonedasDisponibles(lista);
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
        const totalIngresos = movs.filter((m) => m.tipo === "ingreso").reduce((s, m) => s + m.monto_num, 0);
        const totalGastos = movs.filter((m) => m.tipo === "gasto").reduce((s, m) => s + m.monto_num, 0);
        const totalAjustes = movs.filter((m) => m.tipo === "ajuste").reduce((s, m) => s + m.monto_num, 0);
        return {
          id: c.id,
          nombre: c.nombre,
          tipo: c.tipo,
          moneda: c.moneda,
          saldo: saldoIni + totalIngresos - totalGastos + totalAjustes,
        };
      });

      const res = await consultarPatrimonio(fechaConsultaPatrimonio, cuentasSimples);
      setPatrimonioData(res);
    } catch (e) {
      console.error("Error al consultar patrimonio:", e);
    }
  }, [cuentas, saldosIniciales, movimientosPorCuenta, fechaConsultaPatrimonio]);

  // Carga inicial de datos locales del dispositivo
  useEffect(() => {
    cargarCuentasTauri();
    cargarCategoriasTauri();
    cargarMonedasYBase();
    cargarTasas();
  }, [cargarCuentasTauri, cargarCategoriasTauri, cargarMonedasYBase, cargarTasas]);

  // Verificación de conectividad y restauración inmediata de sesión al iniciar
  useEffect(() => {
    let cancelado = false;
    const cacheado = obtenerUsuarioCacheado();
    if (cacheado) {
      setUsuario(cacheado);
    }

    async function inicializar() {
      const activo = await verificarEstadoServidor();
      if (cancelado) return;
      setServidorOffline(!activo);

      try {
        const perfil = await obtenerPerfil();
        if (!cancelado) setUsuario(perfil);
      } catch (err: any) {
        if (!cancelado && err?.message?.includes("venció")) {
          setUsuario(null);
          setAvisoSesion("Tu sesión venció, inicia sesión de nuevo.");
        }
      }
    }
    inicializar();
    return () => { cancelado = true; };
  }, []);

  useEffect(() => {
    if (tabActivo === "patrimonio") {
      actualizarPatrimonio();
    }
  }, [tabActivo, fechaConsultaPatrimonio, monedaBaseActual, cuentas, actualizarPatrimonio]);

  // Handlers de tasas
  const handleCambiarMonedaBase = async (nuevaBase: string) => {
    try {
      await cambiarMonedaBase(nuevaBase);
      setMonedaBaseActual(nuevaBase);
      await actualizarPatrimonio();
    } catch (err: any) {
      setAvisoSesion(err?.message || "Error al cambiar moneda base.");
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
        onNavigateHome={() => {
          setTabActivo("cuentas");
          setVista("cuentas");
          setCuentaSeleccionadaId(null);
          cargarCuentasTauri();
          cargarCategoriasTauri();
        }}
        usuario={usuario}
        onOpenAuth={(modo) => {
          setModoAuth(modo);
          setModalAuth(true);
        }}
        onCerrarSesion={async () => {
          await cerrarSesion();
          setUsuario(null);
          setTabActivo("cuentas");
          setVista("cuentas");
          setCuentaSeleccionadaId(null);
          setFiltroParHistorial(null);
        }}
        servidorOffline={servidorOffline}
      />

      <main className="app-main">
        {/* Aviso no bloqueante de sesión vencida o servidor desconectado (RNF-17) */}
        {(avisoSesion || servidorOffline) && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "10px 16px",
              marginBottom: "18px",
              borderRadius: "10px",
              fontSize: "13px",
              fontWeight: 500,
              backgroundColor: avisoSesion ? "#FEF2F2" : "#FFFBEB",
              color: avisoSesion ? "#B91C1C" : "#92400E",
              border: `1px solid ${avisoSesion ? "#FCA5A5" : "#FDE68A"}`,
            }}
          >
            <span>
              {avisoSesion || "Sin conexión con el servidor. Tu libro personal sigue funcionando normalmente."}
            </span>
            {avisoSesion && (
              <button
                type="button"
                onClick={() => {
                  const eraVencimiento = avisoSesion.includes("venció");
                  setAvisoSesion(null);
                  if (eraVencimiento) {
                    setModoAuth("login");
                    setModalAuth(true);
                  }
                }}
                style={{
                  background: "none",
                  border: "none",
                  color: "#B91C1C",
                  fontWeight: 700,
                  cursor: "pointer",
                  textDecoration: "underline",
                }}
              >
                {avisoSesion.includes("venció") ? "Iniciar sesión" : "Cerrar"}
              </button>
            )}
          </div>
        )}

        {/* TAB 1: MIS CUENTAS Y MOVIMIENTOS */}
        {tabActivo === "cuentas" && (
          vista === "cuentas" ? (
            <>
              <div className="hero-section">
                <div>
                  <h1 className="hero-title">Tu dinero, en orden.</h1>
                  <p className="hero-subtitle">Todas tus cuentas en un solo lugar, sin mezclar monedas.</p>
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: "12px", flexWrap: "wrap" }}>
                  <div className="security-pill">
                    <span>Solo en este dispositivo</span>
                  </div>
                  <button
                    onClick={() => setModalNuevaCuenta(true)}
                    className="btn btn-primary"
                    style={{ padding: "9px 18px", fontSize: "13px" }}
                  >
                    <span style={{ fontSize: "15px", fontWeight: 700 }}>+</span>
                    <span>Nueva cuenta</span>
                  </button>
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
                onRegistrarMovimiento={async (m) => {
                  const ok = await handleRegistrarMovimiento(m);
                  if (ok && cuentaActivaDetalle) abrirDetalleCuenta(cuentaActivaDetalle.id);
                  return ok;
                }}
                onCorregirMovimiento={handleCorregirMovimiento}
              />
            )
          )
        )}

        {/* TAB 2: PATRIMONIO CONSOLIDADO */}
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
                if (origen && destino) setFiltroParHistorial({ origen, destino });
                setTabActivo("tasas");
              }}
              onOpenCrearMoneda={() => setTabActivo("tasas")}
            />
          ) : (
            <div style={{ padding: "40px", textAlign: "center", color: "var(--text-muted)" }}>
              Cargando valuación de patrimonio...
            </div>
          )
        )}

        {/* TAB 3: TASAS DE CAMBIO E HISTORIAL */}
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

        {/* TAB 4: GRUPOS, AMIGOS Y GASTOS COMPARTIDOS (ENTREGA 6) */}
        {tabActivo === "comunidad" && (
          <VistaComunidad
            key={usuario?.id || "anon"}
            usuario={usuario}
            monedas={monedasDisponibles}
            onSolicitarLogin={() => {
              setModoAuth("login");
              setModalAuth(true);
            }}
          />
        )}
      </main>

      <footer className="app-footer">
        <div style={{ display: "flex", alignItems: "center", gap: "8px", fontWeight: 500 }}>
          <span>Datos guardados localmente</span>
          <span style={{ color: "var(--border-subtle)" }}>•</span>
          <span style={{ color: "var(--text-muted)" }}>Sin nube. Siempre tuyos.</span>
        </div>
        <button
          type="button"
          onClick={() => setModalDiagnostico(true)}
          className="btn btn-secondary"
          style={{ padding: "4px 12px", fontSize: "12px" }}
          title="Verificar conexión con el servidor"
        >
          Diagnóstico de red
        </button>
      </footer>

      {/* Modales */}
      <ModalNuevaCuenta
        isOpen={modalNuevaCuenta}
        onClose={() => {
          setModalNuevaCuenta(false);
          setErrorValidacion(null);
        }}
        onSubmit={async (e) => {
          const ok = await handleCrearCuenta(e);
          if (ok) setModalNuevaCuenta(false);
          return ok;
        }}
        errorExterno={errorValidacion}
      />

      <ModalDiagnostico
        isOpen={modalDiagnostico}
        onClose={() => setModalDiagnostico(false)}
      />

      <ModalAuth
        isOpen={modalAuth}
        modoInicial={modoAuth}
        onClose={() => setModalAuth(false)}
        onLoginExitoso={(u) => {
          setUsuario(u);
          setTabActivo("cuentas");
          setVista("cuentas");
          setCuentaSeleccionadaId(null);
          setFiltroParHistorial(null);
          setAvisoSesion(null);
          verificarEstadoServidor().then((activo) => setServidorOffline(!activo));
        }}
      />
    </div>
  );
}
