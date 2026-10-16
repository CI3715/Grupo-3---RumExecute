"use client";

import React from "react";

// Estructura de cada cuenta que entrega el caso de uso ListarCuentas
export interface CuentaItem {
    id: string;
    nombre: string;
    tipo: string;              // "efectivo" | "banco" | "billetera_digital"
    tipo_etiqueta?: string;    // "Efectivo" | "Banco" | "Billetera digital"
    moneda: string;            // "VES" | "USD" | "EUR" | "USDT"
    saldo_formateado: string;  // "Bs. 1.250,50" (Calculado por Rust)
}

interface ListaCuentasProps {
    cuentas: CuentaItem[];
    onCrearPrimeraCuenta: () => void;
}

// Configuración de estilo y badges según el tipo de cuenta
const CONFIG_TIPOS: Record<string, { etiqueta: string; icono: string; badgeBg: string; badgeText: string }> = {
    efectivo: {
        etiqueta: "Efectivo",
        icono: "💵",
        badgeBg: "var(--badge-cash-bg)",
        badgeText: "var(--badge-cash-text)",
    },
    banco: {
        etiqueta: "Banco",
        icono: "🏦",
        badgeBg: "var(--badge-bank-bg)",
        badgeText: "var(--badge-bank-text)",
    },
    billetera_digital: {
        etiqueta: "Billetera digital",
        icono: "📱",
        badgeBg: "var(--badge-wallet-bg)",
        badgeText: "var(--badge-wallet-text)",
    },
};

export default function ListaCuentas({ cuentas, onCrearPrimeraCuenta }: ListaCuentasProps) {
    // ESTADO VACÍO (Frame 3:601 de Figma)
    if (cuentas.length === 0) {
        return (
            <section style={styles.container}>
                <div style={styles.emptyCard}>
                    <div style={styles.emptyIconCircle}>👛</div>
                    <h3 style={styles.emptyTitle}>Aún no tienes cuentas registradas</h3>
                    <p style={styles.emptySubtitle}>
                        Registra tu efectivo, cuentas bancarias o billeteras digitales para llevar tus cuentas
                        claras de forma 100% local, sin servidores y con total privacidad.
                    </p>
                    <button onClick={onCrearPrimeraCuenta} style={styles.btnPrimary}>
                        + Crear mi primera cuenta
                    </button>
                </div>
            </section>
        );
    }

    // TABLA DE CUENTAS (Nodo 3:553 de Figma)
    return (
        <section style={styles.container}>
            {/* Encabezado de Sección */}
            <div style={styles.sectionHeader}>
                <span style={styles.sectionTitle}>TUS CUENTAS ({cuentas.length})</span>
                <span style={styles.sectionLegend}>Saldos por cuenta</span>
            </div>

            {/* Contenedor Card de la Tabla */}
            <div style={styles.tableCard}>
                {/* Fila de Títulos de Columna */}
                <div style={styles.columnsHeader}>
                    <span style={{ flex: 1.2 }}>CUENTA</span>
                    <span style={{ width: "160px" }}>TIPO</span>
                    <span style={{ width: "180px", textAlign: "right" }}>SALDO DISPONIBLE</span>
                    <span style={{ width: "24px" }} />
                </div>

                {/* Filas Dinámicas */}
                {cuentas.map((cuenta) => {
                    const config = CONFIG_TIPOS[cuenta.tipo.toLowerCase()] || {
                        etiqueta: cuenta.tipo,
                        icono: "💰",
                        badgeBg: "#F1F5F9",
                        badgeText: "#475569",
                    };

                    return (
                        <div
                            key={cuenta.id}
                            style={styles.accountRow}
                            onMouseOver={(e) => (e.currentTarget.style.backgroundColor = "var(--surface-hover)")}
                            onMouseOut={(e) => (e.currentTarget.style.backgroundColor = "var(--surface-card)")}
                        >
                            {/* Columna 1: Ícono + Nombre + Moneda Fija */}
                            <div style={styles.accountColPrimary}>
                                <div style={styles.iconBox}>{config.icono}</div>
                                <div style={styles.nameBlock}>
                                    <div style={styles.accountName}>{cuenta.nombre}</div>
                                    <div style={styles.currencySubtext}>{cuenta.moneda} · Moneda fija</div>
                                </div>
                            </div>

                            {/* Columna 2: Badge de Tipo */}
                            <div style={{ width: "160px" }}>
                                <span
                                    style={{
                                        ...styles.typeBadge,
                                        backgroundColor: config.badgeBg,
                                        color: config.badgeText,
                                    }}
                                >
                                    {config.etiqueta}
                                </span>
                            </div>

                            {/* Columna 3: Saldo Formateado desde Rust */}
                            <div style={styles.balanceText}>{cuenta.saldo_formateado}</div>

                            {/* Columna 4: Chevron sutil */}
                            <div style={styles.chevron}>›</div>
                        </div>
                    );
                })}
            </div>
        </section>
    );
}

const styles: Record<string, React.CSSProperties> = {
    container: {
        width: "100%",
        maxWidth: "1152px",
        margin: "0 auto",
        marginBottom: "48px",
    },
    sectionHeader: {
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        marginBottom: "14px",
    },
    sectionTitle: {
        fontSize: "11px",
        fontWeight: 600,
        color: "var(--text-secondary)",
        letterSpacing: "1.1px",
    },
    sectionLegend: {
        fontSize: "11px",
        fontWeight: 500,
        color: "var(--text-secondary)",
    },
    tableCard: {
        backgroundColor: "var(--surface-card)",
        border: "1px solid var(--border-subtle)",
        borderRadius: "12px",
        overflow: "hidden",
        boxShadow: "0 1px 3px rgba(0, 0, 0, 0.02)",
    },
    columnsHeader: {
        height: "38px",
        padding: "0 24px",
        display: "flex",
        alignItems: "center",
        fontSize: "10px",
        fontWeight: 700,
        color: "var(--text-secondary)",
        letterSpacing: "0.8px",
        borderBottom: "1px solid var(--border-subtle)",
        backgroundColor: "var(--surface-card)",
    },
    accountRow: {
        height: "84px",
        padding: "0 24px",
        display: "flex",
        alignItems: "center",
        backgroundColor: "var(--surface-card)",
        borderBottom: "1px solid var(--border-subtle)",
        transition: "background-color 0.15s ease",
    },
    accountColPrimary: {
        flex: 1.2,
        display: "flex",
        alignItems: "center",
        gap: "16px",
    },
    iconBox: {
        width: "40px",
        height: "40px",
        borderRadius: "10px",
        border: "1px solid var(--border-subtle)",
        backgroundColor: "var(--bg-app)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontSize: "18px",
        flexShrink: 0,
    },
    nameBlock: {
        display: "flex",
        flexDirection: "column",
        gap: "3px",
    },
    accountName: {
        fontSize: "14px",
        fontWeight: 600,
        color: "var(--text-primary)",
    },
    currencySubtext: {
        fontSize: "11px",
        fontWeight: 500,
        color: "var(--text-secondary)",
    },
    typeBadge: {
        display: "inline-block",
        padding: "4px 9px",
        borderRadius: "6px",
        fontSize: "11px",
        fontWeight: 600,
    },
    balanceText: {
        width: "180px",
        textAlign: "right",
        fontSize: "17px",
        fontWeight: 600,
        color: "var(--text-primary)",
        letterSpacing: "-0.4px",
    },
    chevron: {
        width: "24px",
        textAlign: "right",
        fontSize: "18px",
        fontWeight: 400,
        color: "var(--text-muted)",
    },
    emptyCard: {
        padding: "56px 24px",
        backgroundColor: "var(--surface-card)",
        border: "1px solid var(--border-subtle)",
        borderRadius: "16px",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        textAlign: "center",
    },
    emptyIconCircle: {
        width: "56px",
        height: "56px",
        borderRadius: "50%",
        backgroundColor: "var(--surface-hover)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontSize: "26px",
        marginBottom: "16px",
    },
    emptyTitle: {
        fontSize: "20px",
        fontWeight: 700,
        color: "var(--text-primary)",
        marginBottom: "8px",
    },
    emptySubtitle: {
        fontSize: "14px",
        color: "var(--text-secondary)",
        maxWidth: "520px",
        lineHeight: 1.5,
        marginBottom: "24px",
    },
    btnPrimary: {
        height: "42px",
        padding: "0 20px",
        borderRadius: "8px",
        backgroundColor: "var(--emerald-primary)",
        color: "#FFFFFF",
        fontSize: "13px",
        fontWeight: 600,
        display: "flex",
        alignItems: "center",
        gap: "8px",
    },
};