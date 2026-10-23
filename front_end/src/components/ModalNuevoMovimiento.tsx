"use client";

import React, { useState, useEffect } from "react";
import {
  CATEGORIAS_GASTO,
  CATEGORIAS_INGRESO,
  ICONOS_CATEGORIAS,
  SIMBOLOS_MONEDA,
  formatearMonto,
  normalizarMontoInput,
  completarDecimalesOnBlur,
} from "../utils/formato";

export interface EntradaMovimientoForm {
  id_cuenta: string;
  tipo: "gasto" | "ingreso";
  monto: string;
  fecha: string;
  categoria: string;
  nota: string;
}

export interface ErrorValidacion {
  campo: string;
  mensaje: string;
}

interface ModalNuevoMovimientoProps {
  isOpen: boolean;
  onClose: () => void;
  cuenta: {
    id: string;
    nombre: string;
    moneda: string;
    saldo_actual_num: number;
  };
  tipoInicial?: "gasto" | "ingreso";
  onSubmit: (entrada: EntradaMovimientoForm) => Promise<boolean>;
  errorExterno?: ErrorValidacion | null;
}

export default function ModalNuevoMovimiento({
  isOpen,
  onClose,
  cuenta,
  tipoInicial = "gasto",
  onSubmit,
  errorExterno,
}: ModalNuevoMovimientoProps) {
  const [tipo, setTipo] = useState<"gasto" | "ingreso">(tipoInicial);
  const [monto, setMonto] = useState("");
  const [fecha, setFecha] = useState("");
  const [categoria, setCategoria] = useState("");
  const [categoriaPersonalizada, setCategoriaPersonalizada] = useState("");
  const [esNuevaCategoria, setEsNuevaCategoria] = useState(false);
  const [nota, setNota] = useState("");
  const [confirmaNegativo, setConfirmaNegativo] = useState(false);
  const [cargando, setCargando] = useState(false);
  const [errorActivo, setErrorActivo] = useState<ErrorValidacion | null>(null);

  useEffect(() => {
    if (isOpen) {
      setFecha(new Date().toISOString().split("T")[0]);
      setTipo(tipoInicial);
      setMonto("");
      setCategoria((tipoInicial === "gasto" ? CATEGORIAS_GASTO : CATEGORIAS_INGRESO)[0]);
      setCategoriaPersonalizada("");
      setEsNuevaCategoria(false);
      setNota("");
      setConfirmaNegativo(false);
      setErrorActivo(null);
    }
  }, [isOpen, tipoInicial]);

  useEffect(() => {
    if (errorExterno) setErrorActivo(errorExterno);
  }, [errorExterno]);

  if (!isOpen) return null;

  const handleCambiarTipo = (nuevoTipo: "gasto" | "ingreso") => {
    setTipo(nuevoTipo);
    setEsNuevaCategoria(false);
    setCategoriaPersonalizada("");
    setCategoria((nuevoTipo === "gasto" ? CATEGORIAS_GASTO : CATEGORIAS_INGRESO)[0]);
    setErrorActivo(null);
  };

  const montoNum = parseFloat(monto.trim().replace(",", ".")) || 0;
  const saldoProyectado =
    tipo === "gasto" ? cuenta.saldo_actual_num - montoNum : cuenta.saldo_actual_num + montoNum;
  const dejaraSaldoNegativo = tipo === "gasto" && montoNum > 0 && saldoProyectado < 0;
  const tieneError = (campo: string) => errorActivo?.campo === campo;
  const categoriasActuales = tipo === "gasto" ? CATEGORIAS_GASTO : CATEGORIAS_INGRESO;
  const simbolo = SIMBOLOS_MONEDA[cuenta.moneda] || cuenta.moneda;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorActivo(null);

    const montoFinal = completarDecimalesOnBlur(monto);
    const montoFinalNum = Number(montoFinal.replace(",", "."));

    if (isNaN(montoFinalNum) || montoFinalNum <= 0) {
      return setErrorActivo({ campo: "monto", mensaje: "El monto debe ser mayor que 0." });
    }
    const hoy = new Date().toISOString().split("T")[0];
    if (!fecha) {
      return setErrorActivo({ campo: "fecha", mensaje: "Selecciona una fecha válida." });
    }
    if (fecha > hoy) {
      return setErrorActivo({
        campo: "fecha",
        mensaje: `No se permiten movimientos con fechas futuras (${fecha} es posterior a hoy ${hoy}).`,
      });
    }
    if (esNuevaCategoria && !categoriaPersonalizada.trim()) {
      return setErrorActivo({ campo: "categoria", mensaje: "Ingresa el nombre de la categoría." });
    }
    if (dejaraSaldoNegativo && !confirmaNegativo) {
      return setErrorActivo({
        campo: "general",
        mensaje: "Confirma la advertencia de saldo negativo antes de continuar.",
      });
    }

    setCargando(true);
    try {
      const exito = await onSubmit({
        id_cuenta: cuenta.id,
        tipo,
        monto: montoFinal.trim(),
        fecha,
        categoria: esNuevaCategoria ? categoriaPersonalizada.trim() : categoria,
        nota: nota.trim(),
      });
      if (exito) onClose();
    } catch (err: any) {
      setErrorActivo({
        campo: err?.campo || "fecha",
        mensaje: err?.mensaje || (typeof err === "string" ? err : "Error al registrar el movimiento."),
      });
    } finally {
      setCargando(false);
    }
  };

  return (
    <div
      className="modal-overlay"
      onMouseDown={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className="modal-card">
        <div className="modal-header">
          <div>
            <h3 className="modal-title">
              {tipo === "gasto" ? "Registrar Gasto" : "Registrar Ingreso"}
            </h3>
            <p className="modal-subtitle">
              Cuenta: <strong>{cuenta.nombre}</strong> · Moneda: {cuenta.moneda}
            </p>
          </div>
          <button onClick={onClose} className="btn-close" title="Cerrar">✕</button>
        </div>

        <div className="tab-grid">
          <button
            type="button"
            onClick={() => handleCambiarTipo("gasto")}
            className={`tab-btn ${tipo === "gasto" ? "active-expense" : ""}`}
          >
            📉 Gasto (Salida)
          </button>
          <button
            type="button"
            onClick={() => handleCambiarTipo("ingreso")}
            className={`tab-btn ${tipo === "ingreso" ? "active-income" : ""}`}
          >
            📈 Ingreso (Entrada)
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            {tieneError("general") && (
              <div className="alert-box">{errorActivo?.mensaje}</div>
            )}

            <div className="form-group">
              <label className="form-label">
                Monto del {tipo === "gasto" ? "Gasto" : "Ingreso"}
              </label>
              <div className="input-addon-group">
                <span className="input-addon">{simbolo}</span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={monto}
                  onChange={(e) => {
                    setMonto(normalizarMontoInput(e.target.value));
                    if (tieneError("monto")) setErrorActivo(null);
                  }}
                  onFocus={(e) => e.target.select()}
                  onBlur={() => setMonto((prev) => (prev ? completarDecimalesOnBlur(prev) : ""))}
                  placeholder="0,00"
                  className={`form-input ${tieneError("monto") ? "input-error" : ""}`}
                  disabled={cargando}
                  autoFocus
                />
              </div>
              <span className="helper-text">
                Escribe el monto directamente (ej. 50 o 1250,50)
              </span>
              {tieneError("monto") && <span className="error-text">{errorActivo?.mensaje}</span>}
            </div>

            {dejaraSaldoNegativo && (
              <div className="warning-box">
                <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                  <span>⚠️</span>
                  <strong>Atención: Esta cuenta quedará en saldo negativo</strong>
                </div>
                <p>
                  Este gasto dejará la cuenta en{" "}
                  <strong>{formatearMonto(saldoProyectado, cuenta.moneda)}</strong> (saldo
                  actual: {formatearMonto(cuenta.saldo_actual_num, cuenta.moneda)}).
                </p>
                <label style={{ display: "flex", alignItems: "center", gap: "8px", fontWeight: 600, cursor: "pointer" }}>
                  <input
                    type="checkbox"
                    checked={confirmaNegativo}
                    onChange={(e) => setConfirmaNegativo(e.target.checked)}
                  />
                  <span>Entiendo y deseo registrar el gasto de todas formas</span>
                </label>
              </div>
            )}

            <div className="form-group">
              <label className="form-label">Fecha</label>
              <input
                type="date"
                max={new Date().toISOString().split("T")[0]}
                value={fecha}
                onChange={(e) => {
                  setFecha(e.target.value);
                  if (tieneError("fecha")) setErrorActivo(null);
                }}
                className={`form-input ${tieneError("fecha") ? "input-error" : ""}`}
                disabled={cargando}
              />
              {tieneError("fecha") && <span className="error-text">{errorActivo?.mensaje}</span>}
            </div>

            <div className="form-group">
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <label className="form-label">Categoría</label>
                <button
                  type="button"
                  onClick={() => setEsNuevaCategoria(!esNuevaCategoria)}
                  className="btn-link"
                >
                  {esNuevaCategoria ? "← Seleccionar de la lista" : "+ Crear nueva categoría"}
                </button>
              </div>

              {esNuevaCategoria ? (
                <input
                  type="text"
                  value={categoriaPersonalizada}
                  onChange={(e) => {
                    setCategoriaPersonalizada(e.target.value);
                    if (tieneError("categoria")) setErrorActivo(null);
                  }}
                  placeholder="Ej. Cursos online, Freelance"
                  className={`form-input ${tieneError("categoria") ? "input-error" : ""}`}
                  disabled={cargando}
                />
              ) : (
                <select
                  value={categoria}
                  onChange={(e) => {
                    setCategoria(e.target.value);
                    if (tieneError("categoria")) setErrorActivo(null);
                  }}
                  className={`form-select ${tieneError("categoria") ? "input-error" : ""}`}
                  disabled={cargando}
                >
                  {categoriasActuales.map((cat) => (
                    <option key={cat} value={cat}>
                      {ICONOS_CATEGORIAS[cat] || "🏷️"} {cat}
                    </option>
                  ))}
                </select>
              )}
              {tieneError("categoria") && (
                <span className="error-text">{errorActivo?.mensaje}</span>
              )}
            </div>

            <div className="form-group">
              <label className="form-label">Nota / Concepto (Opcional)</label>
              <input
                type="text"
                value={nota}
                onChange={(e) => setNota(e.target.value)}
                placeholder="Ej. Compra en supermercado, Pago quincenal"
                className="form-input"
                disabled={cargando}
              />
            </div>
          </div>

          <div className="modal-footer">
            <button type="button" onClick={onClose} className="btn btn-secondary" disabled={cargando}>
              Cancelar
            </button>
            <button
              type="submit"
              className={`btn ${tipo === "gasto" ? "btn-danger" : "btn-primary"}`}
              disabled={cargando}
            >
              {cargando ? "Guardando..." : tipo === "gasto" ? "Guardar Gasto" : "Guardar Ingreso"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
