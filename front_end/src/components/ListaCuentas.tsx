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

const CONFIG_TIPOS: Record<string, { etiqueta: string; icono: string; bg: string; text: string }> = {
  efectivo: { etiqueta: "Efectivo", icono: "💵", bg: "var(--badge-cash-bg)", text: "var(--badge-cash-text)" },
  banco: { etiqueta: "Banco", icono: "🏦", bg: "var(--badge-bank-bg)", text: "var(--badge-bank-text)" },
  billetera_digital: { etiqueta: "Billetera digital", icono: "📱", bg: "var(--badge-wallet-bg)", text: "var(--badge-wallet-text)" },
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
          <div style={{ fontSize: "32px", marginBottom: "14px" }}>👛</div>
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
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "14px", fontSize: "11px", color: "var(--text-secondary)", fontWeight: 600 }}>
        <span style={{ letterSpacing: "1px" }}>TUS CUENTAS ({cuentas.length})</span>
        <span>Haz clic en una cuenta para ver sus movimientos</span>
      </div>

      <div className="table-card">
        <div style={{ height: "38px", padding: "0 24px", display: "flex", alignItems: "center", fontSize: "10px", fontWeight: 700, color: "var(--text-secondary)", borderBottom: "1px solid var(--border-subtle)" }}>
          <span style={{ flex: 1.2 }}>CUENTA</span>
          <span style={{ width: "160px" }}>TIPO</span>
          <span style={{ width: "180px", textAlign: "right" }}>SALDO DISPONIBLE</span>
          <span style={{ width: "140px", textAlign: "right" }}>DETALLE</span>
        </div>

        {cuentas.map((cuenta) => {
          const config = CONFIG_TIPOS[cuenta.tipo.toLowerCase()] || {
            etiqueta: cuenta.tipo,
            icono: "💰",
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
              <div style={{ flex: 1.2, display: "flex", alignItems: "center", gap: "16px" }}>
                <div style={{ width: "40px", height: "40px", borderRadius: "10px", border: "1px solid var(--border-subtle)", backgroundColor: "var(--bg-app)", display: "flex", alignItems: "center", justifyContent: "center", fontSize: "18px" }}>
                  {config.icono}
                </div>
                <div>
                  <div style={{ fontSize: "14px", fontWeight: 600 }}>{cuenta.nombre}</div>
                  <div style={{ fontSize: "11px", color: "var(--text-secondary)" }}>
                    {cuenta.moneda} · Moneda fija
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
