"use client";

import { NOMBRES_MONEDA } from "../utils/formato";

export interface SubtotalMoneda {
  moneda: string;
  nombre_moneda?: string;
  simbolo?: string;
  subtotal_formateado: string;
  cantidad_cuentas: number;
}

interface SubtotalesGridProps {
  subtotales: SubtotalMoneda[];
}

export default function SubtotalesGrid({ subtotales }: SubtotalesGridProps) {
  if (subtotales.length === 0) return null;

  return (
    <section style={{ width: "100%", marginBottom: "32px" }}>
      <div className="subtotales-grid">
        {subtotales.map((item) => {
          const nombre = item.nombre_moneda || NOMBRES_MONEDA[item.moneda] || item.moneda;
          const textoCuentas =
            item.cantidad_cuentas === 1 ? "1 cuenta" : `${item.cantidad_cuentas} cuentas`;

          return (
            <div key={item.moneda} className="subtotal-card">
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <span style={{ fontSize: "14px", fontWeight: 500, color: "var(--text-secondary)" }}>
                  {nombre}
                </span>
                <span className="currency-code-pill">{item.moneda}</span>
              </div>

              <div>
                <div style={{ fontSize: "28px", fontWeight: 700, letterSpacing: "-0.5px", lineHeight: 1.1 }}>
                  {item.subtotal_formateado}
                </div>
                <div style={{ fontSize: "12px", color: "var(--text-secondary)", marginTop: "4px" }}>
                  {textoCuentas}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
