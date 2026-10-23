"use client";

interface NavbarProps {
  onOpenNuevaCuenta: () => void;
  onOpenDiagnostico: () => void;
  onNavigateHome?: () => void;
}

export default function Navbar({
  onOpenNuevaCuenta,
  onOpenDiagnostico,
  onNavigateHome,
}: NavbarProps) {
  return (
    <header className="app-navbar">
      <div className="brand-container" onClick={onNavigateHome} title="Volver a Mis Cuentas">
        <svg viewBox="0 0 512 512" width="42" height="42" style={{ flexShrink: 0 }}>
          <defs>
            <linearGradient id="nav-emerald" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#059669" />
              <stop offset="100%" stopColor="#022c22" />
            </linearGradient>
            <linearGradient id="nav-mint" x1="0%" y1="100%" x2="100%" y2="0%">
              <stop offset="0%" stopColor="#34D399" />
              <stop offset="100%" stopColor="#059669" />
            </linearGradient>
          </defs>
          <g transform="translate(256, 256) scale(0.95)">
            <path d="M 50,-150 A 160,160 0 1,0 50,150 L 50,86 A 96,96 0 1,1 50,-86 Z" fill="url(#nav-emerald)" />
            <path d="M -20,-110 A 120,120 0 0,1 110,-20 L 165,-45 A 180,180 0 0,0 -45,-165 Z" fill="url(#nav-mint)" />
            <circle cx="85" cy="55" r="30" fill="#059669" />
            <circle cx="85" cy="55" r="13" fill="#FFFFFF" />
          </g>
        </svg>

        <div>
          <div style={{ fontSize: "22px", letterSpacing: "-0.5px", lineHeight: 1.1 }}>
            <span style={{ fontWeight: 800 }}>Cuentas</span>
            <span style={{ fontWeight: 800, color: "var(--emerald-primary)", marginLeft: "6px" }}>Claras</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginTop: "2px" }}>
            <span style={{ fontSize: "10px", fontWeight: 700, color: "var(--text-secondary)", letterSpacing: "1.2px" }}>
              BILLETERA LOCAL OFFLINE
            </span>
            <span style={{ width: "4px", height: "4px", borderRadius: "50%", backgroundColor: "var(--emerald-primary)" }} />
            <span style={{ fontSize: "10.5px", fontWeight: 600, color: "var(--text-muted)" }}>100% Privado</span>
          </div>
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
        <button onClick={onOpenDiagnostico} className="btn btn-secondary">
          <span>📶</span>
          <span>Diagnóstico Servidor</span>
        </button>
        <button onClick={onOpenNuevaCuenta} className="btn btn-primary">
          <span style={{ fontSize: "16px", fontWeight: 700 }}>+</span>
          <span>Nueva cuenta</span>
        </button>
      </div>
    </header>
  );
}
