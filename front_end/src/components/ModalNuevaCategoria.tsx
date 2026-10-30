"use client";

import React, { useState, useEffect } from "react";
import SelectorEmoji from "./SelectorEmoji";

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
  const [emoji, setEmoji] = useState(tipoInicial === "gasto" ? "🛒" : "💼");
  const [cargando, setCargando] = useState(false);
  const [errorActivo, setErrorActivo] = useState<ErrorValidacion | null>(null);

  useEffect(() => {
    if (isOpen) {
      setTipo(tipoInicial);
      setNombre("");
      setEmoji(tipoInicial === "gasto" ? "🛒" : "💼");
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

    // Si seleccionó un emoji y el texto aún no lo tiene al inicio, adjuntarlo
    const tieneEmojiAlInicio = /\p{Extended_Pictographic}/u.test(nombreLimpio.slice(0, 4));
    const nombreFinal = (emoji && !tieneEmojiAlInicio) ? `${emoji} ${nombreLimpio}` : nombreLimpio;

    setCargando(true);
    try {
      const exito = await onSubmit({
        nombre: nombreFinal,
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
              setEmoji("🛒");
              setErrorActivo(null);
            }}
            className={`tab-btn ${tipo === "gasto" ? "active-expense" : ""}`}
          >
            Categoría de Gasto
          </button>
          <button
            type="button"
            onClick={() => {
              setTipo("ingreso");
              setEmoji("💼");
              setErrorActivo(null);
            }}
            className={`tab-btn ${tipo === "ingreso" ? "active-income" : ""}`}
          >
            Categoría de Ingreso
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="modal-body">
            {errorActivo?.campo === "general" && (
              <div className="alert-box">{errorActivo.mensaje}</div>
            )}

            <div className="form-group">
              <label className="form-label">Icono y Nombre de la Categoría</label>
              <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
                <SelectorEmoji
                  emojiSeleccionado={emoji}
                  onSeleccionarEmoji={setEmoji}
                  tipo={tipo}
                />
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
                  style={{ flex: 1 }}
                />
              </div>
              <span className="helper-text">
                Puedes elegir un icono descriptivo y escribir el nombre de tu categoría.
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
