"use client";

import React, { useState } from "react";
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

            // Invocación nativa Tauri vs navegador
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
        <div style={styles.overlay} onClick={onClose}>
            <div style={styles.modalCard} onClick={(e) => e.stopPropagation()}>
                {/* Header */}
                <div style={styles.header}>
                    <div>
                        <h3 style={styles.title}>Diagnóstico de Red (Entrega 1)</h3>
                        <p style={styles.subtitle}>Verificador de enlace con el servidor FastAPI</p>
                    </div>
                    <button onClick={onClose} style={styles.btnClose}>✕</button>
                </div>

                {/* Contenido */}
                <div style={styles.body}>
                    {/* Badge de estado */}
                    <div style={styles.statusRow}>
                        <span style={{ fontSize: "12px", color: "var(--text-secondary)", fontWeight: 600 }}>Estado:</span>
                        <span style={{ ...styles.badge, backgroundColor: badgeBg, color: badgeColor }}>
                            ● {statusTexto}
                        </span>
                    </div>

                    {/* Botón de Ping */}
                    <button
                        onClick={handlePing}
                        disabled={loading}
                        style={styles.btnPing}
                    >
                        {loading ? "Verificando enlace..." : "📡 Probar Conexión (Ping)"}
                    </button>

                    {/* Resultado exitoso */}
                    {data && (
                        <div style={styles.resultBox}>
                            <div style={styles.resultItem}>
                                <strong>Servidor:</strong> <span>{data.mensaje}</span>
                            </div>
                            <div style={styles.resultItem}>
                                <strong>Versión:</strong> <span>v{data.version}</span>
                            </div>
                            <div style={styles.resultItem}>
                                <strong>Latencia:</strong> <span>{data.latencia_ms} ms</span>
                            </div>
                            <div style={styles.resultItem}>
                                <strong>Timestamp:</strong> <span style={{ fontSize: "11px" }}>{data.timestamp}</span>
                            </div>
                        </div>
                    )}

                    {/* Error si el servidor está apagado */}
                    {error && (
                        <div style={styles.errorBox}>
                            <div style={{ fontWeight: 600, marginBottom: "4px" }}>⚠️ Servidor Inaccesible</div>
                            <div style={{ fontSize: "12px" }}>{error}</div>
                            <div style={{ fontSize: "11px", marginTop: "6px", opacity: 0.85 }}>
                                (Recuerda: Cuentas Claras funciona 100% offline con el servidor apagado).
                            </div>
                        </div>
                    )}
                </div>

                {/* Footer */}
                <div style={styles.footer}>
                    <button onClick={onClose} style={styles.btnCloseFooter}>Cerrar</button>
                </div>
            </div>
        </div>
    );
}

const styles: Record<string, React.CSSProperties> = {
    overlay: {
        position: "fixed",
        inset: 0,
        backgroundColor: "rgba(15, 23, 42, 0.35)",
        backdropFilter: "blur(4px)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 50,
        padding: "16px",
    },
    modalCard: {
        width: "100%",
        maxWidth: "460px",
        backgroundColor: "var(--surface-card)",
        borderRadius: "16px",
        border: "1px solid var(--border-subtle)",
        boxShadow: "0 24px 64px 0 rgba(15, 23, 42, 0.15)",
        overflow: "hidden",
    },
    header: {
        padding: "24px 24px 16px 24px",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "space-between",
        borderBottom: "1px solid var(--border-subtle)",
    },
    title: {
        fontSize: "18px",
        fontWeight: 700,
        color: "var(--text-primary)",
    },
    subtitle: {
        fontSize: "12px",
        color: "var(--text-secondary)",
        marginTop: "2px",
    },
    btnClose: {
        width: "28px",
        height: "28px",
        borderRadius: "6px",
        backgroundColor: "var(--surface-hover)",
        color: "var(--text-secondary)",
        fontSize: "12px",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
    },
    body: {
        padding: "20px 24px",
        display: "flex",
        flexDirection: "column",
        gap: "16px",
    },
    statusRow: {
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
    },
    badge: {
        fontSize: "12px",
        fontWeight: 600,
        padding: "4px 10px",
        borderRadius: "9999px",
    },
    btnPing: {
        height: "44px",
        borderRadius: "8px",
        backgroundColor: "var(--text-primary)",
        color: "#FFFFFF",
        fontSize: "13px",
        fontWeight: 600,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        gap: "8px",
    },
    resultBox: {
        backgroundColor: "var(--bg-app)",
        border: "1px solid var(--border-subtle)",
        borderRadius: "10px",
        padding: "14px",
        display: "flex",
        flexDirection: "column",
        gap: "6px",
        fontSize: "13px",
    },
    resultItem: {
        display: "flex",
        justifyContent: "space-between",
    },
    errorBox: {
        backgroundColor: "#FEE2E2",
        border: "1px solid #FECACA",
        color: "#B91C1C",
        borderRadius: "10px",
        padding: "12px 14px",
    },
    footer: {
        padding: "12px 24px",
        backgroundColor: "var(--bg-app)",
        borderTop: "1px solid var(--border-subtle)",
        display: "flex",
        justifyContent: "flex-end",
    },
    btnCloseFooter: {
        height: "36px",
        padding: "0 16px",
        borderRadius: "6px",
        border: "1px solid var(--border-subtle)",
        backgroundColor: "var(--surface-card)",
        color: "var(--text-primary)",
        fontSize: "13px",
        fontWeight: 600,
    },
};