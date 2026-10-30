"use client";

import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { normalizarMontoInput, completarDecimalesOnBlur } from "../utils/formato";

export interface EntradaCuentaForm {
  nombre: string;
  tipo: string;
  moneda: string;
  saldo_inicial: string;
}

export interface ErrorValidacion {
  campo: string;
  mensaje: string;
}

export interface MonedaItem {
  codigo: string;
  nombre: string;
  simbolo: string;
  decimales: number;
}

const MONEDAS_DEFAULT: MonedaItem[] = [
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
  const [nombre, setNombre] = useState("");
  const [tipo, setTipo] = useState("banco");
  const [moneda, setMoneda] = useState("VES");
  const [saldoInicial, setSaldoInicial] = useState("");
  const [cargando, setCargando] = useState(false);
  const [errorActivo, setErrorActivo] = useState<ErrorValidacion | null>(null);
  const [listaMonedas, setListaMonedas] = useState<MonedaItem[]>(MONEDAS_DEFAULT);

  useEffect(() => {
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      invoke<MonedaItem[]>("listar_monedas")
        .then((cat) => cat?.length && setListaMonedas(cat))
        .catch(() => {});
    }
  }, [isOpen]);

  useEffect(() => {
    if (errorExterno) setErrorActivo(errorExterno);
  }, [errorExterno]);

  useEffect(() => {
    if (isOpen) {
      setNombre("");
      setTipo("banco");
      setMoneda("VES");
      setSaldoInicial("");
      setErrorActivo(null);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const tieneError = (campo: string) => errorActivo?.campo === campo;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorActivo(null);
    setCargando(true);

    try {
      const exito = await onSubmit({
        nombre: nombre.trim(),
        tipo,
        moneda,
        saldo_inicial: completarDecimalesOnBlur(saldoInicial.trim() || "0"),
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
            <h3 className="modal-title">Nueva Cuenta</h3>
            <p className="modal-subtitle">Un lugar para tu dinero. Una moneda por cuenta.</p>
          </div>
          <button onClick={onClose} className="btn-close" title="Cerrar">✕</button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            {tieneError("general") && (
              <div className="alert-box">{errorActivo?.mensaje}</div>
            )}

            <div className="form-group">
              <label className="form-label">Nombre de la cuenta</label>
              <input
                type="text"
                value={nombre}
                onChange={(e) => {
                  setNombre(e.target.value);
                  if (tieneError("nombre")) setErrorActivo(null);
                }}
                placeholder="Ej. Banesco Ahorros, Cartera"
                className={`form-input ${tieneError("nombre") ? "input-error" : ""}`}
                disabled={cargando}
                autoFocus
              />
              {tieneError("nombre") && <span className="error-text">{errorActivo?.mensaje}</span>}
            </div>

            <div className="form-group">
              <label className="form-label">Tipo de cuenta</label>
              <select
                value={tipo}
                onChange={(e) => {
                  setTipo(e.target.value);
                  if (tieneError("tipo")) setErrorActivo(null);
                }}
                className={`form-select ${tieneError("tipo") ? "input-error" : ""}`}
                disabled={cargando}
              >
                <option value="banco">Banco (Cuenta de ahorros / corriente)</option>
                <option value="efectivo">Efectivo (Billetes físicos, cartera)</option>
                <option value="billetera_digital">Billetera digital (PayPal, Zinli, etc.)</option>
              </select>
              {tieneError("tipo") && <span className="error-text">{errorActivo?.mensaje}</span>}
            </div>

            <div className="form-group">
              <label className="form-label">Moneda de la cuenta</label>
              <select
                value={moneda}
                onChange={(e) => {
                  setMoneda(e.target.value);
                  if (tieneError("moneda")) setErrorActivo(null);
                }}
                className={`form-select ${tieneError("moneda") ? "input-error" : ""}`}
                disabled={cargando}
              >
                {listaMonedas.map((m) => (
                  <option key={m.codigo} value={m.codigo}>
                    {m.codigo} — {m.nombre} ({m.simbolo})
                  </option>
                ))}
              </select>
              <span className="helper-text">La moneda de la cuenta no podrá modificarse una vez creada</span>
              {tieneError("moneda") && <span className="error-text">{errorActivo?.mensaje}</span>}
            </div>

            <div className="form-group">
              <label className="form-label">Saldo inicial</label>
              <input
                type="text"
                inputMode="decimal"
                value={saldoInicial}
                onChange={(e) => {
                  setSaldoInicial(normalizarMontoInput(e.target.value));
                  if (tieneError("saldo_inicial")) setErrorActivo(null);
                }}
                onFocus={(e) => e.target.select()}
                onBlur={() => {
                  if (saldoInicial.trim()) {
                    setSaldoInicial((prev) => completarDecimalesOnBlur(prev));
                  }
                }}
                placeholder="0,00"
                className={`form-input ${tieneError("saldo_inicial") ? "input-error" : ""}`}
                disabled={cargando}
              />
              <span className="helper-text">
                Escribe el monto directamente (ej. 1250 o 50,50). Si lo dejas vacío será 0,00.
              </span>
              {tieneError("saldo_inicial") && (
                <span className="error-text">{errorActivo?.mensaje}</span>
              )}
            </div>
          </div>

          <div className="modal-footer">
            <button type="button" onClick={onClose} className="btn btn-secondary" disabled={cargando}>
              Cancelar
            </button>
            <button type="submit" className="btn btn-primary" disabled={cargando}>
              {cargando ? "Guardando..." : "Guardar Cuenta"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
