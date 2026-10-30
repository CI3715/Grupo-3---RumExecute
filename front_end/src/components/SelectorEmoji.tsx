"use client";

import { useState, useRef, useEffect } from "react";

export const EMOJIS_GASTO = [
  "🍔", "🍕", "🛒", "☕", "🍦", "🥖",
  "🚗", "🚌", "✈️", "⛽", "🚕", "🚆",
  "🏠", "💡", "💧", "🔌", "📶", "🛋️",
  "🏥", "💊", "🩺", "🏋️", "🧴", "🦷",
  "🎮", "🎬", "🎟️", "🎧", "📚", "⚽",
  "👕", "👟", "💄", "✂️", "📱", "💻",
  "🎁", "🐾", "🔧", "📦", "💳", "🧾"
];

export const EMOJIS_INGRESO = [
  "💼", "💰", "💵", "🪙", "🏦", "🏢",
  "📈", "🚀", "🏆", "🎁", "🤝", "💻",
  "✨", "🎯", "💎", "⭐", "🏷️", "💳"
];

interface SelectorEmojiProps {
  emojiSeleccionado: string;
  onSeleccionarEmoji: (emoji: string) => void;
  tipo?: "gasto" | "ingreso";
}

export default function SelectorEmoji({
  emojiSeleccionado,
  onSeleccionarEmoji,
  tipo = "gasto",
}: SelectorEmojiProps) {
  const [abierto, setAbierto] = useState(false);
  const [pestaña, setPestaña] = useState<"gasto" | "ingreso">(tipo);
  const contenedorRef = useRef<HTMLDivElement>(null);

  // Sincronizar pestaña si cambia el tipo desde afuera
  useEffect(() => {
    setPestaña(tipo);
  }, [tipo]);

  // Cerrar al hacer clic fuera del popup
  useEffect(() => {
    function handleClickAfuera(e: MouseEvent) {
      if (contenedorRef.current && !contenedorRef.current.contains(e.target as Node)) {
        setAbierto(false);
      }
    }
    if (abierto) {
      document.addEventListener("mousedown", handleClickAfuera);
    }
    return () => {
      document.removeEventListener("mousedown", handleClickAfuera);
    };
  }, [abierto]);

  const listaActual = pestaña === "gasto" ? EMOJIS_GASTO : EMOJIS_INGRESO;

  return (
    <div ref={contenedorRef} style={{ position: "relative", display: "inline-block" }}>
      <button
        type="button"
        onClick={() => setAbierto((prev) => !prev)}
        title="Elegir icono / emoji para la categoría"
        style={{
          width: "42px",
          height: "42px",
          borderRadius: "8px",
          border: "1.5px solid var(--border-subtle)",
          backgroundColor: "#FFFFFF",
          fontSize: "20px",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          cursor: "pointer",
          transition: "all 0.15s ease",
          boxShadow: abierto ? "0 0 0 2px rgba(5, 150, 105, 0.2)" : "none",
        }}
      >
        {emojiSeleccionado || "🏷️"}
      </button>

      {abierto && (
        <div
          style={{
            position: "absolute",
            top: "calc(100% + 6px)",
            left: 0,
            zIndex: 100,
            width: "270px",
            backgroundColor: "#FFFFFF",
            borderRadius: "12px",
            border: "1px solid var(--border-subtle)",
            boxShadow: "0 10px 25px -5px rgba(0, 0, 0, 0.12), 0 8px 10px -6px rgba(0, 0, 0, 0.08)",
            padding: "10px",
          }}
        >
          {/* Cabecera / Pestañas de categorías */}
          <div
            style={{
              display: "flex",
              gap: "4px",
              marginBottom: "8px",
              paddingBottom: "8px",
              borderBottom: "1px solid var(--border-subtle)",
            }}
          >
            <button
              type="button"
              onClick={() => setPestaña("gasto")}
              style={{
                flex: 1,
                padding: "5px 8px",
                fontSize: "11.5px",
                fontWeight: pestaña === "gasto" ? 700 : 500,
                borderRadius: "6px",
                backgroundColor: pestaña === "gasto" ? "rgba(220, 38, 38, 0.08)" : "transparent",
                color: pestaña === "gasto" ? "#DC2626" : "var(--text-secondary)",
                border: pestaña === "gasto" ? "1px solid #FECACA" : "1px solid transparent",
              }}
            >
              Gastos
            </button>
            <button
              type="button"
              onClick={() => setPestaña("ingreso")}
              style={{
                flex: 1,
                padding: "5px 8px",
                fontSize: "11.5px",
                fontWeight: pestaña === "ingreso" ? 700 : 500,
                borderRadius: "6px",
                backgroundColor: pestaña === "ingreso" ? "rgba(5, 150, 105, 0.08)" : "transparent",
                color: pestaña === "ingreso" ? "var(--emerald-primary)" : "var(--text-secondary)",
                border: pestaña === "ingreso" ? "1px solid #A7F3D0" : "1px solid transparent",
              }}
            >
              Ingresos
            </button>
          </div>

          {/* Grilla de Emojis */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(6, 1fr)",
              gap: "6px",
              maxHeight: "180px",
              overflowY: "auto",
              paddingRight: "2px",
            }}
          >
            {listaActual.map((emoji) => {
              const estaSeleccionado = emoji === emojiSeleccionado;
              return (
                <button
                  key={emoji}
                  type="button"
                  onClick={() => {
                    onSeleccionarEmoji(emoji);
                    setAbierto(false);
                  }}
                  style={{
                    width: "36px",
                    height: "36px",
                    borderRadius: "6px",
                    fontSize: "18px",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    backgroundColor: estaSeleccionado ? "rgba(5, 150, 105, 0.12)" : "transparent",
                    border: estaSeleccionado ? "1.5px solid var(--emerald-primary)" : "1px solid transparent",
                    cursor: "pointer",
                    transition: "transform 0.1s ease, background-color 0.1s ease",
                  }}
                  onMouseEnter={(e) => {
                    if (!estaSeleccionado) e.currentTarget.style.backgroundColor = "var(--surface-hover)";
                  }}
                  onMouseLeave={(e) => {
                    if (!estaSeleccionado) e.currentTarget.style.backgroundColor = "transparent";
                  }}
                >
                  {emoji}
                </button>
              );
            })}
          </div>

          <div
            style={{
              marginTop: "8px",
              paddingTop: "6px",
              borderTop: "1px solid var(--border-subtle)",
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              fontSize: "10.5px",
              color: "var(--text-muted)",
            }}
          >
            <span>Selecciona un icono</span>
            <button
              type="button"
              onClick={() => {
                onSeleccionarEmoji("");
                setAbierto(false);
              }}
              style={{
                color: "var(--text-secondary)",
                fontSize: "11px",
                textDecoration: "underline",
              }}
            >
              Sin icono
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
