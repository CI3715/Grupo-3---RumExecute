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

interface ModalDiagnosticoProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function ModalDiagnostico({ isOpen, onClose }: ModalDiagnosticoProps) {
  const [data, setData] = useState<PingResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<"desconectado" | "conectando" | "conectado" | "error">("desconectado");

  if (!isOpen) return null;

  const handlePing = async () => {
    setLoading(true);
    setError(null);
    setData(null);
    setStatus("conectando");

    try {
      let resultado: PingResult;
      if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
        resultado = await invoke<PingResult>("ping_servidor");
      } else {
        const tInicio = performance.now();
        const res = await fetch("http://localhost:8000/ping");
        const tFin = performance.now();
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const json = await res.json();
        resultado = { ...json, latencia_ms: Math.round(tFin - tInicio) };
      }
      setData(resultado);
      setStatus("conectado");
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus("error");
    } finally {
      setLoading(false);
    }
  };

  const statusConfig = {
    conectado: { color: "#15803d", bg: "#dcfce7", texto: "Conectado" },
    conectando: { color: "#b45309", bg: "#fef3c7", texto: "Verificando..." },
    error: { color: "#b91c1c", bg: "#fee2e2", texto: "Error de Conexión" },
    desconectado: { color: "#6b7280", bg: "#f3f4f6", texto: "Desconectado" },
  }[status];

  return (
    <div
      className="modal-overlay"
      onMouseDown={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className="modal-card" style={{ maxWidth: "460px" }}>
        <div className="modal-header">
          <div>
            <h3 className="modal-title">Diagnóstico de Red</h3>
            <p className="modal-subtitle">Verificador de enlace local con el servidor</p>
          </div>
          <button onClick={onClose} className="btn-close">✕</button>
        </div>

        <div className="modal-body">
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <span style={{ fontSize: "12px", color: "var(--text-secondary)", fontWeight: 600 }}>Estado:</span>
            <span className="badge-pill" style={{ backgroundColor: statusConfig.bg, color: statusConfig.color }}>
              ● {statusConfig.texto}
            </span>
          </div>

          <button onClick={handlePing} disabled={loading} className="btn btn-primary" style={{ width: "100%" }}>
            {loading ? "Verificando enlace..." : "📡 Probar Conexión (Ping)"}
          </button>

          {data && (
            <div className="equation-bar" style={{ flexDirection: "column", alignItems: "stretch", fontSize: "12px" }}>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <span style={{ color: "var(--text-secondary)" }}>Estado:</span>
                <strong>{data.estado}</strong>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <span style={{ color: "var(--text-secondary)" }}>Mensaje:</span>
                <strong>{data.mensaje}</strong>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <span style={{ color: "var(--text-secondary)" }}>Versión:</span>
                <strong>{data.version}</strong>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <span style={{ color: "var(--text-secondary)" }}>Latencia:</span>
                <strong style={{ color: "var(--emerald-primary)" }}>{data.latencia_ms} ms</strong>
              </div>
            </div>
          )}

          {error && (
            <div className="alert-box">
              <strong>No se pudo conectar:</strong> {error}. Asegúrate de tener el servidor encendido en{" "}
              <code>http://127.0.0.1:8000</code>.
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button onClick={onClose} className="btn btn-secondary">Cerrar</button>
        </div>
      </div>
    </div>
  );
}
