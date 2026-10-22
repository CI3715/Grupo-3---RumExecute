"use client";

import React, { useState, useEffect } from "react";
import {
  SIMBOLOS_MONEDA,
  normalizarMontoInput,
  completarDecimalesOnBlur,
} from "../utils/formato";

export interface MovimientoItem {
  id: string;
  id_cuenta: string;
  tipo: "gasto" | "ingreso" | "ajuste";
  monto_num: number;
  monto_formateado: string;
  fecha: string;
  categoria: string;
  nota?: string;
  estado: "vigente" | "corregido" | "ajuste";
  id_movimiento_referenciado?: string;
}

export interface EntradaCorreccionForm {
  id_movimiento_original: string;
  monto_correcto: string;
  motivo?: string;
}

export interface ErrorValidacion {
  campo: string;
  mensaje: string;
}

interface ModalCorregirMovimientoProps {
  isOpen: boolean;
  onClose: () => void;
  movimiento: MovimientoItem | null;
  moneda: string;
  onSubmit: (entrada: EntradaCorreccionForm) => Promise<boolean>;
  errorExterno?: ErrorValidacion | null;
}

export default function ModalCorregirMovimiento({
  isOpen,
  onClose,
  movimiento,
  moneda,
  onSubmit,
  errorExterno,
}: ModalCorregirMovimientoProps) {
  const [montoCorrecto, setMontoCorrecto] = useState("");
  const [motivo, setMotivo] = useState("");
  const [cargando, setCargando] = useState(false);
  const [errorActivo, setErrorActivo] = useState<ErrorValidacion | null>(null);

  useEffect(() => {
    if (isOpen && movimiento) {
      setMontoCorrecto("");
      setMotivo("");
      setErrorActivo(null);
    }
  }, [isOpen, movimiento]);

  useEffect(() => {
    if (errorExterno) setErrorActivo(errorExterno);
  }, [errorExterno]);

  if (!isOpen || !movimiento) return null;

  const simbolo = SIMBOLOS_MONEDA[moneda] || moneda;
  const montoCorrectoNum = parseFloat(montoCorrecto.trim().replace(",", ".")) || 0;
  const diferenciaAjuste =
    movimiento.tipo === "gasto"
      ? movimiento.monto_num - montoCorrectoNum
      : montoCorrectoNum - movimiento.monto_num;

  const tieneError = (campo: string) => errorActivo?.campo === campo;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorActivo(null);

    const montoFinal = completarDecimalesOnBlur(montoCorrecto);
    const num = Number(montoFinal.replace(",", "."));

    if (isNaN(num) || num < 0) {
      return setErrorActivo({
        campo: "monto_correcto",
        mensaje: "El monto correcto debe ser mayor o igual a 0.",
      });
    }
    if (num === movimiento.monto_num) {
      return setErrorActivo({
        campo: "monto_correcto",
        mensaje: "El monto correcto no puede ser idéntico al monto original.",
      });
    }

    setCargando(true);
    try {
      const exito = await onSubmit({
        id_movimiento_original: movimiento.id,
        monto_correcto: montoFinal.trim(),
        motivo: motivo.trim(),
      });
      if (exito) onClose();
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
            <h3 className="modal-title">Corregir Movimiento</h3>
            <p className="modal-subtitle">
              Corrige el monto ingresado creando un ajuste en tu saldo
            </p>
          </div>
          <button onClick={onClose} className="btn-close" title="Cerrar">✕</button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            {tieneError("general") && (
              <div className="alert-box">{errorActivo?.mensaje}</div>
            )}

            <div className="equation-bar" style={{ flexDirection: "column", alignItems: "stretch", gap: "6px" }}>
              <div style={{ display: "flex", justifyContent: "space-between", fontSize: "11.5px", color: "var(--text-secondary)" }}>
                <strong>MOVIMIENTO ORIGINAL</strong>
                <span>{movimiento.fecha}</span>
              </div>
              <div style={{ fontSize: "13px" }}>
                Categoría: <strong>{movimiento.categoria}</strong>
                {movimiento.nota ? ` · "${movimiento.nota}"` : ""}
              </div>
              <div style={{ fontSize: "13.5px" }}>
                Monto registrado: <strong>{movimiento.monto_formateado}</strong>
              </div>
            </div>

            <div className="info-box">
              <span>ℹ️</span>
              <p>
                <strong>El registro original se mantendrá en tu historial</strong>. Se
                marcará como <em>"Corregido"</em> y se registrará un movimiento de ajuste por
                la diferencia en tu saldo.
              </p>
            </div>

            <div className="form-group">
              <label className="form-label">Nuevo monto correcto real</label>
              <div className="input-addon-group">
                <span className="input-addon">{simbolo}</span>
                <input
                  type="text"
                  inputMode="decimal"
                  value={montoCorrecto}
                  onChange={(e) => {
                    setMontoCorrecto(normalizarMontoInput(e.target.value));
                    if (tieneError("monto_correcto")) setErrorActivo(null);
                  }}
                  onFocus={(e) => e.target.select()}
                  onBlur={() =>
                    setMontoCorrecto((prev) => (prev ? completarDecimalesOnBlur(prev) : ""))
                  }
                  placeholder="0,00"
                  className={`form-input ${tieneError("monto_correcto") ? "input-error" : ""}`}
                  disabled={cargando}
                  autoFocus
                />
              </div>
              <span className="helper-text">
                Si fue una compra duplicada que no debió existir, escribe <strong>0</strong>.
              </span>
              {tieneError("monto_correcto") && (
                <span className="error-text">{errorActivo?.mensaje}</span>
              )}
            </div>

            {montoCorrecto.trim() !== "" && !isNaN(montoCorrectoNum) && (
              <div className="equation-bar">
                <span style={{ fontSize: "13px", color: "var(--text-secondary)" }}>
                  Ajuste que se registrará:
                </span>
                <strong
                  style={{
                    fontSize: "14px",
                    color: diferenciaAjuste >= 0 ? "var(--emerald-primary)" : "var(--error-color)",
                  }}
                >
                  {diferenciaAjuste >= 0 ? "+ " : "- "}
                  {simbolo} {Math.abs(diferenciaAjuste).toFixed(2).replace(".", ",")}
                </strong>
              </div>
            )}

            <div className="form-group">
              <label className="form-label">Motivo de la corrección (Opcional)</label>
              <input
                type="text"
                value={motivo}
                onChange={(e) => setMotivo(e.target.value)}
                placeholder="Ej. Error al escribir el monto, cargo duplicado"
                className="form-input"
                disabled={cargando}
              />
            </div>
          </div>

          <div className="modal-footer">
            <button type="button" onClick={onClose} className="btn btn-secondary" disabled={cargando}>
              Cancelar
            </button>
            <button type="submit" className="btn btn-blue" disabled={cargando}>
              {cargando ? "Guardando..." : "Registrar Ajuste"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
