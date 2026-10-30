"use client";

import { useState } from "react";
import {
  TasaCambioItem,
  MonedaItem,
  EntradaRegistroTasa,
  EntradaCrearMoneda,
} from "../types/tasas";
import { obtenerFechaHoyISO } from "../services/tasasService";

interface VistaTasasProps {
  tasas: TasaCambioItem[];
  monedas: MonedaItem[];
  filtroPar?: { origen?: string; destino?: string } | null;
  onRegistrarTasa: (entrada: EntradaRegistroTasa) => Promise<void>;
  onEliminarTasa: (idTasa: string) => Promise<void>;
  onCrearMoneda: (entrada: EntradaCrearMoneda) => Promise<void>;
}

export default function VistaTasas({
  tasas,
  monedas,
  filtroPar,
  onRegistrarTasa,
  onEliminarTasa,
  onCrearMoneda,
}: VistaTasasProps) {
  const hoy = obtenerFechaHoyISO();

  // Acordeón / Formularios integrados (sin exceso de modales)
  const [panelAbierto, setPanelAbierto] = useState<"ninguno" | "nueva_tasa" | "nueva_moneda">(
    filtroPar ? "nueva_tasa" : "ninguno"
  );

  // Filtros de tabla
  const [filtroMoneda, setFiltroMoneda] = useState<string>(filtroPar?.origen || "TODOS");
  const [eliminandoId, setEliminandoId] = useState<string | null>(null);

  // Formulario Nueva Tasa
  const [formTasa, setFormTasa] = useState<{
    origen: string;
    destino: string;
    valor: string;
    fecha_vigencia: string;
    fuente: string;
    error: string | null;
    guardando: boolean;
  }>({
    origen: filtroPar?.origen || "VES",
    destino: filtroPar?.destino || "USD",
    valor: "",
    fecha_vigencia: hoy,
    fuente: "",
    error: null,
    guardando: false,
  });

  // Formulario Nueva Moneda
  const [formMoneda, setFormMoneda] = useState<{
    codigo: string;
    nombre: string;
    simbolo: string;
    decimales: number;
    error: string | null;
    guardando: boolean;
  }>({
    codigo: "",
    nombre: "",
    simbolo: "",
    decimales: 2,
    error: null,
    guardando: false,
  });

  // Envío Formulario Nueva Tasa
  const handleGuardarTasa = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormTasa((prev) => ({ ...prev, error: null }));

    if (formTasa.origen === formTasa.destino) {
      setFormTasa((prev) => ({ ...prev, error: "Las monedas origen y destino deben ser distintas." }));
      return;
    }

    const valLimpio = formTasa.valor.trim().replace(",", ".");
    const num = parseFloat(valLimpio);
    if (isNaN(num) || num <= 0) {
      setFormTasa((prev) => ({ ...prev, error: "El valor debe ser un número estrictamente mayor a 0." }));
      return;
    }

    if (formTasa.fecha_vigencia > hoy) {
      setFormTasa((prev) => ({
        ...prev,
        error: `La fecha de vigencia (${formTasa.fecha_vigencia}) no puede ser futura.`,
      }));
      return;
    }

    try {
      setFormTasa((prev) => ({ ...prev, guardando: true }));
      await onRegistrarTasa({
        origen: formTasa.origen,
        destino: formTasa.destino,
        valor: valLimpio,
        fecha_vigencia: formTasa.fecha_vigencia,
        fuente: formTasa.fuente.trim() || undefined,
      });
      setFormTasa({
        origen: "VES",
        destino: "USD",
        valor: "",
        fecha_vigencia: hoy,
        fuente: "",
        error: null,
        guardando: false,
      });
      setPanelAbierto("ninguno");
    } catch (err: any) {
      setFormTasa((prev) => ({
        ...prev,
        guardando: false,
        error: err?.message || "Error al registrar la tasa.",
      }));
    }
  };

  // Envío Formulario Nueva Moneda
  const handleGuardarMoneda = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormMoneda((prev) => ({ ...prev, error: null }));

    const codUpper = formMoneda.codigo.trim().toUpperCase();
    if (!/^[A-Z]{3,5}$/.test(codUpper)) {
      setFormMoneda((prev) => ({
        ...prev,
        error: "El código debe tener entre 3 y 5 letras mayúsculas (A-Z).",
      }));
      return;
    }

    if (["VES", "USD", "EUR", "USDT"].includes(codUpper)) {
      setFormMoneda((prev) => ({
        ...prev,
        error: `La moneda ${codUpper} es una de las 4 monedas de fábrica y está protegida.`,
      }));
      return;
    }

    if (!formMoneda.nombre.trim()) {
      setFormMoneda((prev) => ({ ...prev, error: "El nombre es obligatorio." }));
      return;
    }

    if (!formMoneda.simbolo.trim()) {
      setFormMoneda((prev) => ({ ...prev, error: "El símbolo es obligatorio." }));
      return;
    }

    try {
      setFormMoneda((prev) => ({ ...prev, guardando: true }));
      await onCrearMoneda({
        codigo: codUpper,
        nombre: formMoneda.nombre.trim(),
        simbolo: formMoneda.simbolo.trim(),
        decimales: Number(formMoneda.decimales),
      });
      setFormMoneda({
        codigo: "",
        nombre: "",
        simbolo: "",
        decimales: 2,
        error: null,
        guardando: false,
      });
      setPanelAbierto("ninguno");
    } catch (err: any) {
      setFormMoneda((prev) => ({
        ...prev,
        guardando: false,
        error: err?.message || "Error al crear la moneda.",
      }));
    }
  };

  const handleEliminarTasaConfirm = async (t: TasaCambioItem) => {
    const seguro = window.confirm(
      `¿Deseas eliminar la tasa ${t.origen} → ${t.destino} de ${t.valor} con fecha ${t.fecha_vigencia}?`
    );
    if (!seguro) return;

    try {
      setEliminandoId(t.id);
      await onEliminarTasa(t.id);
    } finally {
      setEliminandoId(null);
    }
  };

  // Filtrado de tasas
  const tasasFiltradas = tasas.filter((t) => {
    if (filtroMoneda === "TODOS") return true;
    return t.origen === filtroMoneda || t.destino === filtroMoneda;
  });

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
      {/* 1. Header */}
      <div className="hero-section">
        <div>
          <h1 className="hero-title">Tasas de Cambio e Historial</h1>
          <p className="hero-subtitle">
            Historial de cotizaciones registradas y cálculo automático de tasas inversas.
          </p>
        </div>

        <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
          <button
            onClick={() => setPanelAbierto(panelAbierto === "nueva_moneda" ? "ninguno" : "nueva_moneda")}
            style={{
              backgroundColor: panelAbierto === "nueva_moneda" ? "var(--surface-hover)" : "#FFFFFF",
              color: "var(--text-primary)",
              border: "1px solid var(--border-subtle)",
              padding: "9px 16px",
              borderRadius: "8px",
              fontSize: "13px",
              fontWeight: 600,
            }}
          >
            {panelAbierto === "nueva_moneda" ? "Cerrar Moneda" : "+ Moneda Propia"}
          </button>

          <button
            onClick={() => setPanelAbierto(panelAbierto === "nueva_tasa" ? "ninguno" : "nueva_tasa")}
            className="btn btn-primary"
            style={{ fontSize: "13px", padding: "9px 18px" }}
          >
            {panelAbierto === "nueva_tasa" ? "Cerrar Formulario" : "+ Registrar Tasa"}
          </button>
        </div>
      </div>

      {/* 2. Formulario Integrado: Registrar Nueva Tasa */}
      {panelAbierto === "nueva_tasa" && (
        <div
          style={{
            backgroundColor: "var(--surface-card)",
            border: "2px solid var(--emerald-primary)",
            borderRadius: "14px",
            padding: "24px 28px",
            boxShadow: "0 4px 15px rgba(5, 150, 105, 0.08)",
          }}
        >
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
            <div>
              <h2 style={{ fontSize: "16px", fontWeight: 800, color: "var(--text-primary)" }}>
                Registrar Nueva Tasa de Cambio
              </h2>
              <p style={{ fontSize: "12px", color: "var(--text-secondary)", marginTop: "2px" }}>
                Las conversiones inversas se calculan automáticamente (1/valor).
              </p>
            </div>
            <button
              onClick={() => setPanelAbierto("ninguno")}
              style={{ fontSize: "16px", color: "var(--text-secondary)" }}
            >
              ✕
            </button>
          </div>

          {formTasa.error && (
            <div style={{ backgroundColor: "#FEF2F2", color: "#DC2626", padding: "10px 14px", borderRadius: "8px", fontSize: "12.5px", fontWeight: 600, marginBottom: "16px" }}>
              {formTasa.error}
            </div>
          )}

          <form onSubmit={handleGuardarTasa} style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            <div style={{ display: "grid", gridTemplateColumns: "1fr auto 1fr", gap: "12px", alignItems: "center" }}>
              <div>
                <label style={{ display: "block", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                  Moneda Origen
                </label>
                <select
                  value={formTasa.origen}
                  onChange={(e) => setFormTasa((prev) => ({ ...prev, origen: e.target.value }))}
                  style={{ width: "100%", padding: "8px 12px", border: "1px solid var(--border-subtle)", borderRadius: "8px", fontSize: "13px", fontWeight: 700 }}
                >
                  {monedas.map((m) => (
                    <option key={`orig-${m.codigo}`} value={m.codigo}>
                      {m.codigo} — {m.nombre} ({m.simbolo})
                    </option>
                  ))}
                </select>
              </div>

              <div style={{ fontSize: "20px", color: "var(--text-secondary)", marginTop: "18px", textAlign: "center" }}>
                ➔
              </div>

              <div>
                <label style={{ display: "block", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                  Moneda Destino
                </label>
                <select
                  value={formTasa.destino}
                  onChange={(e) => setFormTasa((prev) => ({ ...prev, destino: e.target.value }))}
                  style={{ width: "100%", padding: "8px 12px", border: "1px solid var(--border-subtle)", borderRadius: "8px", fontSize: "13px", fontWeight: 700 }}
                >
                  {monedas.map((m) => (
                    <option key={`dest-${m.codigo}`} value={m.codigo}>
                      {m.codigo} — {m.nombre} ({m.simbolo})
                    </option>
                  ))}
                </select>
              </div>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px" }}>
              <div>
                <label style={{ display: "block", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                  Valor (1 {formTasa.origen} = ¿cuántos {formTasa.destino}?)
                </label>
                <input
                  type="text"
                  placeholder="Ej. 0.025 o 40.00"
                  value={formTasa.valor}
                  onChange={(e) => setFormTasa((prev) => ({ ...prev, valor: e.target.value }))}
                  style={{ width: "100%", padding: "9px 12px", border: "1px solid var(--border-subtle)", borderRadius: "8px", fontSize: "13.5px", fontWeight: 600 }}
                  required
                />
              </div>

              <div>
                <label style={{ display: "block", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                  Fecha de Vigencia
                </label>
                <input
                  type="date"
                  max={hoy}
                  value={formTasa.fecha_vigencia}
                  onChange={(e) => setFormTasa((prev) => ({ ...prev, fecha_vigencia: e.target.value }))}
                  style={{ width: "100%", padding: "9px 12px", border: "1px solid var(--border-subtle)", borderRadius: "8px", fontSize: "13px" }}
                  required
                />
              </div>
            </div>

            <div>
              <label style={{ display: "block", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                Fuente o Referencia (Opcional)
              </label>
              <input
                type="text"
                placeholder="Ej. BCV Oficial, DolarToday, Binance P2P, Registro personal"
                value={formTasa.fuente}
                onChange={(e) => setFormTasa((prev) => ({ ...prev, fuente: e.target.value }))}
                style={{ width: "100%", padding: "8px 12px", border: "1px solid var(--border-subtle)", borderRadius: "8px", fontSize: "13px" }}
              />
            </div>

            <div style={{ display: "flex", justifyContent: "flex-end", gap: "10px", marginTop: "8px" }}>
              <button
                type="button"
                onClick={() => setPanelAbierto("ninguno")}
                className="btn btn-secondary"
                style={{ padding: "8px 16px", fontSize: "13px" }}
              >
                Cancelar
              </button>
              <button
                type="submit"
                disabled={formTasa.guardando}
                className="btn btn-primary"
                style={{ padding: "8px 20px", fontSize: "13px" }}
              >
                {formTasa.guardando ? "Registrando..." : "Guardar Tasa"}
              </button>
            </div>
          </form>
        </div>
      )}

      {/* 3. Formulario Integrado: Crear Moneda Propia */}
      {panelAbierto === "nueva_moneda" && (
        <div
          style={{
            backgroundColor: "var(--surface-card)",
            border: "2px solid #2563EB",
            borderRadius: "14px",
            padding: "24px 28px",
            boxShadow: "0 4px 15px rgba(37, 99, 235, 0.08)",
          }}
        >
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
            <div>
              <h2 style={{ fontSize: "16px", fontWeight: 800, color: "var(--text-primary)" }}>
                Crear Moneda Personalizada
              </h2>
              <p style={{ fontSize: "12px", color: "var(--text-secondary)", marginTop: "2px" }}>
                Extiende el catálogo del sistema para tus cuentas multimoneda.
              </p>
            </div>
            <button
              onClick={() => setPanelAbierto("ninguno")}
              style={{ fontSize: "16px", color: "var(--text-secondary)" }}
            >
              ✕
            </button>
          </div>

          {formMoneda.error && (
            <div style={{ backgroundColor: "#FEF2F2", color: "#DC2626", padding: "10px 14px", borderRadius: "8px", fontSize: "12.5px", fontWeight: 600, marginBottom: "16px" }}>
              {formMoneda.error}
            </div>
          )}

          <form onSubmit={handleGuardarMoneda} style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 2fr", gap: "14px" }}>
              <div>
                <label style={{ display: "block", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                  Código (3-5 letras mayúsculas)
                </label>
                <input
                  type="text"
                  placeholder="Ej. CAD, COP, BRL"
                  maxLength={5}
                  value={formMoneda.codigo}
                  onChange={(e) => setFormMoneda((prev) => ({ ...prev, codigo: e.target.value.toUpperCase() }))}
                  style={{ width: "100%", padding: "9px 12px", border: "1px solid var(--border-subtle)", borderRadius: "8px", fontSize: "14px", fontWeight: 700, letterSpacing: "1px" }}
                  required
                />
              </div>

              <div>
                <label style={{ display: "block", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                  Nombre Descriptivo
                </label>
                <input
                  type="text"
                  placeholder="Ej. Dólar Canadiense, Peso Colombiano"
                  value={formMoneda.nombre}
                  onChange={(e) => setFormMoneda((prev) => ({ ...prev, nombre: e.target.value }))}
                  style={{ width: "100%", padding: "9px 12px", border: "1px solid var(--border-subtle)", borderRadius: "8px", fontSize: "13px" }}
                  required
                />
              </div>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "14px" }}>
              <div>
                <label style={{ display: "block", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                  Símbolo
                </label>
                <input
                  type="text"
                  placeholder="Ej. C$, $, ₿"
                  maxLength={5}
                  value={formMoneda.simbolo}
                  onChange={(e) => setFormMoneda((prev) => ({ ...prev, simbolo: e.target.value }))}
                  style={{ width: "100%", padding: "9px 12px", border: "1px solid var(--border-subtle)", borderRadius: "8px", fontSize: "14px", fontWeight: 700 }}
                  required
                />
              </div>

              <div>
                <label style={{ display: "block", fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "4px" }}>
                  Decimales (0 a 8)
                </label>
                <input
                  type="number"
                  min={0}
                  max={8}
                  value={formMoneda.decimales}
                  onChange={(e) => setFormMoneda((prev) => ({ ...prev, decimales: parseInt(e.target.value) || 0 }))}
                  style={{ width: "100%", padding: "9px 12px", border: "1px solid var(--border-subtle)", borderRadius: "8px", fontSize: "13px", fontWeight: 700 }}
                  required
                />
              </div>
            </div>

            <div style={{ display: "flex", justifyContent: "flex-end", gap: "10px", marginTop: "8px" }}>
              <button
                type="button"
                onClick={() => setPanelAbierto("ninguno")}
                className="btn btn-secondary"
                style={{ padding: "8px 16px", fontSize: "13px" }}
              >
                Cancelar
              </button>
              <button
                type="submit"
                disabled={formMoneda.guardando}
                className="btn btn-blue"
                style={{ padding: "8px 20px", fontSize: "13px" }}
              >
                {formMoneda.guardando ? "Creando..." : "Crear Moneda"}
              </button>
            </div>
          </form>
        </div>
      )}

      {/* 4. Barra de Filtro de Historial */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexWrap: "wrap",
          gap: "14px",
          backgroundColor: "var(--surface-card)",
          padding: "14px 20px",
          borderRadius: "12px",
          border: "1px solid var(--border-subtle)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "10px", flexWrap: "wrap" }}>
          <span style={{ fontSize: "13px", fontWeight: 700, color: "var(--text-secondary)" }}>
            Filtrar historial por moneda:
          </span>
          <select
            value={filtroMoneda}
            onChange={(e) => setFiltroMoneda(e.target.value)}
            style={{
              padding: "6px 12px",
              borderRadius: "6px",
              border: "1px solid var(--border-subtle)",
              fontSize: "13px",
              fontWeight: 600,
            }}
          >
            <option value="TODOS">Todas las monedas</option>
            {monedas.map((m) => (
              <option key={`flt-${m.codigo}`} value={m.codigo}>
                {m.codigo} — {m.nombre}
              </option>
            ))}
          </select>
          {filtroMoneda !== "TODOS" && (
            <button
              onClick={() => setFiltroMoneda("TODOS")}
              style={{ fontSize: "12px", color: "var(--emerald-primary)", fontWeight: 600 }}
            >
              Quitar filtro
            </button>
          )}
        </div>

        <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
          Total registros: <strong>{tasasFiltradas.length}</strong>
        </div>
      </div>

      {/* 5. Tabla de Historial */}
      <div
        style={{
          backgroundColor: "var(--surface-card)",
          borderRadius: "14px",
          border: "1px solid var(--border-subtle)",
          overflow: "hidden",
        }}
      >
        {tasasFiltradas.length === 0 ? (
          <div style={{ padding: "48px 24px", textAlign: "center", color: "var(--text-secondary)", fontSize: "13.5px" }}>
            No hay tasas de cambio registradas con los filtros seleccionados.
          </div>
        ) : (
          <div style={{ overflowX: "auto" }}>
            <table style={{ width: "100%", borderCollapse: "collapse", textAlign: "left" }}>
              <thead>
                <tr style={{ backgroundColor: "#F8FAFC", borderBottom: "1px solid var(--border-subtle)" }}>
                  <th style={{ padding: "12px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase" }}>
                    Par de Monedas
                  </th>
                  <th style={{ padding: "12px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase" }}>
                    Tasa Registrada
                  </th>
                  <th style={{ padding: "12px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase" }}>
                    Tasa Inversa Calculada (1/x)
                  </th>
                  <th style={{ padding: "12px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase" }}>
                    Fecha Vigencia
                  </th>
                  <th style={{ padding: "12px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase" }}>
                    Fuente
                  </th>
                  <th style={{ padding: "12px 18px", fontSize: "11px", fontWeight: 700, color: "var(--text-secondary)", textTransform: "uppercase", textAlign: "center" }}>
                    Acción
                  </th>
                </tr>
              </thead>
              <tbody>
                {tasasFiltradas.map((t) => {
                  const inversa = t.valor > 0 ? 1 / t.valor : 0;
                  return (
                    <tr key={t.id} style={{ borderBottom: "1px solid var(--border-subtle)" }}>
                      <td style={{ padding: "14px 18px" }}>
                        <span style={{ fontWeight: 800, fontSize: "13.5px", color: "var(--text-primary)" }}>
                          {t.origen} ➔ {t.destino}
                        </span>
                      </td>

                      <td style={{ padding: "14px 18px", fontWeight: 700, color: "var(--emerald-primary)", fontSize: "13.5px" }}>
                        1 {t.origen} = {t.valor} {t.destino}
                      </td>

                      <td style={{ padding: "14px 18px", fontSize: "12.5px", color: "var(--text-secondary)" }}>
                        1 {t.destino} = {inversa.toFixed(6)} {t.origen}
                      </td>

                      <td style={{ padding: "14px 18px", fontSize: "13px", fontWeight: 600 }}>
                        {t.fecha_vigencia}
                      </td>

                      <td style={{ padding: "14px 18px", fontSize: "12.5px", color: "var(--text-secondary)" }}>
                        {t.fuente || "Manual"}
                      </td>

                      <td style={{ padding: "14px 18px", textAlign: "center" }}>
                        <button
                          onClick={() => handleEliminarTasaConfirm(t)}
                          disabled={eliminandoId === t.id}
                          style={{
                            fontSize: "11.5px",
                            fontWeight: 700,
                            color: "#DC2626",
                            backgroundColor: "#FEE2E2",
                            padding: "5px 10px",
                            borderRadius: "6px",
                          }}
                        >
                          {eliminandoId === t.id ? "Eliminando..." : "Eliminar"}
                        </button>
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
