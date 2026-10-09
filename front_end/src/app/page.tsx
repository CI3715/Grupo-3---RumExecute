"use client";

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface PingResult {
  estado: string;
  mensaje: string;
  version: string;
  timestamp: string;
  latencia_ms: number;
}

export default function HomePage() {
  const [data, setData] = useState<PingResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<"desconectado" | "conectando" | "conectado" | "error">("desconectado");

  const handlePing = async () => {
    setLoading(true);
    setError(null);
    setData(null);
    setStatus("conectando");

    try {
      let resultado: PingResult;

      // Detección de entorno: invocación nativa en Tauri vs modo navegador
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        resultado = await invoke<PingResult>("ping_servidor");
      } else {
        const tInicio = performance.now();
        const res = await fetch("http://localhost:8000/ping");
        const tFin = performance.now();

        if (!res.ok) {
          throw new Error(`El servidor respondió con estado HTTP ${res.status}`);
        }

        const json = await res.json();
        resultado = {
          estado: json.estado,
          mensaje: json.mensaje,
          version: json.version,
          timestamp: json.timestamp,
          latencia_ms: Math.round(tFin - tInicio),
        };
      }

      setData(resultado);
      setStatus("conectado");
    } catch (err: unknown) {
      const mensajeError =
        typeof err === "string"
          ? err
          : err instanceof Error
          ? err.message
          : "Error al comunicarse con el servidor backend.";
      setError(mensajeError);
      setStatus("error");
    } finally {
      setLoading(false);
    }
  };

  const badgeColor =
    status === "conectado"
      ? "#15803d"
      : status === "error"
      ? "#b91c1c"
      : status === "conectando"
      ? "#b45309"
      : "#6b7280";

  const badgeBg =
    status === "conectado"
      ? "#dcfce7"
      : status === "error"
      ? "#fee2e2"
      : status === "conectando"
      ? "#fef3c7"
      : "#f3f4f6";

  const statusTexto =
    status === "conectado"
      ? "Conectado"
      : status === "conectando"
      ? "Verificando..."
      : status === "error"
      ? "Error de Conexión"
      : "Desconectado";

  return (
    <main
      style={{
        minHeight: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: "#f9fafb",
        fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        padding: "2rem",
        boxSizing: "border-box",
      }}
    >
      <div
        style={{
          width: "100%",
          maxWidth: "600px",
          backgroundColor: "#ffffff",
          borderRadius: "12px",
          border: "1px solid #e5e7eb",
          boxShadow: "0 4px 6px -1px rgba(0, 0, 0, 0.05), 0 2px 4px -2px rgba(0, 0, 0, 0.05)",
          padding: "2.5rem",
        }}
      >
        <header style={{ marginBottom: "2rem", borderBottom: "1px solid #f3f4f6", paddingBottom: "1.5rem" }}>
          <h1 style={{ fontSize: "1.6rem", fontWeight: 700, color: "#111827", margin: 0 }}>
            Cuentas Claras
          </h1>
          <p style={{ color: "#6b7280", marginTop: "0.5rem", fontSize: "0.95rem" }}>
            Entrega 1: Ambiente Básico y Conexión Inicial (Full Stack Ping)
          </p>
        </header>

        <section style={{ marginBottom: "2rem" }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "1.5rem" }}>
            <span style={{ fontSize: "0.95rem", color: "#374151", fontWeight: 500 }}>
              Estado del Servidor:
            </span>
            <span
              style={{
                display: "inline-block",
                padding: "0.35rem 0.85rem",
                borderRadius: "9999px",
                fontSize: "0.85rem",
                fontWeight: 600,
                color: badgeColor,
                backgroundColor: badgeBg,
              }}
            >
              {statusTexto}
            </span>
          </div>

          <button
            onClick={handlePing}
            disabled={loading}
            style={{
              width: "100%",
              padding: "0.85rem 1.25rem",
              fontSize: "1rem",
              fontWeight: 600,
              color: "#ffffff",
              backgroundColor: loading ? "#9ca3af" : "#2563eb",
              border: "none",
              borderRadius: "8px",
              cursor: loading ? "not-allowed" : "pointer",
              transition: "background-color 0.15s ease",
            }}
          >
            {loading ? "Verificando conexión..." : "Verificar Conexión con el Servidor"}
          </button>
        </section>

        {error && (
          <div
            style={{
              padding: "1rem",
              borderRadius: "8px",
              backgroundColor: "#fef2f2",
              border: "1px solid #fecaca",
              color: "#991b1b",
              fontSize: "0.9rem",
              marginBottom: "1.5rem",
            }}
          >
            <strong>Fallo de conexión:</strong> {error}
          </div>
        )}

        {data && (
          <section
            style={{
              padding: "1.25rem",
              borderRadius: "8px",
              backgroundColor: "#f8fafc",
              border: "1px solid #e2e8f0",
            }}
          >
            <h3 style={{ margin: "0 0 1rem 0", fontSize: "1rem", color: "#0f172a", fontWeight: 600 }}>
              Métricas de Conexión Recibidas:
            </h3>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.75rem", fontSize: "0.9rem" }}>
              <div>
                <span style={{ color: "#64748b" }}>Estado:</span>{" "}
                <strong style={{ color: "#0f172a" }}>{data.estado}</strong>
              </div>
              <div>
                <span style={{ color: "#64748b" }}>Versión del Servidor:</span>{" "}
                <strong style={{ color: "#0f172a" }}>{data.version}</strong>
              </div>
              <div>
                <span style={{ color: "#64748b" }}>Latencia:</span>{" "}
                <strong style={{ color: "#16a34a" }}>{data.latencia_ms} ms</strong>
              </div>
              <div>
                <span style={{ color: "#64748b" }}>Timestamp:</span>{" "}
                <strong style={{ color: "#0f172a" }}>{data.timestamp}</strong>
              </div>
            </div>

            <div style={{ marginTop: "1rem", borderTop: "1px solid #e2e8f0", paddingTop: "0.75rem", fontSize: "0.9rem" }}>
              <span style={{ color: "#64748b" }}>Mensaje del Servidor:</span>
              <p style={{ margin: "0.35rem 0 0 0", color: "#0f172a", fontStyle: "italic" }}>
                "{data.mensaje}"
              </p>
            </div>
          </section>
        )}
      </div>
    </main>
  );
}
