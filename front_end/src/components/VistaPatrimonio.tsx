"use client";

import { useState } from "react";
import {
  ResumenPatrimonioData,
  MonedaItem,
  EntradaRegistroTasa,
} from "../types/tasas";
import { obtenerFechaHoyISO } from "../services/tasasService";

interface VistaPatrimonioProps {
  data: ResumenPatrimonioData;
  monedasDisponibles: MonedaItem[];
  monedaBaseActual: string;
  fechaConsulta: string;
  onCambiarFecha: (fecha: string) => void;
  onCambiarMonedaBase: (nuevaMoneda: string) => void;
  onGuardarTasa: (entrada: EntradaRegistroTasa) => Promise<void>;
  onIrATasas: (origen?: string, destino?: string) => void;
  onOpenCrearMoneda: () => void;
}

export default function VistaPatrimonio({
  data,
  monedasDisponibles,
  monedaBaseActual,
  fechaConsulta,
  onCambiarFecha,
  onCambiarMonedaBase,
  onGuardarTasa,
  onIrATasas,
  onOpenCrearMoneda,
}: VistaPatrimonioProps) {
  const hoy = obtenerFechaHoyISO();
  const { cuentas, subtotales, total_consolidado, total_consolidado_formateado, moneda_base, simbolo_base, tasa_faltante } = data;

  // Estado para el registro rápido de tasa faltante (formulario integrado sin exceso de modales)
  const [registroRapido, setRegistroRapido] = useState<{
    abierto: boolean;
    origen: string;
    destino: string;
    fecha: string;
    valor: string;
    fuente: string;
    error: string | null;
    guardando: boolean;
  }>({
    abierto: false,
    origen: "VES",
    destino: moneda_base,
    fecha: fechaConsulta,
    valor: "",
    fuente: "",
    error: null,
    guardando: false,
  });

  const abrirRegistroRapido = (origen: string, destino: string, fecha?: string) => {
    setRegistroRapido({
      abierto: true,
      origen,
      destino: destino || moneda_base,
      fecha: fecha || fechaConsulta,
      valor: "",
      fuente: "Manual",
      error: null,
      guardando: false,
    });
  };

  const handleGuardarRegistroRapido = async (e: React.FormEvent) => {
    e.preventDefault();
    const valLimpio = registroRapido.valor.trim().replace(",", ".");
    const num = parseFloat(valLimpio);

    if (isNaN(num) || num <= 0) {
      setRegistroRapido((prev) => ({ ...prev, error: "El valor debe ser mayor a 0." }));
      return;
    }

    if (registroRapido.fecha > hoy) {
      setRegistroRapido((prev) => ({
        ...prev,
        error: `La fecha (${registroRapido.fecha}) no puede ser futura.`,
      }));
      return;
    }

    try {
      setRegistroRapido((prev) => ({ ...prev, guardando: true, error: null }));
      await onGuardarTasa({
        origen: registroRapido.origen,
        destino: registroRapido.destino,
        valor: valLimpio,
        fecha_vigencia: registroRapido.fecha,
        fuente: registroRapido.fuente.trim() || undefined,
      });
      setRegistroRapido((prev) => ({ ...prev, abierto: false, guardando: false }));
    } catch (err: any) {
      setRegistroRapido((prev) => ({
        ...prev,
        guardando: false,
        error: err?.message || "Error al registrar la tasa de cambio.",
      }));
    }
  };

  const getTipoBadge = (tipo: string) => {
    const t = tipo.toLowerCase();
    if (t === "efectivo") {
      return { bg: "var(--badge-cash-bg)", color: "var(--badge-cash-text)", label: "Efectivo" };
    }
    if (t === "banco") {
      return { bg: "var(--badge-bank-bg)", color: "var(--badge-bank-text)", label: "Banco" };
    }
    return { bg: "var(--badge-wallet-bg)", color: "var(--badge-wallet-text)", label: "Billetera digital" };
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
      {/* 1. Header con descripción normativa */}
      <div className="hero-section">
        <div>
          <h1 className="hero-title">Patrimonio Consolidado</h1>
          <p className="hero-subtitle">
            Valuación multimoneda a una fecha de corte en tu moneda preferida.
          </p>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <button
            onClick={() => onIrATasas()}
            className="btn btn-secondary"
            style={{ fontWeight: 600, fontSize: "13px" }}
          >
            Ver Historial de Tasas
          </button>
        </div>
      </div>

      {/* 2. Barra de Controles: Selector de Fecha y Selector de Moneda Base */}
      <div
        style={{
          display: "flex",
          gap: "16px",
          flexWrap: "wrap",
          alignItems: "center",
          justifyContent: "space-between",
          backgroundColor: "var(--surface-card)",
          padding: "16px 20px",
          borderRadius: "12px",
          border: "1px solid var(--border-subtle)",
        }}
      >
        {/* Selector de Fecha de Consulta */}
        <div style={{ display: "flex", alignItems: "center", gap: "10px", flexWrap: "wrap" }}>
          <span style={{ fontSize: "13px", fontWeight: 700, color: "var(--text-secondary)" }}>
            Fecha de Consulta:
          </span>
          <input
            type="date"
            max={hoy}
            value={fechaConsulta}
            onChange={(e) => {
              if (e.target.value) onCambiarFecha(e.target.value);
            }}
            style={{
              border: "1px solid var(--border-subtle)",
              borderRadius: "6px",
              padding: "6px 10px",
              fontSize: "13px",
              fontWeight: 600,
              color: "var(--text-primary)",
              backgroundColor: "#FFFFFF",
            }}
          />
          <div style={{ display: "flex", gap: "6px" }}>
            <button
              type="button"
              onClick={() => onCambiarFecha(hoy)}
              style={{
                fontSize: "12px",
                fontWeight: fechaConsulta === hoy ? 700 : 500,
                padding: "5px 10px",
                borderRadius: "6px",
                backgroundColor: fechaConsulta === hoy ? "rgba(5, 150, 105, 0.08)" : "var(--surface-hover)",
                color: fechaConsulta === hoy ? "var(--emerald-primary)" : "var(--text-secondary)",
                border: fechaConsulta === hoy ? "1.5px solid var(--emerald-primary)" : "1px solid transparent",
              }}
            >
              Hoy
            </button>
            <button
              type="button"
              onClick={() => {
                const d = new Date();
                d.setDate(d.getDate() - 1);
                onCambiarFecha(d.toISOString().split("T")[0]);
              }}
              style={{
                fontSize: "12px",
                fontWeight: 500,
                padding: "5px 10px",
                borderRadius: "6px",
                backgroundColor: "var(--surface-hover)",
                color: "var(--text-secondary)",
              }}
            >
              Ayer
            </button>
          </div>
        </div>

        {/* Selector de Moneda Base */}
        <div style={{ display: "flex", alignItems: "center", gap: "10px", flexWrap: "wrap" }}>
          <span style={{ fontSize: "13px", fontWeight: 700, color: "var(--text-secondary)" }}>
            Moneda Base:
          </span>
          <select
            value={monedaBaseActual}
            onChange={(e) => onCambiarMonedaBase(e.target.value)}
            style={{
              border: "1px solid var(--border-subtle)",
              borderRadius: "6px",
              padding: "6px 12px",
              fontSize: "13px",
              fontWeight: 700,
              color: "var(--emerald-primary)",
              backgroundColor: "#FFFFFF",
              cursor: "pointer",
            }}
          >
            {monedasDisponibles.map((m) => (
              <option key={m.codigo} value={m.codigo}>
                {m.codigo} — {m.nombre} ({m.simbolo})
              </option>
            ))}
          </select>

          <button
            type="button"
            onClick={onOpenCrearMoneda}
            title="Crear una moneda personalizada"
            style={{
              fontSize: "12px",
              fontWeight: 600,
              color: "var(--text-secondary)",
              backgroundColor: "var(--surface-hover)",
              padding: "6px 10px",
              borderRadius: "6px",
            }}
          >
            + Moneda Propia
          </button>
        </div>
      </div>

      {/* 3. Formulario Integrado de Registro Rápido (Se abre al presionar "Registrar Tasa Ahora") */}
      {registroRapido.abierto && (
        <div
          style={{
            backgroundColor: "#FFFFFF",
            border: "2px solid var(--emerald-primary)",
            borderRadius: "12px",
            padding: "20px 24px",
            boxShadow: "0 4px 12px rgba(5, 150, 105, 0.1)",
          }}
        >
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "14px" }}>
            <div>
              <h3 style={{ fontSize: "15px", fontWeight: 800, color: "var(--text-primary)" }}>
                Registrar Tasa: {registroRapido.origen} ➔ {registroRapido.destino}
              </h3>
              <p style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
                Ingresa el valor para habilitar la conversión automática y consolidar tu patrimonio.
              </p>
            </div>
            <button
              onClick={() => setRegistroRapido((prev) => ({ ...prev, abierto: false }))}
              style={{ fontSize: "16px", color: "var(--text-secondary)", padding: "4px" }}
            >
              ✕
            </button>
          </div>

          {registroRapido.error && (
            <div style={{ backgroundColor: "#FEF2F2", color: "#DC2626", padding: "8px 12px", borderRadius: "6px", fontSize: "12px", fontWeight: 600, marginBottom: "12px" }}>
              {registroRapido.error}
            </div>
          )}

          <form onSubmit={handleGuardarRegistroRapido} style={{ display: "flex", gap: "12px", flexWrap: "wrap", alignItems: "flex-end" }}>
            <div style={{ flex: 1, minWidth: "180px" }}>
              <label style={{ display: "block", fontSize: "11.5px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                1 {registroRapido.origen} = ¿Cuántos {registroRapido.destino}?
              </label>
              <input
                type="text"
                placeholder="Ej. 0.025 o 40.00"
                value={registroRapido.valor}
                onChange={(e) => setRegistroRapido((prev) => ({ ...prev, valor: e.target.value }))}
                style={{ width: "100%", padding: "8px 12px", border: "1px solid var(--border-subtle)", borderRadius: "6px", fontSize: "13px", fontWeight: 600 }}
                required
                autoFocus
              />
            </div>

            <div style={{ minWidth: "150px" }}>
              <label style={{ display: "block", fontSize: "11.5px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                Fecha de Vigencia
              </label>
              <input
                type="date"
                max={hoy}
                value={registroRapido.fecha}
                onChange={(e) => setRegistroRapido((prev) => ({ ...prev, fecha: e.target.value }))}
                style={{ width: "100%", padding: "8px 12px", border: "1px solid var(--border-subtle)", borderRadius: "6px", fontSize: "13px" }}
                required
              />
            </div>

            <div style={{ flex: 1, minWidth: "160px" }}>
              <label style={{ display: "block", fontSize: "11.5px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                Fuente (Opcional)
              </label>
              <input
                type="text"
                placeholder="Ej. BCV, DolarToday"
                value={registroRapido.fuente}
                onChange={(e) => setRegistroRapido((prev) => ({ ...prev, fuente: e.target.value }))}
                style={{ width: "100%", padding: "8px 12px", border: "1px solid var(--border-subtle)", borderRadius: "6px", fontSize: "13px" }}
              />
            </div>

            <div style={{ display: "flex", gap: "8px" }}>
              <button
                type="button"
                onClick={() => setRegistroRapido((prev) => ({ ...prev, abierto: false }))}
                className="btn btn-secondary"
                style={{ padding: "8px 14px", fontSize: "12.5px" }}
              >
                Cancelar
              </button>
              <button
                type="submit"
                disabled={registroRapido.guardando}
                className="btn btn-primary"
                style={{ padding: "8px 18px", fontSize: "12.5px" }}
              >
                {registroRapido.guardando ? "Guardando..." : "Guardar Tasa"}
              </button>
            </div>
          </form>
        </div>
      )}

      {/* 4. Banner de Advertencia de Tasa Faltante */}
      {tasa_faltante && !registroRapido.abierto && (
        <div
          style={{
            backgroundColor: "#FEF2F2",
            border: "1px solid #FCA5A5",
            borderRadius: "12px",
            padding: "16px 20px",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            flexWrap: "wrap",
            gap: "16px",
          }}
        >
          <div style={{ display: "flex", alignItems: "flex-start", gap: "12px", flex: 1, minWidth: "260px" }}>
            <div>
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <strong style={{ fontSize: "13.5px", color: "#991B1B" }}>
                  TASA DE CAMBIO FALTANTE
                </strong>
                <span style={{ fontSize: "11px", fontWeight: 700, backgroundColor: "#FEE2E2", color: "#B91C1C", padding: "1px 6px", borderRadius: "4px" }}>
                  Total Bloqueado
                </span>
              </div>
              <p style={{ fontSize: "12.5px", color: "#7F1D1D", marginTop: "3px", lineHeight: 1.4 }}>
                {tasa_faltante.mensaje}
              </p>
            </div>
          </div>

          <button
            onClick={() => abrirRegistroRapido(tasa_faltante.moneda_origen, tasa_faltante.moneda_destino, tasa_faltante.fecha_requerida)}
            style={{
              backgroundColor: "#FFFFFF",
              color: "#DC2626",
              border: "1.5px solid #DC2626",
              fontWeight: 700,
              fontSize: "12.5px",
              padding: "8px 16px",
              borderRadius: "8px",
              display: "flex",
              alignItems: "center",
              gap: "6px",
              boxShadow: "0 1px 2px rgba(220, 38, 38, 0.08)",
            }}
          >
            <span>+</span>
            <span>Registrar {tasa_faltante.moneda_origen} ➔ {tasa_faltante.moneda_destino}</span>
          </button>
        </div>
      )}

      {/* 5. Tarjeta Principal de Total Consolidado */}
      <div
        style={{
          background: total_consolidado !== null
            ? "linear-gradient(135deg, #065F46 0%, #047857 50%, #059669 100%)"
            : "#334155",
          borderRadius: "16px",
          padding: "26px 32px",
          color: "#FFFFFF",
          boxShadow: "0 8px 20px -4px rgba(5, 150, 105, 0.2)",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexWrap: "wrap",
          gap: "18px",
        }}
      >
        <div>
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <span style={{ fontSize: "12px", fontWeight: 700, letterSpacing: "1px", opacity: 0.85, textTransform: "uppercase" }}>
              Patrimonio Total Consolidado
            </span>
            <span style={{ fontSize: "11px", fontWeight: 700, backgroundColor: "rgba(255, 255, 255, 0.2)", padding: "2px 8px", borderRadius: "9999px" }}>
              Base: {moneda_base} ({simbolo_base})
            </span>
          </div>

          <div style={{ fontSize: "34px", fontWeight: 800, marginTop: "6px", letterSpacing: "-0.5px" }}>
            {total_consolidado !== null ? (
              total_consolidado_formateado
            ) : (
              <span style={{ fontSize: "20px", color: "#FCA5A5", fontWeight: 700 }}>
                No disponible (Tasa faltante)
              </span>
            )}
          </div>

          <div style={{ fontSize: "12px", opacity: 0.85, marginTop: "4px" }}>
            Valuado a fecha de consulta: <strong>{data.fecha_consulta}</strong>
            {total_consolidado !== null && " · Calculado con las tasas vigentes a la fecha"}
          </div>
        </div>

        <div style={{ textAlign: "right" }}>
          <div style={{ fontSize: "11.5px", opacity: 0.8 }}>Cuentas Activas</div>
          <div style={{ fontSize: "24px", fontWeight: 800 }}>{cuentas.length}</div>
          <div style={{ fontSize: "11px", opacity: 0.75, marginTop: "2px" }}>
            {subtotales.length} {subtotales.length === 1 ? "moneda diferente" : "monedas diferentes"}
          </div>
        </div>
      </div>

      {/* 6. Grid de Subtotales por Moneda Original */}
      <div>
        <h3 style={{ fontSize: "15px", fontWeight: 700, color: "var(--text-primary)", marginBottom: "12px" }}>
          Saldos por Moneda
        </h3>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", gap: "16px" }}>
          {subtotales.map((sub) => {
            const esBase = sub.moneda === moneda_base;
            return (
              <div
                key={sub.moneda}
                style={{
                  backgroundColor: "var(--surface-card)",
                  border: esBase ? "2px solid var(--emerald-primary)" : "1px solid var(--border-subtle)",
                  borderRadius: "12px",
                  padding: "16px 18px",
                  display: "flex",
                  flexDirection: "column",
                  justifyContent: "space-between",
                }}
              >
                <div>
                  <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                    <span style={{ fontSize: "13.5px", fontWeight: 700, color: "var(--text-secondary)" }}>
                      {sub.nombre} ({sub.moneda})
                    </span>
                    {esBase ? (
                      <span style={{ fontSize: "10px", fontWeight: 700, backgroundColor: "#ECFDF5", color: "#047857", padding: "2px 6px", borderRadius: "4px" }}>
                        MONEDA BASE
                      </span>
                    ) : (
                      <button
                        onClick={() => onIrATasas(sub.moneda, moneda_base)}
                        style={{ fontSize: "11px", color: "var(--emerald-primary)", fontWeight: 600 }}
                      >
                        Tasas ↗
                      </button>
                    )}
                  </div>

                  <div style={{ fontSize: "20px", fontWeight: 800, color: "var(--text-primary)", marginTop: "8px" }}>
                    {sub.subtotal_formateado}
                  </div>
                </div>

                <div style={{ marginTop: "12px", paddingTop: "8px", borderTop: "1px dashed var(--border-subtle)", fontSize: "11.5px" }}>
                  {esBase ? (
                    <span style={{ color: "var(--text-muted)" }}>
                      {sub.cantidad_cuentas} {sub.cantidad_cuentas === 1 ? "cuenta" : "cuentas"} en moneda base
                    </span>
                  ) : sub.subtotal_convertido_formateado ? (
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                      <span style={{ color: "var(--text-muted)" }}>Equivalente:</span>
                      <strong style={{ color: "var(--emerald-primary)" }}>{sub.subtotal_convertido_formateado}</strong>
                    </div>
                  ) : (
                    <span style={{ color: "#DC2626", fontWeight: 600 }}>Sin tasa vigente hacia {moneda_base}</span>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* 7. Tabla Detallada de Cuentas */}
      <div
        style={{
          backgroundColor: "var(--surface-card)",
          borderRadius: "14px",
          border: "1px solid var(--border-subtle)",
          overflow: "hidden",
        }}
      >
        <div style={{ padding: "16px 20px", borderBottom: "1px solid var(--border-subtle)" }}>
          <h3 style={{ fontSize: "15px", fontWeight: 700, color: "var(--text-primary)" }}>
            Detalle Consolidado de Cuentas
          </h3>
          <p style={{ fontSize: "12px", color: "var(--text-secondary)", marginTop: "2px" }}>
            Conversión exacta a <strong>{moneda_base}</strong> a la fecha de consulta
          </p>
        </div>

        {cuentas.length === 0 ? (
          <div style={{ padding: "40px 20px", textAlign: "center", color: "var(--text-secondary)", fontSize: "13px" }}>
            No hay cuentas registradas en el sistema.
          </div>
        ) : (
          <div style={{ overflowX: "auto" }}>
            <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left" }}>
              <thead>
                <tr style={{ backgroundColor: "#F8FAFC", borderBottom: "1px solid var(--border-subtle)" }}>
                  <th style={{ padding: "10px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase" }}>
                    Cuenta
                  </th>
                  <th style={{ padding: "10px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase" }}>
                    Tipo
                  </th>
                  <th style={{ padding: "10px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase", textAlign: "right" }}>
                    Saldo Original
                  </th>
                  <th style={{ padding: "10px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase", textAlign: "center" }}>
                    Tasa Vigente ({data.fecha_consulta})
                  </th>
                  <th style={{ padding: "10px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase", textAlign: "right" }}>
                    Saldo en {moneda_base}
                  </th>
                  <th style={{ padding: "10px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase", textAlign: "center" }}>
                    Acción
                  </th>
                </tr>
              </thead>
              <tbody>
                {cuentas.map((c) => {
                  const badge = getTipoBadge(c.tipo);
                  const faltaTasa = c.moneda_original !== moneda_base && c.saldo_convertido === null;

                  return (
                    <tr
                      key={c.id}
                      style={{
                        borderBottom: "1px solid var(--border-subtle)",
                        backgroundColor: faltaTasa ? "#FFFBEB" : "transparent",
                      }}
                    >
                      <td style={{ padding: "14px 18px" }}>
                        <div style={{ fontWeight: 700, color: "var(--text-primary)", fontSize: "13.5px" }}>
                          {c.nombre}
                        </div>
                      </td>

                      <td style={{ padding: "14px 18px" }}>
                        <span
                          style={{
                            backgroundColor: badge.bg,
                            color: badge.color,
                            padding: "2px 7px",
                            borderRadius: "5px",
                            fontSize: "11px",
                            fontWeight: 700,
                          }}
                        >
                          {badge.label}
                        </span>
                      </td>

                      <td style={{ padding: "14px 18px", textAlign: "right", fontWeight: 700, fontSize: "13.5px" }}>
                        {c.saldo_original_formateado}
                      </td>

                      <td style={{ padding: "14px 18px", textAlign: "center", fontSize: "12px" }}>
                        {c.moneda_original === moneda_base ? (
                          <span style={{ color: "var(--text-muted)", fontWeight: 600 }}>1.00 (Base)</span>
                        ) : c.tasa_aplicada !== null ? (
                          <div>
                            <span style={{ fontWeight: 700, color: "var(--text-primary)" }}>
                              {c.tasa_texto}
                            </span>
                            {c.es_tasa_inversa && (
                              <span
                                style={{
                                  display: "inline-block",
                                  marginLeft: "4px",
                                  fontSize: "9.5px",
                                  backgroundColor: "#EFF6FF",
                                  color: "#2563EB",
                                  padding: "1px 4px",
                                  borderRadius: "3px",
                                  fontWeight: 700,
                                }}
                              >
                                Inversa
                              </span>
                            )}
                          </div>
                        ) : (
                          <span style={{ color: "#DC2626", fontWeight: 700, fontSize: "11.5px" }}>
                            ⚠️ Tasa Faltante
                          </span>
                        )}
                      </td>

                      <td style={{ padding: "14px 18px", textAlign: "right", fontWeight: 800, fontSize: "13.5px" }}>
                        {c.saldo_convertido_formateado ? (
                          <span style={{ color: "var(--emerald-primary)" }}>{c.saldo_convertido_formateado}</span>
                        ) : (
                          <span style={{ color: "#DC2626" }}>—</span>
                        )}
                      </td>

                      <td style={{ padding: "14px 18px", textAlign: "center" }}>
                        {faltaTasa ? (
                          <button
                            onClick={() => abrirRegistroRapido(c.moneda_original, moneda_base, data.fecha_consulta)}
                            style={{
                              fontSize: "11px",
                              fontWeight: 700,
                              color: "#DC2626",
                              backgroundColor: "#FFFFFF",
                              border: "1px solid #DC2626",
                              padding: "4px 9px",
                              borderRadius: "6px",
                            }}
                          >
                            + Registrar
                          </button>
                        ) : c.moneda_original !== moneda_base ? (
                          <button
                            onClick={() => onIrATasas(c.moneda_original, moneda_base)}
                            style={{ fontSize: "11.5px", fontWeight: 600, color: "var(--emerald-primary)" }}
                          >
                            Historial
                          </button>
                        ) : (
                          <span style={{ color: "var(--text-muted)", fontSize: "12px" }}>—</span>
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
    </div>
  );
}
