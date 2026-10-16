"use client";

import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Navbar from "../components/Navbar";
import SubtotalesGrid, { SubtotalMoneda } from "../components/SubtotalesGrid";
import ListaCuentas, { CuentaItem } from "../components/ListaCuentas";
import ModalNuevaCuenta, { EntradaCuentaForm, ErrorValidacion } from "../components/ModalNuevaCuenta";
import ModalDiagnostico from "../components/ModalDiagnostico";

import { formatearMonto } from "../utils/formato";

interface CuentaRust {
  id: string;
  nombre: string;
  tipo: string;
  moneda: string;
  saldo_inicial: {
    cantidad: string | number;
    moneda: string;
  };
}

interface SubtotalRust {
  cantidad: string | number;
  moneda: string;
}

interface ResumenCuentasRust {
  cuentas: CuentaRust[];
  subtotales_por_moneda: SubtotalRust[];
}

export default function HomePage() {
  const [cuentas, setCuentas] = useState<CuentaItem[]>([]);
  const [subtotales, setSubtotales] = useState<SubtotalMoneda[]>([]);
  const [modalNuevaCuenta, setModalNuevaCuenta] = useState(false);
  const [modalDiagnostico, setModalDiagnostico] = useState(false);
  const [errorValidacion, setErrorValidacion] = useState<ErrorValidacion | null>(null);

  // 1. Cargar cuentas reales desde Rust SQLite al iniciar la aplicación
  const cargarCuentas = async () => {
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      try {
        const respuesta = await invoke<ResumenCuentasRust>("listar_cuentas");
        if (respuesta) {
          const cuentasMapeadas: CuentaItem[] = (respuesta.cuentas || []).map((c) => ({
            id: c.id,
            nombre: c.nombre,
            tipo: c.tipo,
            moneda: c.moneda,
            saldo_formateado: formatearMonto(c.saldo_inicial?.cantidad ?? "0", c.moneda),
          }));

          const subtotalesMapeados: SubtotalMoneda[] = (respuesta.subtotales_por_moneda || []).map((s) => {
            const numCuentas = (respuesta.cuentas || []).filter((c) => c.moneda === s.moneda).length;
            return {
              moneda: s.moneda,
              subtotal_formateado: formatearMonto(s.cantidad, s.moneda),
              cantidad_cuentas: numCuentas,
            };
          });

          setCuentas(cuentasMapeadas);
          setSubtotales(subtotalesMapeados);
        }
      } catch (err) {
        console.warn("Error al cargar cuentas desde Tauri IPC:", err);
      }
    }
  };

  useEffect(() => {
    cargarCuentas();
  }, []);

  // 2. Manejo de creación de una cuenta
  const handleCrearCuenta = async (entrada: EntradaCuentaForm): Promise<boolean> => {
    setErrorValidacion(null);

    // --- Flujo A: En entorno Tauri Nativo (Conexión directa con Rust y SQLite) ---
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      try {
        await invoke("crear_cuenta", {
          entrada: {
            nombre: entrada.nombre,
            tipo: entrada.tipo,
            moneda: entrada.moneda,
            saldo_inicial: entrada.saldo_inicial,
          },
        });
        await cargarCuentas();
        return true;
      } catch (err: any) {
        // En Rust: ErrorComando::Validacion(ErrorDominio { campo, mensaje })
        if (err && typeof err === "object" && err.campo && err.mensaje) {
          setErrorValidacion({
            campo: err.campo,
            mensaje: err.mensaje,
          });
        } else if (typeof err === "string") {
          setErrorValidacion({
            campo: "general",
            mensaje: err,
          });
        } else {
          setErrorValidacion({
            campo: "general",
            mensaje: "Error inesperado al registrar la cuenta",
          });
        }
        return false;
      }
    }

    // --- Flujo B: En Navegador Web (Modo prueba funcional previa a empaquetado) ---
    // Validación 1: Nombre vacío
    if (!entrada.nombre.trim()) {
      setErrorValidacion({
        campo: "nombre",
        mensaje: "El nombre de la cuenta no puede estar vacío.",
      });
      return false;
    }

    // Validación 2: Nombre duplicado (case-insensitive)
    if (cuentas.some((c) => c.nombre.trim().toLowerCase() === entrada.nombre.trim().toLowerCase())) {
      setErrorValidacion({
        campo: "nombre",
        mensaje: `Ya existe una cuenta con el nombre '${entrada.nombre}'. Elige un nombre distinto.`,
      });
      return false;
    }

    // Validación 3: Saldo negativo o caracteres inválidos
    const saldoNormalizado = entrada.saldo_inicial.replace(",", ".");
    const numeroSaldo = Number(saldoNormalizado);
    if (isNaN(numeroSaldo) || numeroSaldo < 0) {
      setErrorValidacion({
        campo: "saldo_inicial",
        mensaje: "El saldo inicial no puede ser negativo ni contener letras.",
      });
      return false;
    }

    // Validación 4: Máximo 2 decimales
    const partes = saldoNormalizado.split(".");
    if (partes.length > 1 && partes[1].length > 2) {
      setErrorValidacion({
        campo: "saldo_inicial",
        mensaje: "El saldo inicial no puede tener más de 2 decimales.",
      });
      return false;
    }

    // Crear la nueva cuenta formateada
    const saldoFormateado = formatearMonto(numeroSaldo, entrada.moneda);
    const nuevaCuenta: CuentaItem & { _saldo_num?: number } = {
      id: `cta-${Date.now()}`,
      nombre: entrada.nombre.trim(),
      tipo: entrada.tipo,
      moneda: entrada.moneda,
      saldo_formateado: saldoFormateado,
      _saldo_num: numeroSaldo,
    };

    const nuevasCuentas = [...cuentas, nuevaCuenta];
    setCuentas(nuevasCuentas);

    // INV-02: Calcular subtotales por moneda sumando TODAS las cuentas de la misma moneda
    const monedasPresentes = Array.from(new Set(nuevasCuentas.map((c) => c.moneda)));
    const nuevosSubtotales: SubtotalMoneda[] = monedasPresentes.map((codMoneda) => {
      const cuentasDeEstaMoneda = nuevasCuentas.filter((c) => c.moneda === codMoneda);
      const totalMoneda = cuentasDeEstaMoneda.reduce((acum, c) => {
        const val = (c as any)._saldo_num !== undefined
          ? (c as any)._saldo_num
          : Number((c.saldo_formateado || "0").replace(/[^0-9,-]/g, "").replace(",", ".")) || 0;
        return acum + val;
      }, 0);

      return {
        moneda: codMoneda,
        subtotal_formateado: formatearMonto(totalMoneda, codMoneda),
        cantidad_cuentas: cuentasDeEstaMoneda.length,
      };
    });

    setSubtotales(nuevosSubtotales);
    return true;
  };

  return (
    <div style={styles.mainWrapper}>
      {/* 1. Barra de Navegación Oficial */}
      <Navbar
        onOpenNuevaCuenta={() => setModalNuevaCuenta(true)}
        onOpenDiagnostico={() => setModalDiagnostico(true)}
      />

      {/* 2. Contenedor Principal */}
      <main style={styles.content}>
        {/* Cabecera / Saludo */}
        <div style={styles.heroSection}>
          <div>
            <h1 style={styles.heroTitle}>Tu dinero, en orden.</h1>
            <p style={styles.heroSubtitle}>
              Todas tus cuentas en un solo lugar, sin mezclar monedas.
            </p>
          </div>
          <div style={styles.securityBadge}>
            <span style={{ fontSize: "14px" }}>🛡️</span>
            <span>Solo en este dispositivo</span>
          </div>
        </div>

        {/* 3. Subtotales por Moneda (Solo aparecen si hay cuentas creadas) */}
        <SubtotalesGrid subtotales={subtotales} />

        {/* 4. Tabla de Cuentas o Estado Vacío */}
        <ListaCuentas
          cuentas={cuentas}
          onCrearPrimeraCuenta={() => setModalNuevaCuenta(true)}
        />
      </main>

      {/* 5. Pie de página Local-First */}
      <footer style={styles.footer}>
        <div style={styles.footerLeft}>
          <span style={{ fontSize: "13px" }}>💾</span>
          <span>Datos guardados localmente</span>
        </div>
        <div style={styles.footerRight}>Sin nube. Sin conexión. Siempre tuyos.</div>
      </footer>

      {/* 6. Modal Crear Nueva Cuenta */}
      <ModalNuevaCuenta
        isOpen={modalNuevaCuenta}
        onClose={() => {
          setModalNuevaCuenta(false);
          setErrorValidacion(null);
        }}
        onSubmit={handleCrearCuenta}
        errorExterno={errorValidacion}
      />

      {/* 7. Modal Diagnóstico de Red */}
      <ModalDiagnostico
        isOpen={modalDiagnostico}
        onClose={() => setModalDiagnostico(false)}
      />
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  mainWrapper: {
    minHeight: "100vh",
    display: "flex",
    flexDirection: "column",
    backgroundColor: "var(--bg-app)",
  },
  content: {
    flex: 1,
    width: "100%",
    maxWidth: "1152px",
    margin: "0 auto",
    padding: "36px 24px 48px 24px",
  },
  heroSection: {
    display: "flex",
    alignItems: "flex-end",
    justifyContent: "space-between",
    marginBottom: "32px",
    flexWrap: "wrap",
    gap: "16px",
  },
  heroTitle: {
    fontSize: "28px",
    fontWeight: 700,
    color: "var(--text-primary)",
    letterSpacing: "-0.6px",
    lineHeight: 1.15,
  },
  heroSubtitle: {
    fontSize: "14px",
    color: "var(--text-secondary)",
    marginTop: "6px",
  },
  securityBadge: {
    display: "flex",
    alignItems: "center",
    gap: "6px",
    fontSize: "12px",
    fontWeight: 600,
    color: "var(--text-secondary)",
    backgroundColor: "var(--surface-card)",
    border: "1px solid var(--border-subtle)",
    padding: "6px 12px",
    borderRadius: "9999px",
  },
  footer: {
    height: "64px",
    padding: "0 64px",
    borderTop: "1px solid var(--border-subtle)",
    backgroundColor: "var(--surface-card)",
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    fontSize: "11.5px",
    color: "var(--text-secondary)",
  },
  footerLeft: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    fontWeight: 500,
  },
  footerRight: {
    fontWeight: 500,
    color: "var(--text-muted)",
  },
};