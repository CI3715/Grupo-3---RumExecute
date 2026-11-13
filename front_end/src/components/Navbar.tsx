"use client";

import { DatosUsuario } from "../services/authService";

interface NavbarProps {
  tabActivo?: "cuentas" | "patrimonio" | "tasas" | "comunidad";
  onCambiarTab?: (tab: "cuentas" | "patrimonio" | "tasas" | "comunidad") => void;
  onNavigateHome?: () => void;
  usuario?: DatosUsuario | null;
  onOpenAuth?: (modo: "login" | "registro") => void;
  onCerrarSesion?: () => void;
  servidorOffline?: boolean;
}

export default function Navbar({
  tabActivo = "cuentas",
  onCambiarTab,
  onNavigateHome,
  usuario,
  onOpenAuth,
  onCerrarSesion,
  servidorOffline = false,
}: NavbarProps) {
  const iniciales = (nombre: string) => {
    const partes = nombre.trim().split(" ");
    return partes.length >= 2
      ? (partes[0][0] + partes[1][0]).toUpperCase()
      : (nombre[0] || "U").toUpperCase();
  };

  return (
    <header className="app-navbar" style={{ gap: "16px" }}>
      <div
        className="brand-container"
        onClick={() => {
          onNavigateHome?.();
          onCambiarTab?.("cuentas");
        }}
        title="Volver a Mis Cuentas"
      >
        <svg viewBox="0 0 512 512" width="40" height="40" style={{ flexShrink: 0 }}>
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
          <div style={{ fontSize: "21px", letterSpacing: "-0.5px", lineHeight: 1.1 }}>
            <span style={{ fontWeight: 800 }}>Cuentas</span>
            <span style={{ fontWeight: 800, color: "var(--emerald-primary)", marginLeft: "5px" }}>Claras</span>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: "6px", marginTop: "2px" }}>
            <span style={{ fontSize: "9.5px", fontWeight: 700, color: "var(--text-secondary)", letterSpacing: "1.1px" }}>
              BILLETERA LOCAL
            </span>
            <span style={{ width: "4px", height: "4px", borderRadius: "50%", backgroundColor: "var(--emerald-primary)" }} />
            {servidorOffline ? (
              <span
                style={{
                  fontSize: "10px",
                  fontWeight: 600,
                  color: "#D97706",
                  backgroundColor: "#FEF3C7",
                  padding: "1px 6px",
                  borderRadius: "10px",
                }}
                title="Servidor de sincronización desconectado. Tus datos locales están 100% seguros y privados."
              >
                Offline local
              </span>
            ) : (
              <span style={{ fontSize: "10px", fontWeight: 600, color: "var(--text-muted)" }}>100% Privado</span>
            )}
          </div>
        </div>
      </div>

      {/* Tabs Principales de Navegación */}
      {onCambiarTab && (
        <nav
          style={{
            display: "flex",
            alignItems: "center",
            backgroundColor: "var(--surface-hover)",
            padding: "3px",
            borderRadius: "10px",
            border: "1px solid var(--border-subtle)",
          }}
        >
          <button
            type="button"
            onClick={() => onCambiarTab("cuentas")}
            style={{
              padding: "6px 14px",
              borderRadius: "8px",
              fontSize: "13px",
              fontWeight: tabActivo === "cuentas" ? 700 : 500,
              backgroundColor: tabActivo === "cuentas" ? "#FFFFFF" : "transparent",
              color: tabActivo === "cuentas" ? "var(--text-primary)" : "var(--text-secondary)",
              boxShadow: tabActivo === "cuentas" ? "0 1px 3px rgba(0,0,0,0.08)" : "none",
              transition: "all 0.15s ease",
            }}
          >
            Mis Cuentas
          </button>

          <button
            type="button"
            onClick={() => onCambiarTab("patrimonio")}
            style={{
              padding: "6px 14px",
              borderRadius: "8px",
              fontSize: "13px",
              fontWeight: tabActivo === "patrimonio" ? 700 : 500,
              backgroundColor: tabActivo === "patrimonio" ? "#FFFFFF" : "transparent",
              color: tabActivo === "patrimonio" ? "var(--text-primary)" : "var(--text-secondary)",
              boxShadow: tabActivo === "patrimonio" ? "0 1px 3px rgba(0,0,0,0.08)" : "none",
              transition: "all 0.15s ease",
            }}
          >
            Patrimonio
          </button>

          <button
            type="button"
            onClick={() => onCambiarTab("tasas")}
            style={{
              padding: "6px 14px",
              borderRadius: "8px",
              fontSize: "13px",
              fontWeight: tabActivo === "tasas" ? 700 : 500,
              backgroundColor: tabActivo === "tasas" ? "#FFFFFF" : "transparent",
              color: tabActivo === "tasas" ? "var(--text-primary)" : "var(--text-secondary)",
              boxShadow: tabActivo === "tasas" ? "0 1px 3px rgba(0,0,0,0.08)" : "none",
              transition: "all 0.15s ease",
            }}
          >
            Tasas de Cambio
          </button>

          <button
            type="button"
            onClick={() => onCambiarTab("comunidad")}
            style={{
              padding: "6px 14px",
              borderRadius: "8px",
              fontSize: "13px",
              fontWeight: tabActivo === "comunidad" ? 700 : 500,
              backgroundColor: tabActivo === "comunidad" ? "#FFFFFF" : "transparent",
              color: tabActivo === "comunidad" ? "var(--text-primary)" : "var(--text-secondary)",
              boxShadow: tabActivo === "comunidad" ? "0 1px 3px rgba(0,0,0,0.08)" : "none",
              transition: "all 0.15s ease",
            }}
          >
            Grupos y Amigos
          </button>
        </nav>
      )}

      {/* Sección de Sesión de Usuario */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        {usuario ? (
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: "8px",
                padding: "3px 10px 3px 4px",
                backgroundColor: "var(--surface-hover)",
                borderRadius: "20px",
                border: "1px solid var(--border-subtle)",
              }}
              title={`Conectado como @${usuario.nombre_usuario}`}
            >
              <div
                style={{
                  width: "26px",
                  height: "26px",
                  borderRadius: "50%",
                  background: "linear-gradient(135deg, #059669 0%, #10B981 100%)",
                  color: "#FFFFFF",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: "11px",
                  fontWeight: 700,
                }}
              >
                {iniciales(usuario.nombre_visible)}
              </div>
              <div style={{ display: "flex", flexDirection: "column", lineHeight: 1.1 }}>
                <span style={{ fontSize: "12.5px", fontWeight: 700, color: "var(--text-primary)" }}>
                  {usuario.nombre_visible}
                </span>
                <span style={{ fontSize: "10.5px", color: "var(--text-muted)" }}>
                  @{usuario.nombre_usuario}
                </span>
              </div>
            </div>

            {onCerrarSesion && (
              <button
                type="button"
                onClick={onCerrarSesion}
                className="btn btn-secondary"
                style={{ padding: "5px 10px", fontSize: "12px", color: "#EF4444" }}
                title="Cerrar sesión"
              >
                Cerrar sesión
              </button>
            )}
          </div>
        ) : (
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            <button
              type="button"
              onClick={() => onOpenAuth?.("login")}
              className="btn btn-secondary"
              style={{ padding: "6px 12px", fontSize: "12.5px" }}
            >
              Iniciar Sesión
            </button>
            <button
              type="button"
              onClick={() => onOpenAuth?.("registro")}
              className="btn btn-secondary"
              style={{
                padding: "6px 12px",
                fontSize: "12.5px",
                fontWeight: 700,
                color: "var(--emerald-primary)",
                borderColor: "var(--emerald-primary)",
              }}
            >
              Registrarse
            </button>
          </div>
        )}
      </div>
    </header>
  );
}
