import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface EntradaCuentaForm {
    nombre: string;
    tipo: string;
    moneda: string;
    saldo_inicial: string;
}

export interface ErrorValidacion {
    campo: string;   // "nombre" | "moneda" | "tipo" | "saldo_inicial" | "general"
    mensaje: string; // Mensaje en español devuelto por el dominio
}

export interface MonedaItem {
    codigo: string;
    nombre: string;
    simbolo: string;
    decimales: number;
}

const MONEDAS_CATALOGO_DEFAULT: MonedaItem[] = [
    { codigo: "VES", nombre: "Bolívar", simbolo: "Bs.", decimales: 2 },
    { codigo: "USD", nombre: "Dólar estadounidense", simbolo: "$", decimales: 2 },
    { codigo: "EUR", nombre: "Euro", simbolo: "€", decimales: 2 },
    { codigo: "USDT", nombre: "Tether (dólar digital)", simbolo: "USDT", decimales: 2 },
];

interface ModalNuevaCuentaProps {
    isOpen: boolean;
    onClose: () => void;
    onSubmit: (entrada: EntradaCuentaForm) => Promise<boolean>;
    errorExterno?: ErrorValidacion | null;
}

export default function ModalNuevaCuenta({
    isOpen,
    onClose,
    onSubmit,
    errorExterno,
}: ModalNuevaCuentaProps) {
    // Estado local del formulario
    const [nombre, setNombre] = useState("");
    const [tipo, setTipo] = useState("banco");
    const [moneda, setMoneda] = useState("VES");
    const [saldoInicial, setSaldoInicial] = useState("0,00");
    const [cargando, setCargando] = useState(false);
    const [errorActivo, setErrorActivo] = useState<ErrorValidacion | null>(null);
    const [listaMonedas, setListaMonedas] = useState<MonedaItem[]>(MONEDAS_CATALOGO_DEFAULT);

    // Cargar catálogo oficial de monedas desde Rust (RF-CAM-01 / invoke('listar_monedas'))
    useEffect(() => {
        const cargarCatalogoMonedas = async () => {
            if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
                try {
                    const catalogo = await invoke<MonedaItem[]>("listar_monedas");
                    if (catalogo && catalogo.length > 0) {
                        setListaMonedas(catalogo);
                    }
                } catch (err) {
                    console.warn("No se pudo cargar el catálogo de monedas desde Tauri:", err);
                }
            }
        };
        cargarCatalogoMonedas();
    }, []);

    // Sincronizar error si viene desde Rust
    useEffect(() => {
        if (errorExterno) {
            setErrorActivo(errorExterno);
        }
    }, [errorExterno]);

    // Limpiar campos al abrir o cerrar
    useEffect(() => {
        if (isOpen) {
            setNombre("");
            setTipo("banco");
            setMoneda("VES");
            setSaldoInicial("0,00");
            setErrorActivo(null);
        }
    }, [isOpen]);

    if (!isOpen) return null;

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setErrorActivo(null);
        setCargando(true);

        try {
            const exito = await onSubmit({
                nombre: nombre.trim(),
                tipo,
                moneda,
                saldo_inicial: saldoInicial.trim(),
            });

            if (exito) {
                onClose();
            }
        } finally {
            setCargando(false);
        }
    };

    // Helper para saber si un campo tiene error activo
    const tieneError = (campo: string) => errorActivo?.campo === campo;

    return (
        <div style={styles.overlay} onClick={onClose}>
            <div style={styles.modalCard} onClick={(e) => e.stopPropagation()}>
                {/* Cabecera del Modal */}
                <div style={styles.modalHeader}>
                    <div>
                        <h3 style={styles.modalTitle}>Nueva Cuenta</h3>
                        <p style={styles.modalSubtitle}>
                            Un lugar para tu dinero. Una moneda por cuenta.
                        </p>
                    </div>
                    <button onClick={onClose} style={styles.btnClose} title="Cerrar">
                        ✕
                    </button>
                </div>

                {/* Formulario */}
                <form onSubmit={handleSubmit}>
                    <div style={styles.modalBody}>
                        {/* Error General si existiera */}
                        {errorActivo?.campo === "general" && (
                            <div style={styles.alertBox}>{errorActivo.mensaje}</div>
                        )}

                        {/* 1. Nombre de la cuenta */}
                        <div style={styles.formGroup}>
                            <label style={styles.label}>Nombre de la cuenta</label>
                            <input
                                type="text"
                                value={nombre}
                                onChange={(e) => {
                                    setNombre(e.target.value);
                                    if (errorActivo?.campo === "nombre") setErrorActivo(null);
                                }}
                                placeholder="Ej. Banesco Ahorros, Cartera"
                                style={{
                                    ...styles.input,
                                    borderColor: tieneError("nombre") ? "var(--error-color)" : "var(--border-subtle)",
                                }}
                                disabled={cargando}
                                autoFocus
                            />
                            {tieneError("nombre") && (
                                <span style={styles.errorText}>{errorActivo?.mensaje}</span>
                            )}
                        </div>

                        {/* 2. Tipo de cuenta */}
                        <div style={styles.formGroup}>
                            <label style={styles.label}>Tipo de cuenta</label>
                            <select
                                value={tipo}
                                onChange={(e) => {
                                    setTipo(e.target.value);
                                    if (errorActivo?.campo === "tipo") setErrorActivo(null);
                                }}
                                style={{
                                    ...styles.select,
                                    borderColor: tieneError("tipo") ? "var(--error-color)" : "var(--border-subtle)",
                                }}
                                disabled={cargando}
                            >
                                <option value="efectivo">Efectivo</option>
                                <option value="banco">Banco</option>
                                <option value="billetera_digital">Billetera digital</option>
                            </select>
                            <span style={styles.helperText}>
                                Opciones: Efectivo · Banco · Billetera digital
                            </span>
                            {tieneError("tipo") && (
                                <span style={styles.errorText}>{errorActivo?.mensaje}</span>
                            )}
                        </div>

                        {/* 3. Moneda de la cuenta (INV-18: Inmutable) */}
                        <div style={styles.formGroup}>
                            <label style={styles.label}>Moneda de la cuenta</label>
                            <select
                                value={moneda}
                                onChange={(e) => {
                                    setMoneda(e.target.value);
                                    if (errorActivo?.campo === "moneda") setErrorActivo(null);
                                }}
                                style={{
                                    ...styles.select,
                                    borderColor: tieneError("moneda") ? "var(--error-color)" : "var(--border-subtle)",
                                }}
                                disabled={cargando}
                            >
                                {listaMonedas.map((m) => (
                                    <option key={m.codigo} value={m.codigo}>
                                        {m.codigo} — {m.nombre} ({m.simbolo})
                                    </option>
                                ))}
                            </select>
                            <div style={styles.immutableNotice}>
                                <span>🔒</span>
                                <span>La moneda no podrá cambiarse luego</span>
                            </div>
                            {tieneError("moneda") && (
                                <span style={styles.errorText}>{errorActivo?.mensaje}</span>
                            )}
                        </div>

                        {/* 4. Saldo inicial */}
                        <div style={styles.formGroup}>
                            <label style={styles.label}>Saldo inicial</label>
                            <input
                                type="text"
                                value={saldoInicial}
                                onChange={(e) => {
                                    setSaldoInicial(e.target.value);
                                    if (errorActivo?.campo === "saldo_inicial") setErrorActivo(null);
                                }}
                                placeholder="0,00"
                                style={{
                                    ...styles.input,
                                    borderColor: tieneError("saldo_inicial")
                                        ? "var(--error-color)"
                                        : "var(--border-subtle)",
                                }}
                                disabled={cargando}
                            />
                            <span style={styles.helperText}>
                                Puedes ingresar 0 o un monto con hasta 2 decimales (ej. 1250,50)
                            </span>
                            {tieneError("saldo_inicial") && (
                                <span style={styles.errorText}>{errorActivo?.mensaje}</span>
                            )}
                        </div>
                    </div>

                    {/* Footer de Acciones */}
                    <div style={styles.modalFooter}>
                        <button
                            type="button"
                            onClick={onClose}
                            style={styles.btnCancel}
                            disabled={cargando}
                        >
                            Cancelar
                        </button>
                        <button
                            type="submit"
                            style={styles.btnSubmit}
                            disabled={cargando}
                        >
                            {cargando ? "Guardando..." : "Guardar Cuenta"}
                        </button>
                    </div>
                </form>
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
        maxWidth: "480px",
        backgroundColor: "var(--surface-card)",
        borderRadius: "16px",
        border: "1px solid var(--border-subtle)",
        boxShadow: "0 24px 64px 0 rgba(15, 23, 42, 0.15)",
        overflow: "hidden",
    },
    modalHeader: {
        padding: "26px 28px 20px 28px",
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "space-between",
    },
    modalTitle: {
        fontSize: "22px",
        fontWeight: 700,
        color: "var(--text-primary)",
        letterSpacing: "-0.4px",
    },
    modalSubtitle: {
        fontSize: "12px",
        color: "var(--text-secondary)",
        marginTop: "4px",
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
    modalBody: {
        padding: "0 28px 24px 28px",
        display: "flex",
        flexDirection: "column",
        gap: "18px",
    },
    formGroup: {
        display: "flex",
        flexDirection: "column",
        gap: "6px",
    },
    label: {
        fontSize: "12px",
        fontWeight: 600,
        color: "var(--text-primary)",
    },
    input: {
        height: "44px",
        padding: "0 14px",
        borderRadius: "8px",
        border: "1px solid var(--border-subtle)",
        backgroundColor: "var(--surface-card)",
        color: "var(--text-primary)",
        fontSize: "13px",
        transition: "border-color 0.15s ease",
    },
    select: {
        height: "44px",
        padding: "0 14px",
        borderRadius: "8px",
        border: "1px solid var(--border-subtle)",
        backgroundColor: "var(--surface-card)",
        color: "var(--text-primary)",
        fontSize: "13px",
        cursor: "pointer",
    },
    helperText: {
        fontSize: "11px",
        color: "var(--text-secondary)",
    },
    immutableNotice: {
        display: "flex",
        alignItems: "center",
        gap: "6px",
        fontSize: "11px",
        color: "var(--text-secondary)",
        marginTop: "2px",
    },
    errorText: {
        fontSize: "11px",
        fontWeight: 600,
        color: "var(--error-color)",
        marginTop: "2px",
    },
    alertBox: {
        padding: "10px 14px",
        borderRadius: "8px",
        backgroundColor: "#FEE2E2",
        color: "#B91C1C",
        fontSize: "12px",
        fontWeight: 500,
    },
    modalFooter: {
        height: "76px",
        padding: "0 28px",
        backgroundColor: "var(--bg-app)",
        borderTop: "1px solid var(--border-subtle)",
        display: "flex",
        alignItems: "center",
        justifyContent: "flex-end",
        gap: "12px",
    },
    btnCancel: {
        height: "42px",
        padding: "0 18px",
        borderRadius: "8px",
        border: "1px solid var(--border-subtle)",
        backgroundColor: "var(--surface-card)",
        color: "var(--text-primary)",
        fontSize: "13px",
        fontWeight: 600,
    },
    btnSubmit: {
        height: "42px",
        padding: "0 20px",
        borderRadius: "8px",
        backgroundColor: "var(--emerald-primary)",
        color: "#FFFFFF",
        fontSize: "13px",
        fontWeight: 600,
    },
};