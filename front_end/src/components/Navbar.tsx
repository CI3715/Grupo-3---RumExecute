"use client";

import React from "react";

interface NavbarProps {
    onOpenNuevaCuenta: () => void;
    onOpenDiagnostico: () => void;
}

export default function Navbar({ onOpenNuevaCuenta, onOpenDiagnostico }: NavbarProps) {
    return (
        <header style={styles.header}>
            {/* Lado Izquierdo: Logotipo Oficial */}
            <div style={styles.brandContainer}>
                {/* Isotipo SVG de La Balanza */}
                <svg viewBox="0 0 512 512" width="42" height="42" style={{ flexShrink: 0 }}>
                    <defs>
                        <linearGradient id="nav-emerald" x1="0%" y1="0%" x2="100%" y2="100%">
                            <stop offset="0%" stopColor="#059669" />
                            <stop offset="100%" stopColor="#022c22" />
                        </linearGradient>
                        <linearGradient id="nav-mint" x1="0%" y1="100%" x2="100%" y2="0%">
                            <stop offset="0%" stopColor="#34D399" />
                            <stop offset="100%" stopColor="#059669" />
                        </linearGradient>
                    </defs>
                    <g transform="translate(256, 256) scale(0.95)">
                        <path
                            d="M 50,-150 A 160,160 0 1,0 50,150 L 50,86 A 96,96 0 1,1 50,-86 Z"
                            fill="url(#nav-emerald)"
                        />
                        <path
                            d="M -20,-110 A 120,120 0 0,1 110,-20 L 165,-45 A 180,180 0 0,0 -45,-165 Z"
                            fill="url(#nav-mint)"
                        />
                        <circle cx="85" cy="55" r="30" fill="#059669" />
                        <circle cx="85" cy="55" r="13" fill="#FFFFFF" />
                    </g>
                </svg>

                {/* Tipografía de Marca */}
                <div style={styles.brandText}>
                    <div style={styles.titleRow}>
                        <span style={{ fontWeight: 800, color: "var(--text-primary)" }}>Cuentas</span>
                        <span style={{ fontWeight: 800, color: "var(--emerald-primary)", marginLeft: "6px" }}>
                            Claras
                        </span>
                    </div>
                    <div style={styles.subtitleRow}>
                        <span style={styles.subtitleBadge}>BILLETERA LOCAL OFFLINE</span>
                        <span style={styles.privacyDot} />
                        <span style={styles.privacyText}>100% Privado</span>
                    </div>
                </div>
            </div>

            {/* Lado Derecho: Botones de Acción */}
            <div style={styles.actionsContainer}>
                {/* Botón Secundario: Diagnóstico de Red (Entrega 1) */}
                <button
                    onClick={onOpenDiagnostico}
                    style={styles.btnSecondary}
                    onMouseOver={(e) => (e.currentTarget.style.backgroundColor = "var(--surface-hover)")}
                    onMouseOut={(e) => (e.currentTarget.style.backgroundColor = "var(--surface-card)")}
                >
                    <span style={{ fontSize: "14px" }}>📶</span>
                    <span>Diagnóstico Servidor</span>
                </button>

                {/* Botón Primario: Nueva Cuenta (Entrega 2) */}
                <button
                    onClick={onOpenNuevaCuenta}
                    style={styles.btnPrimary}
                    onMouseOver={(e) => (e.currentTarget.style.backgroundColor = "var(--emerald-hover)")}
                    onMouseOut={(e) => (e.currentTarget.style.backgroundColor = "var(--emerald-primary)")}
                >
                    <span style={{ fontSize: "16px", fontWeight: 700 }}>+</span>
                    <span>Nueva cuenta</span>
                </button>
            </div>
        </header>
    );
}

const styles: Record<string, React.CSSProperties> = {
    header: {
        height: "104px",
        padding: "0 64px",
        backgroundColor: "var(--surface-card)",
        borderBottom: "1px solid var(--border-subtle)",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        position: "sticky",
        top: 0,
        zIndex: 10,
    },
    brandContainer: {
        display: "flex",
        alignItems: "center",
        gap: "14px",
    },
    brandText: {
        display: "flex",
        flexDirection: "column",
        gap: "2px",
    },
    titleRow: {
        fontSize: "22px",
        letterSpacing: "-0.5px",
        lineHeight: 1.1,
    },
    subtitleRow: {
        display: "flex",
        alignItems: "center",
        gap: "8px",
    },
    subtitleBadge: {
        fontSize: "10px",
        fontWeight: 700,
        color: "var(--text-secondary)",
        letterSpacing: "1.2px",
    },
    privacyDot: {
        width: "4px",
        height: "4px",
        borderRadius: "50%",
        backgroundColor: "var(--emerald-primary)",
    },
    privacyText: {
        fontSize: "10.5px",
        fontWeight: 600,
        color: "var(--text-muted)",
    },
    actionsContainer: {
        display: "flex",
        alignItems: "center",
        gap: "12px",
    },
    btnSecondary: {
        height: "42px",
        padding: "0 16px",
        borderRadius: "8px",
        border: "1px solid var(--border-subtle)",
        backgroundColor: "var(--surface-card)",
        color: "var(--text-primary)",
        fontSize: "13px",
        fontWeight: 600,
        display: "flex",
        alignItems: "center",
        gap: "8px",
    },
    btnPrimary: {
        height: "42px",
        padding: "0 18px",
        borderRadius: "8px",
        backgroundColor: "var(--emerald-primary)",
        color: "#FFFFFF",
        fontSize: "13px",
        fontWeight: 600,
        display: "flex",
        alignItems: "center",
        gap: "8px",
        boxShadow: "0 1px 2px rgba(5, 150, 105, 0.2)",
    },
};