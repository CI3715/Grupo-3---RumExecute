"use client";

import { useState } from "react";
import {
  ICONOS_CATEGORIAS,
  formatearMonto,
  formatearMontoConSigno,
} from "../utils/formato";
import ModalNuevoMovimiento, { EntradaMovimientoForm } from "./ModalNuevoMovimiento";
import ModalCorregirMovimiento, {
  MovimientoItem,
  EntradaCorreccionForm,
} from "./ModalCorregirMovimiento";

export interface DetalleCuentaData {
  id: string;
  nombre: string;
  tipo: string;
  moneda: string;
  saldo_inicial_num: number;
  saldo_actual_num: number;
  total_ingresos_num: number;
  total_gastos_num: number;
  total_ajustes_num: number;
  movimientos: MovimientoItem[];
}

interface DetalleCuentaProps {
  cuenta: DetalleCuentaData;
  onVolver: () => void;
  categoriasGasto?: string[];
  categoriasIngreso?: string[];
  onCrearCategoria?: (entrada: { nombre: string; tipo: "gasto" | "ingreso" }) => Promise<boolean>;
  onRegistrarMovimiento: (entrada: EntradaMovimientoForm) => Promise<boolean>;
  onCorregirMovimiento: (entrada: EntradaCorreccionForm) => Promise<boolean>;
}

const ETIQUETA_TIPO: Record<string, string> = {
  efectivo: "💵 Efectivo",
  banco: "🏦 Banco",
  billetera_digital: "📱 Billetera digital",
};

export default function DetalleCuenta({
  cuenta,
  onVolver,
  categoriasGasto,
  categoriasIngreso,
  onCrearCategoria,
  onRegistrarMovimiento,
  onCorregirMovimiento,
}: DetalleCuentaProps) {
  const [modalNuevoMovimiento, setModalNuevoMovimiento] = useState(false);
  const [tipoMovimientoInicial, setTipoMovimientoInicial] = useState<"gasto" | "ingreso">("gasto");
  const [modalCorregir, setModalCorregir] = useState(false);
  const [movimientoACorregir, setMovimientoACorregir] = useState<MovimientoItem | null>(null);

  const esSaldoNegativo = cuenta.saldo_actual_num < 0;

  const abrirNuevo = (tipo: "gasto" | "ingreso") => {
    setTipoMovimientoInicial(tipo);
    setModalNuevoMovimiento(true);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "24px", width: "100%" }}>
      {/* 1. Barra Superior */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <button
          onClick={onVolver}
          style={{ fontSize: "13.5px", fontWeight: 600, color: "var(--text-secondary)" }}
        >
          ← Volver a Mis Cuentas
        </button>
        <div className="security-pill">
          <span>🔒</span>
          <span>100% Local y Privado</span>
        </div>
      </div>

      {/* 2. Cabecera de la Cuenta */}
      <div className="hero-section" style={{ marginBottom: 0 }}>
        <div>
          <div style={{ display: "flex", alignItems: "center", gap: "10px", flexWrap: "wrap" }}>
            <h1 className="hero-title">{cuenta.nombre}</h1>
            <span className="badge" style={{ backgroundColor: "var(--surface-hover)", border: "1px solid var(--border-subtle)" }}>
              {ETIQUETA_TIPO[cuenta.tipo] || cuenta.tipo}
            </span>
            <span className="badge-pill" style={{ backgroundColor: "var(--text-primary)", color: "#fff" }}>
              {cuenta.moneda}
            </span>
          </div>
          <p className="hero-subtitle">
            Moneda: {cuenta.moneda} · Todos tus datos seguros en este dispositivo
          </p>
        </div>

        <div style={{ display: "flex", gap: "10px" }}>
          <button onClick={() => abrirNuevo("gasto")} className="btn btn-danger">
            - Registrar Gasto
          </button>
          <button onClick={() => abrirNuevo("ingreso")} className="btn btn-primary">
            + Registrar Ingreso
          </button>
        </div>
      </div>

      {/* 3. Tarjeta de Saldo Actual y Composición */}
      <div className={`balance-hero-card ${esSaldoNegativo ? "is-negative" : ""}`}>
        <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", flexWrap: "wrap", gap: "12px" }}>
          <div>
            <span style={{ fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase" }}>
              Saldo Actual de la Cuenta
            </span>
            <div
              style={{
                fontSize: "36px",
                fontWeight: 800,
                letterSpacing: "-1px",
                marginTop: "4px",
                color: esSaldoNegativo ? "var(--error-color)" : "var(--text-primary)",
              }}
            >
              {formatearMonto(cuenta.saldo_actual_num, cuenta.moneda)}
            </div>
            {esSaldoNegativo && (
              <span className="badge" style={{ backgroundColor: "#FEF2F2", color: "var(--error-color)", marginTop: "6px" }}>
                ⚠️ Cuenta en sobregiro / saldo negativo
              </span>
            )}
          </div>
          <div className="security-pill" style={{ backgroundColor: "var(--bg-app)" }}>
            <span>⚡</span>
            <span>Balance calculado en base a tus movimientos</span>
          </div>
        </div>

        <div className="equation-bar">
          <div>
            <div style={{ fontSize: "11px", fontWeight: 600, color: "var(--text-muted)" }}>INICIAL</div>
            <div style={{ fontSize: "13.5px", fontWeight: 600 }}>
              {formatearMonto(cuenta.saldo_inicial_num, cuenta.moneda)}
            </div>
          </div>
          <strong style={{ color: "var(--text-muted)" }}>+</strong>
          <div>
            <div style={{ fontSize: "11px", fontWeight: 600, color: "var(--text-muted)" }}>INGRESOS</div>
            <div style={{ fontSize: "13.5px", fontWeight: 600, color: "var(--emerald-primary)" }}>
              +{formatearMonto(cuenta.total_ingresos_num, cuenta.moneda)}
            </div>
          </div>
          <strong style={{ color: "var(--text-muted)" }}>−</strong>
          <div>
            <div style={{ fontSize: "11px", fontWeight: 600, color: "var(--text-muted)" }}>GASTOS</div>
            <div style={{ fontSize: "13.5px", fontWeight: 600, color: "var(--error-color)" }}>
              −{formatearMonto(cuenta.total_gastos_num, cuenta.moneda)}
            </div>
          </div>
          <strong style={{ color: "var(--text-muted)" }}>±</strong>
          <div>
            <div style={{ fontSize: "11px", fontWeight: 600, color: "var(--text-muted)" }}>AJUSTES</div>
            <div style={{ fontSize: "13.5px", fontWeight: 600, color: "#2563EB" }}>
              {cuenta.total_ajustes_num >= 0 ? "+" : "−"}
              {formatearMonto(Math.abs(cuenta.total_ajustes_num), cuenta.moneda)}
            </div>
          </div>
          <strong style={{ color: "var(--text-muted)" }}>=</strong>
          <div>
            <div style={{ fontSize: "11px", fontWeight: 600, color: "var(--text-muted)" }}>ACTUAL</div>
            <div style={{ fontSize: "13.5px", fontWeight: 700 }}>
              {formatearMonto(cuenta.saldo_actual_num, cuenta.moneda)}
            </div>
          </div>
        </div>
      </div>

      {/* 4. Historial de Movimientos */}
      <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
        <div style={{ display: "flex", alignItems: "flex-end", justifyContent: "space-between" }}>
          <div>
            <h2 style={{ fontSize: "18px", fontWeight: 700 }}>Movimientos</h2>
            <p className="helper-text" style={{ marginTop: "2px" }}>
              Historial ordenado cronológicamente con tus ingresos, gastos y ajustes.
            </p>
          </div>
          <span className="security-pill">
            {cuenta.movimientos.length} {cuenta.movimientos.length === 1 ? "movimiento" : "movimientos"}
          </span>
        </div>

        {cuenta.movimientos.length === 0 ? (
          <div className="empty-card" style={{ borderStyle: "dashed" }}>
            <div style={{ fontSize: "32px", marginBottom: "8px" }}>📖</div>
            <h4 style={{ fontSize: "15px", fontWeight: 700 }}>Aún no hay movimientos en esta cuenta</h4>
            <p className="helper-text" style={{ maxWidth: "420px", marginTop: "4px" }}>
              Comienza registrando tu primera compra o entrada de dinero con los botones superiores.
            </p>
            <div style={{ display: "flex", gap: "10px", marginTop: "16px" }}>
              <button onClick={() => abrirNuevo("gasto")} className="btn btn-danger">
                - Registrar Gasto
              </button>
              <button onClick={() => abrirNuevo("ingreso")} className="btn btn-primary">
                + Registrar Ingreso
              </button>
            </div>
          </div>
        ) : (
          <div className="table-card">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Fecha</th>
                  <th>Tipo & Categoría</th>
                  <th>Nota / Concepto</th>
                  <th style={{ textAlign: "right" }}>Monto</th>
                  <th style={{ textAlign: "center" }}>Estado</th>
                  <th style={{ textAlign: "right" }}>Acción</th>
                </tr>
              </thead>
              <tbody>
                {cuenta.movimientos.map((mov) => {
                  const esAjuste = mov.tipo === "ajuste";
                  const esCorregido = mov.estado === "corregido";
                  const nombreCat = typeof mov.categoria === "object" && mov.categoria !== null
                    ? ((mov.categoria as any).nombre || (mov.categoria as any).id || "Varios")
                    : String(mov.categoria || "Varios");

                  return (
                    <tr
                      key={mov.id}
                      style={{
                        opacity: esCorregido ? 0.75 : 1,
                        backgroundColor: esCorregido ? "#F8FAFC" : "transparent",
                      }}
                    >
                      <td style={{ color: "var(--text-secondary)", fontWeight: 500 }}>
                        {mov.fecha}
                      </td>
                      <td>
                        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                          <span style={{ fontSize: "18px" }}>{ICONOS_CATEGORIAS[nombreCat] || "🏷️"}</span>
                          <div>
                            <div style={{ fontWeight: 600 }}>{nombreCat}</div>
                            <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                              {mov.tipo === "gasto" ? "Gasto" : mov.tipo === "ingreso" ? "Ingreso" : "Ajuste contable"}
                            </div>
                          </div>
                        </div>
                      </td>
                      <td>
                        {mov.nota || <span style={{ color: "var(--text-muted)" }}>—</span>}
                        {mov.id_movimiento_referenciado && (
                          <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                            Ref: {mov.id_movimiento_referenciado.slice(0, 10)}...
                          </div>
                        )}
                      </td>
                      <td style={{ textAlign: "right" }}>
                        <span
                          style={{
                            fontWeight: 700,
                            color:
                              mov.tipo === "gasto"
                                ? "var(--error-color)"
                                : mov.tipo === "ingreso"
                                ? "var(--emerald-primary)"
                                : "#2563EB",
                            textDecoration: esCorregido ? "line-through" : "none",
                          }}
                        >
                          {formatearMontoConSigno(mov.monto_num, cuenta.moneda, mov.tipo)}
                        </span>
                      </td>
                      <td style={{ textAlign: "center" }}>
                        {esCorregido ? (
                          <span className="badge-pill badge-corregido">⚪ Corregido</span>
                        ) : esAjuste ? (
                          <span className="badge-pill badge-ajuste">🔵 Ajuste</span>
                        ) : (
                          <span className="badge-pill badge-vigente">🟢 Vigente</span>
                        )}
                      </td>
                      <td style={{ textAlign: "right" }}>
                        {!esCorregido && !esAjuste ? (
                          <button
                            onClick={() => {
                              setMovimientoACorregir(mov);
                              setModalCorregir(true);
                            }}
                            className="btn-corregir"
                          >
                            Corregir
                          </button>
                        ) : (
                          <span style={{ color: "var(--text-muted)" }}>—</span>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* 5. Modales */}
      <ModalNuevoMovimiento
        isOpen={modalNuevoMovimiento}
        onClose={() => setModalNuevoMovimiento(false)}
        cuenta={{
          id: cuenta.id,
          nombre: cuenta.nombre,
          moneda: cuenta.moneda,
          saldo_actual_num: cuenta.saldo_actual_num,
        }}
        tipoInicial={tipoMovimientoInicial}
        categoriasGasto={categoriasGasto}
        categoriasIngreso={categoriasIngreso}
        onCrearCategoria={onCrearCategoria}
        onSubmit={onRegistrarMovimiento}
      />

      <ModalCorregirMovimiento
        isOpen={modalCorregir}
        onClose={() => {
          setModalCorregir(false);
          setMovimientoACorregir(null);
        }}
        movimiento={movimientoACorregir}
        moneda={cuenta.moneda}
        onSubmit={onCorregirMovimiento}
      />
    </div>
  );
}
