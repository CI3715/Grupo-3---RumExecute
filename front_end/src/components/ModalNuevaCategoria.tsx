"use client";

import React, { useState, useEffect } from "react";

export interface EntradaNuevaCategoriaForm {
  nombre: string;
  tipo: "gasto" | "ingreso";
}

export interface ErrorValidacion {
  campo: string;
  mensaje: string;
}

interface ModalNuevaCategoriaProps {
  isOpen: boolean;
  onClose: () => void;
  tipoInicial?: "gasto" | "ingreso";
  onSubmit: (entrada: EntradaNuevaCategoriaForm) => Promise<boolean>;
  errorExterno?: ErrorValidacion | null;
}

export default function ModalNuevaCategoria({
  isOpen,
  onClose,
  tipoInicial = "gasto",
  onSubmit,
  errorExterno,
}: ModalNuevaCategoriaProps) {
  const [tipo, setTipo] = useState<"gasto" | "ingreso">(tipoInicial);
  const [nombre, setNombre] = useState("");
  const [cargando, setCargando] = useState(false);
  const [errorActivo, setErrorActivo] = useState<ErrorValidacion | null>(null);

  useEffect(() => {
    if (isOpen) {
      setTipo(tipoInicial);
      setNombre("");
      setErrorActivo(null);
    }
  }, [isOpen, tipoInicial]);

  useEffect(() => {
    if (errorExterno) {
      setErrorActivo(errorExterno);
    }
  }, [errorExterno]);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorActivo(null);

    const nombreLimpio = nombre.trim();
    if (!nombreLimpio) {
      return setErrorActivo({
        campo: "categoria",
        mensaje: "El nombre de la categoría no puede estar vacío.",
      });
    }

    setCargando(true);
    try {
      const exito = await onSubmit({
        nombre: nombreLimpio,
        tipo,
      });
      if (exito) {
        onClose();
      }
    } catch (err: any) {
      setErrorActivo({
        campo: err?.campo || "categoria",
        mensaje:
          err?.mensaje ||
          (typeof err === "string" ? err : "Error al crear la categoría."),
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
            <h3 className="modal-title">Crear Nueva Categoría</h3>
            <p className="modal-subtitle">
              Agrega una categoría personalizada para tus movimientos
            </p>
          </div>
          <button onClick={onClose} className="btn-close" title="Cerrar">
            ✕
          </button>
        </div>

        <div className="tab-grid" style={{ marginBottom: "1.25rem" }}>
          <button
            type="button"
            onClick={() => {
              setTipo("gasto");
              setErrorActivo(null);
            }}
            className={`tab-btn ${tipo === "gasto" ? "active-expense" : ""}`}
          >
            📉 Categoría de Gasto
          </button>
          <button
            type="button"
            onClick={() => {
              setTipo("ingreso");
              setErrorActivo(null);
            }}
            className={`tab-btn ${tipo === "ingreso" ? "active-income" : ""}`}
          >
            📈 Categoría de Ingreso
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            {errorActivo?.campo === "general" && (
              <div className="alert-box">{errorActivo.mensaje}</div>
            )}

            <div className="form-group">
              <label className="form-label">Nombre de la Categoría</label>
              <input
                type="text"
                value={nombre}
                onChange={(e) => {
                  setNombre(e.target.value);
                  if (errorActivo?.campo === "categoria") setErrorActivo(null);
                }}
                placeholder={
                  tipo === "gasto"
                    ? "Ej. Gimnasio, Cursos online, Mascotas"
                    : "Ej. Freelance, Inversiones, Bono"
                }
                className={`form-input ${
                  errorActivo?.campo === "categoria" ? "input-error" : ""
                }`}
                disabled={cargando}
                autoFocus
              />
              <span className="helper-text">
                No puede haber dos categorías de {tipo} con el mismo nombre.
              </span>
              {errorActivo?.campo === "categoria" && (
                <span className="error-text">{errorActivo.mensaje}</span>
              )}
            </div>
          </div>

          <div className="modal-footer">
            <button
              type="button"
              onClick={onClose}
              className="btn btn-secondary"
              disabled={cargando}
            >
              Cancelar
            </button>
            <button
              type="submit"
              className={`btn ${
                tipo === "gasto" ? "btn-danger" : "btn-primary"
              }`}
              disabled={cargando}
            >
              {cargando ? "Guardando..." : "Crear Categoría"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
