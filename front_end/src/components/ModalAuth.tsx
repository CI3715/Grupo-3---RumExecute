"use client";
import React, { useState, useEffect } from "react";
import {
  DatosUsuario,
  iniciarSesion,
  registrarUsuario,
} from "../services/authService";
interface ModalAuthProps {
  isOpen: boolean;
  modoInicial?: "login" | "registro";
  onClose: () => void;
  onLoginExitoso: (usuario: DatosUsuario) => void;
}
export default function ModalAuth({
  isOpen,
  modoInicial = "login",
  onClose,
  onLoginExitoso,
}: ModalAuthProps) {
  const [modo, setModo] = useState<"login" | "registro">(modoInicial);
  const [correo, setCorreo] = useState("");
  const [contrasena, setContrasena] = useState("");
  const [nombreUsuario, setNombreUsuario] = useState("");
  const [nombreVisible, setNombreVisible] = useState("");
  const [confirmarClave, setConfirmarClave] = useState("");
  const [mostrarClave, setMostrarClave] = useState(false);
  const [mostrarConfirmarClave, setMostrarConfirmarClave] = useState(false);
  const [cargando, setCargando] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [campoError, setCampoError] = useState<string | null>(null);
  const [mensajeExito, setMensajeExito] = useState<string | null>(null);
  useEffect(() => {
    if (isOpen) {
      setModo(modoInicial);
      setError(null);
      setCampoError(null);
      setMensajeExito(null);
      setMostrarClave(false);
      setMostrarConfirmarClave(false);
    }
  }, [isOpen, modoInicial]);
  if (!isOpen) return null;
  // Reglas de validación en vivo (RF-SOC-01, RNF-18)
  const esUsernameValido = /^[a-z0-9._]{3,30}$/.test(nombreUsuario);
  const esCorreoValido = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(correo.trim());
  const esClaveValida = contrasena.length >= 10;
  const coincidenClaves = contrasena === confirmarClave;
  const formularioValido =
    modo === "login"
      ? esCorreoValido && contrasena.length > 0
      : esUsernameValido && nombreVisible.trim().length > 0 && esCorreoValido && esClaveValida && coincidenClaves;
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formularioValido || cargando) return;
    setCargando(true);
    setError(null);
    setCampoError(null);
    setMensajeExito(null);
    try {
      if (modo === "login") {
        const sesion = await iniciarSesion({
          correo: correo.trim().toLowerCase(),
          contrasena,
        });
        setCorreo("");
        setContrasena("");
        onLoginExitoso(sesion.usuario);
        onClose();
      } else {
        await registrarUsuario({
          nombre_usuario: nombreUsuario.trim().toLowerCase(),
          nombre_visible: nombreVisible.trim(),
          correo: correo.trim().toLowerCase(),
          contrasena,
        });
        setMensajeExito("¡Cuenta registrada con éxito! Ingresa con tus credenciales.");
        setModo("login");
        setContrasena("");
        setConfirmarClave("");
      }
    } catch (err: any) {
      const msg = err?.message || "Ocurrió un error en la solicitud.";
      setError(msg);
      if (err?.campo) {
        setCampoError(err.campo);
      } else if (msg.toLowerCase().includes("usuario")) {
        setCampoError("nombre_usuario");
      } else if (msg.toLowerCase().includes("correo")) {
        setCampoError("correo");
      }
    } finally {
      setCargando(false);
    }
  };
  const cambiarModo = (nuevoModo: "login" | "registro") => {
    setModo(nuevoModo);
    setError(null);
    setCampoError(null);
    setMensajeExito(null);
    setMostrarClave(false);
    setMostrarConfirmarClave(false);
  };
  return (
    <div
      className="modal-overlay"
      onMouseDown={(e) => e.target === e.currentTarget && onClose()}
      role="dialog"
      aria-modal="true"
    >
      <div className="modal-card" style={{ maxWidth: "430px" }}>
        <div className="modal-header" style={{ paddingBottom: "12px" }}>
          <div>
            <h3 className="modal-title">
              {modo === "login" ? "Iniciar Sesión" : "Crear Cuenta"}
            </h3>
            <p className="modal-subtitle">
              {modo === "login"
                ? "Accede para sincronizar tus grupos y finanzas compartidas."
                : "Crea tu perfil para empezar a compartir gastos."}
            </p>
          </div>
          <button onClick={onClose} className="btn-close" title="Cerrar" disabled={cargando}>✕</button>
        </div>
        {/* Selector de pestañas Login / Registro */}
        <div style={{ display: "flex", gap: "6px", padding: "0 24px 12px 24px" }}>
          <button
            type="button"
            onClick={() => cambiarModo("login")}
            className="btn"
            style={{
              flex: 1,
              padding: "6px",
              fontSize: "13px",
              fontWeight: modo === "login" ? 700 : 500,
              backgroundColor: modo === "login" ? "var(--surface-hover)" : "transparent",
              borderColor: modo === "login" ? "var(--emerald-primary)" : "var(--border-subtle)",
              color: modo === "login" ? "var(--emerald-primary)" : "var(--text-secondary)",
            }}
          >
            Iniciar Sesión
          </button>
          <button
            type="button"
            onClick={() => cambiarModo("registro")}
            className="btn"
            style={{
              flex: 1,
              padding: "6px",
              fontSize: "13px",
              fontWeight: modo === "registro" ? 700 : 500,
              backgroundColor: modo === "registro" ? "var(--surface-hover)" : "transparent",
              borderColor: modo === "registro" ? "var(--emerald-primary)" : "var(--border-subtle)",
              color: modo === "registro" ? "var(--emerald-primary)" : "var(--text-secondary)",
            }}
          >
            Registrarse
          </button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="modal-body" style={{ display: "flex", flexDirection: "column", gap: "12px", paddingTop: "0" }}>
            {mensajeExito && (
              <div style={{ padding: "8px 12px", backgroundColor: "#ECFDF5", color: "#065F46", borderRadius: "8px", fontSize: "13px" }}>
                {mensajeExito}
              </div>
            )}
            {error && (
              <div style={{ padding: "8px 12px", backgroundColor: "#FEF2F2", color: "#B91C1C", borderRadius: "8px", fontSize: "13px" }}>
                {error}
              </div>
            )}
            {modo === "registro" && (
              <>
                <div className="form-group">
                  <label className="form-label">Nombre de usuario</label>
                  <input
                    type="text"
                    value={nombreUsuario}
                    onChange={(e) => {
                      setNombreUsuario(e.target.value.toLowerCase().replace(/\s/g, ""));
                      setError(null);
                      setCampoError(null);
                    }}
                    placeholder="ej. juan_perez"
                    className={`form-input ${(nombreUsuario && !esUsernameValido) || campoError === "nombre_usuario" ? "input-error" : ""}`}
                    disabled={cargando}
                    autoFocus
                  />
                  {nombreUsuario && !esUsernameValido ? (
                    <span className="error-text" style={{ color: "#EF4444", fontSize: "11.5px" }}>
                      Debe tener entre 3 y 30 caracteres (solo minúsculas a-z, 0-9, punto y guion bajo).
                    </span>
                  ) : (
                    <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                      3-30 caracteres: solo letras minúsculas, números, punto o guion bajo.
                    </span>
                  )}
                </div>
                <div className="form-group">
                  <label className="form-label">Nombre visible</label>
                  <input
                    type="text"
                    value={nombreVisible}
                    onChange={(e) => {
                      setNombreVisible(e.target.value);
                      setError(null);
                    }}
                    placeholder="ej. Juan Pérez"
                    className={`form-input ${campoError === "nombre_visible" ? "input-error" : ""}`}
                    disabled={cargando}
                  />
                </div>
              </>
            )}
            <div className="form-group">
              <label className="form-label">Correo electrónico</label>
              <input
                type="email"
                value={correo}
                onChange={(e) => {
                  setCorreo(e.target.value);
                  setError(null);
                  setCampoError(null);
                }}
                placeholder="tu@correo.com"
                className={`form-input ${(correo && !esCorreoValido) || campoError === "correo" ? "input-error" : ""}`}
                disabled={cargando}
                autoFocus={modo === "login"}
              />
              {correo && !esCorreoValido && (
                <span className="error-text" style={{ color: "#EF4444", fontSize: "11.5px" }}>
                  Ingresa un formato de correo electrónico válido.
                </span>
              )}
            </div>
            <div className="form-group">
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <label className="form-label">Contraseña</label>
                {modo === "registro" && (
                  <span style={{ fontSize: "11px", color: esClaveValida ? "#059669" : "var(--text-muted)" }}>
                    {contrasena.length}/10 mín.
                  </span>
                )}
              </div>
              <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                <input
                  type={mostrarClave ? "text" : "password"}
                  value={contrasena}
                  onChange={(e) => {
                    setContrasena(e.target.value);
                    setError(null);
                  }}
                  placeholder={modo === "registro" ? "Mínimo 10 caracteres" : "Tu contraseña"}
                  className={`form-input ${modo === "registro" && contrasena && !esClaveValida ? "input-error" : ""}`}
                  style={{ width: "100%", paddingRight: "40px" }}
                  disabled={cargando}
                />
                <button
                  type="button"
                  onClick={() => setMostrarClave((prev) => !prev)}
                  title={mostrarClave ? "Ocultar contraseña" : "Mostrar contraseña"}
                  aria-label={mostrarClave ? "Ocultar contraseña" : "Mostrar contraseña"}
                  style={{
                    position: "absolute",
                    right: "8px",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    background: "none",
                    border: "none",
                    color: "var(--text-secondary)",
                    cursor: "pointer",
                    padding: "4px",
                    borderRadius: "6px",
                  }}
                  tabIndex={-1}
                >
                  {mostrarClave ? (
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" />
                      <line x1="1" y1="1" x2="23" y2="23" />
                    </svg>
                  ) : (
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                      <circle cx="12" cy="12" r="3" />
                    </svg>
                  )}
                </button>
              </div>
              {modo === "registro" && contrasena && !esClaveValida && (
                <span className="error-text" style={{ color: "#EF4444", fontSize: "11.5px" }}>
                  La contraseña debe tener al menos 10 caracteres.
                </span>
              )}
            </div>
            {modo === "registro" && (
              <div className="form-group">
                <label className="form-label">Confirmar contraseña</label>
                <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
                  <input
                    type={mostrarConfirmarClave ? "text" : "password"}
                    value={confirmarClave}
                    onChange={(e) => setConfirmarClave(e.target.value)}
                    placeholder="Repite la contraseña"
                    className={`form-input ${confirmarClave && !coincidenClaves ? "input-error" : ""}`}
                    style={{ width: "100%", paddingRight: "40px" }}
                    disabled={cargando}
                  />
                  <button
                    type="button"
                    onClick={() => setMostrarConfirmarClave((prev) => !prev)}
                    title={mostrarConfirmarClave ? "Ocultar contraseña" : "Mostrar contraseña"}
                    aria-label={mostrarConfirmarClave ? "Ocultar contraseña" : "Mostrar contraseña"}
                    style={{
                      position: "absolute",
                      right: "8px",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      background: "none",
                      border: "none",
                      color: "var(--text-secondary)",
                      cursor: "pointer",
                      padding: "4px",
                      borderRadius: "6px",
                    }}
                    tabIndex={-1}
                  >
                    {mostrarConfirmarClave ? (
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" />
                        <line x1="1" y1="1" x2="23" y2="23" />
                      </svg>
                    ) : (
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                        <circle cx="12" cy="12" r="3" />
                      </svg>
                    )}
                  </button>
                </div>
                {confirmarClave && !coincidenClaves && (
                  <span className="error-text" style={{ color: "#EF4444", fontSize: "11.5px" }}>
                    Las contraseñas no coinciden.
                  </span>
                )}
              </div>
            )}
          </div>
          <div className="modal-footer" style={{ flexDirection: "column", gap: "10px", marginTop: "16px" }}>
            <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px", width: "100%" }}>
              <button type="button" onClick={onClose} className="btn btn-secondary" disabled={cargando}>
                Cancelar
              </button>
              <button
                type="submit"
                className="btn btn-primary"
                disabled={!formularioValido || cargando}
                style={{ minWidth: "120px", opacity: !formularioValido || cargando ? 0.6 : 1 }}
              >
                {cargando ? "Procesando..." : modo === "login" ? "Entrar" : "Crear Cuenta"}
              </button>
            </div>
            <div style={{ textAlign: "center", fontSize: "12.5px", color: "var(--text-secondary)", paddingTop: "6px", borderTop: "1px solid var(--border-subtle)", width: "100%" }}>
              {modo === "login" ? (
                <>
                  ¿No tienes una cuenta?{" "}
                  <button
                    type="button"
                    onClick={() => cambiarModo("registro")}
                    style={{ background: "none", border: "none", color: "var(--emerald-primary)", fontWeight: 700, cursor: "pointer", padding: 0 }}
                  >
                    Crear cuenta nueva
                  </button>
                </>
              ) : (
                <>
                  ¿Ya tienes una cuenta?{" "}
                  <button
                    type="button"
                    onClick={() => cambiarModo("login")}
                    style={{ background: "none", border: "none", color: "var(--emerald-primary)", fontWeight: 700, cursor: "pointer", padding: 0 }}
                  >
                    Iniciar sesión
                  </button>
                </>
              )}
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}
