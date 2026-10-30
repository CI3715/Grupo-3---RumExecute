"use client";

export interface CuentaItem {
  id: string;
  nombre: string;
  tipo: string;
  tipo_etiqueta?: string;
  moneda: string;
  saldo_formateado: string;
}

interface ListaCuentasProps {
  cuentas: CuentaItem[];
  onCrearPrimeraCuenta: () => void;
  onSeleccionarCuenta?: (id: string) => void;
}

const CONFIG_TIPOS: Record<string, { etiqueta: string; bg: string; text: string }> = {
  efectivo: { etiqueta: "Efectivo", bg: "var(--badge-cash-bg)", text: "var(--badge-cash-text)" },
  banco: { etiqueta: "Banco", bg: "var(--badge-bank-bg)", text: "var(--badge-bank-text)" },
  billetera_digital: { etiqueta: "Billetera digital", bg: "var(--badge-wallet-bg)", text: "var(--badge-wallet-text)" },
};

export default function ListaCuentas({
  cuentas,
  onCrearPrimeraCuenta,
  onSeleccionarCuenta,
}: ListaCuentasProps) {
  if (cuentas.length === 0) {
    return (
      <section style={{ width: "100%", marginBottom: "48px" }}>
        <div className="empty-card">
          <h3 style={{ fontSize: "20px", fontWeight: 700, marginBottom: "8px" }}>
            Aún no tienes cuentas registradas
          </h3>
          <p className="helper-text" style={{ maxWidth: "500px", lineHeight: 1.5, marginBottom: "24px", fontSize: "14px" }}>
            Registra tu efectivo, cuentas bancarias o billeteras digitales para llevar tus cuentas
            claras de forma 100% local, sin servidores y con total privacidad.
          </p>
          <button onClick={onCrearPrimeraCuenta} className="btn btn-primary">
            + Crear mi primera cuenta
          </button>
        </div>
      </section>
    );
  }

  return (
    <section style={{ width: "100%", marginBottom: "48px" }}>
      <div className="table-card">
        <div className="account-row-header" style={{
          display: "flex",
          alignItems: "center",
          padding: "12px 24px",
          background: "var(--bg-app)",
          borderBottom: "1px solid var(--border-subtle)",
          fontSize: "11px",
          fontWeight: 700,
          color: "var(--text-secondary)",
          letterSpacing: "0.5px"
        }}>
          <span style={{ flex: 1.2 }}>NOMBRE DE CUENTA</span>
          <span style={{ width: "160px" }}>TIPO</span>
          <span style={{ width: "180px", textAlign: "right" }}>SALDO DISPONIBLE</span>
          <span style={{ width: "140px", textAlign: "right" }}>DETALLE</span>
        </div>

        {cuentas.map((cuenta) => {
          const config = CONFIG_TIPOS[cuenta.tipo.toLowerCase()] || {
            etiqueta: cuenta.tipo,
            bg: "#F1F5F9",
            text: "#475569",
          };

          return (
            <div
              key={cuenta.id}
              onClick={() => onSeleccionarCuenta?.(cuenta.id)}
              className="account-row"
              title="Ver movimientos y registrar gastos o ingresos"
            >
              <div style={{ flex: 1.2, display: "flex", alignItems: "center", gap: "12px" }}>
                <span className="currency-code-pill">{cuenta.moneda}</span>
                <div>
                  <div style={{ fontSize: "14px", fontWeight: 600 }}>{cuenta.nombre}</div>
                  <div style={{ fontSize: "11px", color: "var(--text-secondary)" }}>
                    Moneda fija ({cuenta.moneda})
                  </div>
                </div>
              </div>

              <div style={{ width: "160px" }}>
                <span className="badge" style={{ backgroundColor: config.bg, color: config.text }}>
                  {config.etiqueta}
                </span>
              </div>

              <div style={{ width: "180px", textAlign: "right", fontSize: "17px", fontWeight: 600, letterSpacing: "-0.4px" }}>
                {cuenta.saldo_formateado}
              </div>

              <div style={{ width: "140px", display: "flex", alignItems: "center", justifyContent: "flex-end", gap: "6px" }}>
                <span className="btn-link">Ver movimientos</span>
                <span style={{ fontSize: "18px", color: "var(--emerald-primary)" }}>›</span>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
