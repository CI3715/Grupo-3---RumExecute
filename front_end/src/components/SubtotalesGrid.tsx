"use client";

import React from "react";

// Tipado del subtotal por moneda que entrega la capa de aplicación
export interface SubtotalMoneda {
    moneda: string;              // "VES" | "USD" | "EUR" | "USDT"
    nombre_moneda?: string;      // "Bolívar" | "Dólar" | "Euro" | "Tether"
    simbolo?: string;            // "Bs." | "$" | "€" | "USDT"
    subtotal_formateado: string; // "Bs. 1.250,50"
    cantidad_cuentas: number;    // 1
}

interface SubtotalesGridProps {
    subtotales: SubtotalMoneda[];
}

// Diccionario de nombres en español
const NOMBRES_MONEDA: Record<string, string> = {
    VES: "Bolívar",
    USD: "Dólar",
    EUR: "Euro",
    USDT: "Tether",
};

export default function SubtotalesGrid({ subtotales }: SubtotalesGridProps) {
    // Si no hay cuentas registradas aún, no mostramos tarjetas de saldos
    if (subtotales.length === 0) {
        return null;
    }

    return (
        <section style={styles.sectionContainer}>
            <div style={styles.grid}>
                {subtotales.map((item) => {
                    const nombre = item.nombre_moneda || NOMBRES_MONEDA[item.moneda] || item.moneda;
                    const textoCuentas =
                        item.cantidad_cuentas === 1 ? "1 cuenta" : `${item.cantidad_cuentas} cuentas`;

                    return (
                        <div key={item.moneda} style={styles.card}>
                            {/* Fila Superior: Nombre de la moneda y Pill con el código */}
                            <div style={styles.cardHeader}>
                                <span style={styles.currencyName}>{nombre}</span>
                                <span style={styles.currencyPill}>{item.moneda}</span>
                            </div>

                            {/* Fila Inferior: Monto destacado y contador de cuentas */}
                            <div style={styles.cardFooter}>
                                <div style={styles.amountText}>{item.subtotal_formateado}</div>
                                <div style={styles.countText}>{textoCuentas}</div>
                            </div>
                        </div>
                    );
                })}
            </div>
        </section>
    );
}

const styles: Record<string, React.CSSProperties> = {
    sectionContainer: {
        width: "100%",
        maxWidth: "1152px",
        margin: "0 auto",
        marginBottom: "32px",
    },
    grid: {
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))",
        gap: "20px",
    },
    card: {
        height: "152px",
        padding: "24px",
        backgroundColor: "var(--surface-card)",
        border: "1px solid var(--border-subtle)",
        borderRadius: "12px",
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
        boxShadow: "0 2px 4px 0 rgba(15, 23, 42, 0.02)",
        transition: "transform 0.15s ease, box-shadow 0.15s ease",
    },
    cardHeader: {
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
    },
    currencyName: {
        fontSize: "14px",
        fontWeight: 500,
        color: "var(--text-secondary)",
    },
    currencyPill: {
        fontSize: "10px",
        fontWeight: 700,
        color: "var(--text-secondary)",
        backgroundColor: "var(--bg-app)",
        border: "1px solid var(--border-subtle)",
        padding: "3px 8px",
        borderRadius: "6px",
        letterSpacing: "0.5px",
    },
    cardFooter: {
        display: "flex",
        flexDirection: "column",
        gap: "4px",
    },
    amountText: {
        fontSize: "28px",
        fontWeight: 700,
        color: "var(--text-primary)",
        letterSpacing: "-0.5px",
        lineHeight: 1.1,
    },
    countText: {
        fontSize: "12px",
        fontWeight: 400,
        color: "var(--text-secondary)",
    },
};