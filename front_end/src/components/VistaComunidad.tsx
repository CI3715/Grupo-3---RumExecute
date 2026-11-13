"use client";
import React, { useState, useEffect, useCallback } from "react";
import { DatosUsuario } from "../services/authService";
import { MonedaItem } from "../types/tasas";
import { formatearMonto } from "../utils/formato";
import {
  UsuarioPublico,
  ListadoAmistades,
  ListadoGrupos,
  DetalleGrupo,
  buscarUsuarioExacto,
  listarAmistades,
  enviarInvitacionAmistad,
  responderInvitacionAmistad,
  eliminarAmistad,
  listarGrupos,
  crearGrupo,
  invitarMiembroGrupo,
  responderInvitacionGrupo,
  obtenerDetalleGrupo,
  registrarGastoCompartido,
} from "../services/socialService";
interface VistaComunidadProps {
  usuario: DatosUsuario | null;
  monedas: MonedaItem[];
  onSolicitarLogin: () => void;
}
const cardStyle: React.CSSProperties = {
  backgroundColor: "var(--surface-card)",
  border: "1px solid var(--border-subtle)",
  borderRadius: "14px",
  padding: "20px 24px",
};
const rowStyle: React.CSSProperties = {
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
  padding: "10px 14px",
  borderRadius: "10px",
  border: "1px solid var(--border-subtle)",
};
export default function VistaComunidad({ usuario, monedas, onSolicitarLogin }: VistaComunidadProps) {
  const [subVista, setSubVista] = useState<"grupos" | "amigos" | "solicitudes" | "detalle_grupo">("grupos");
  const [aviso, setAviso] = useState<{ tipo: "exito" | "error"; texto: string } | null>(null);
  const [amistades, setAmistades] = useState<ListadoAmistades>({ amigos: [], invitaciones_recibidas: [], invitaciones_enviadas: [] });
  const [busquedaUsername, setBusquedaUsername] = useState("");
  const [usuarioEncontrado, setUsuarioEncontrado] = useState<UsuarioPublico | null>(null);
  const [amigoPorEliminar, setAmigoPorEliminar] = useState<{ amistad_id: string; nombre_visible: string; nombre_usuario: string } | null>(null);
  const [listadoGrupos, setListadoGrupos] = useState<ListadoGrupos>({ grupos: [], invitaciones_recibidas: [] });
  const [mostrarFormGrupo, setMostrarFormGrupo] = useState(false);
  const [formGrupo, setFormGrupo] = useState({ nombre: "", moneda: "USD", descripcion: "", guardando: false });
  const [busquedaAmigosGrupo, setBusquedaAmigosGrupo] = useState("");
  const [amigosSeleccionadosGrupo, setAmigosSeleccionadosGrupo] = useState<string[]>([]);
  const [grupoActivo, setGrupoActivo] = useState<DetalleGrupo | null>(null);
  const [tabGrupo, setTabGrupo] = useState<"gastos" | "balances" | "miembros">("gastos");
  const [mostrarFormGasto, setMostrarFormGasto] = useState(false);
  const [busquedaAmigosMiembros, setBusquedaAmigosMiembros] = useState("");
  const [conceptoGasto, setConceptoGasto] = useState("");
  const [montoGasto, setMontoGasto] = useState("");
  const [pagadorId, setPagadorId] = useState("");
  const [reglaDivision, setReglaDivision] = useState<"IGUALES" | "PORCENTAJES" | "MONTOS">("IGUALES");
  const [participantesIds, setParticipantesIds] = useState<string[]>([]);
  const [valoresCustom, setValoresCustom] = useState<Record<string, string>>({});
  const [guardandoGasto, setGuardandoGasto] = useState(false);
  const cargarDatos = useCallback(async () => {
    if (!usuario) return;
    try {
      const [am, gr] = await Promise.all([listarAmistades(), listarGrupos()]);
      setAmistades(am);
      setListadoGrupos(gr);
    } catch {}
  }, [usuario]);
  useEffect(() => {
    setSubVista("grupos");
    setGrupoActivo(null);
    setMostrarFormGrupo(false);
    setMostrarFormGasto(false);
    setAviso(null);
    cargarDatos();
  }, [usuario?.id, cargarDatos]);
  const abrirGrupo = async (grupoId: string) => {
    setAviso(null);
    try {
      const det = await obtenerDetalleGrupo(grupoId);
      setGrupoActivo(det);
      setPagadorId(usuario?.id || det.miembros[0]?.usuario_id || "");
      setParticipantesIds(det.miembros.map((m) => m.usuario_id));
      setValoresCustom({});
      setSubVista("detalle_grupo");
    } catch (e: any) {
      setAviso({ tipo: "error", texto: e?.message || "No se pudo abrir el grupo." });
    }
  };
  if (!usuario) {
    return (
      <div style={{ ...cardStyle, padding: "48px 28px", textAlign: "center" }}>
        <h2 style={{ fontSize: "20px", fontWeight: 800 }}>Grupos, Amigos y Gastos Compartidos</h2>
        <p style={{ fontSize: "14px", color: "var(--text-secondary)", margin: "10px auto 18px", maxWidth: "460px" }}>
          Inicia sesión o crea tu perfil para agregar amigos por su <strong>@usuario</strong>, crear grupos y dividir gastos compartidos.
        </p>
        <button type="button" onClick={onSolicitarLogin} className="btn btn-primary">
          Iniciar sesión o Registrarse
        </button>
      </div>
    );
  }
  const ejecutarAccion = async (fn: () => Promise<void>, exitoMsg?: string) => {
    setAviso(null);
    try {
      await fn();
      if (exitoMsg) setAviso({ tipo: "exito", texto: exitoMsg });
    } catch (err: any) {
      setAviso({ tipo: "error", texto: err?.message || "Ocurrió un error al procesar la solicitud." });
    }
  };
  const handleGuardarGasto = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!grupoActivo || !conceptoGasto.trim() || !montoGasto.trim()) return;
    setGuardandoGasto(true);
    await ejecutarAccion(async () => {
      const valores: Record<string, number> = {};
      if (reglaDivision !== "IGUALES") {
        participantesIds.forEach((uid) => {
          valores[uid] = parseFloat(String(valoresCustom[uid] || "0").replace(",", ".")) || 0;
        });
      }
      await registrarGastoCompartido(grupoActivo.id, {
        concepto: conceptoGasto.trim(),
        monto: parseFloat(montoGasto.replace(",", ".")),
        pagador_id: pagadorId || usuario.id,
        regla_division: reglaDivision,
        participantes: participantesIds,
        valores: reglaDivision === "IGUALES" ? undefined : valores,
      });
      setGrupoActivo(await obtenerDetalleGrupo(grupoActivo.id));
      setConceptoGasto("");
      setMontoGasto("");
      setValoresCustom({});
      setMostrarFormGasto(false);
    }, "Gasto registrado y balances actualizados.");
    setGuardandoGasto(false);
  };
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "20px" }}>
      <div className="hero-section">
        <div>
          <h1 className="hero-title">{subVista === "detalle_grupo" && grupoActivo ? grupoActivo.nombre_grupo : "Finanzas Compartidas"}</h1>
          <p className="hero-subtitle">
            {subVista === "detalle_grupo" && grupoActivo
              ? `${grupoActivo.descripcion || "Grupo colaborativo"} • Moneda: ${grupoActivo.moneda}`
              : "Divide gastos con amigos y grupos sin tocar las cuentas de tu libro personal."}
          </p>
        </div>
        <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
          {subVista === "detalle_grupo" ? (
            <>
              <button type="button" onClick={() => { setSubVista("grupos"); setGrupoActivo(null); cargarDatos(); }} className="btn btn-secondary">
                ← Volver a Grupos
              </button>
              <button type="button" onClick={() => setMostrarFormGasto(!mostrarFormGasto)} className="btn btn-primary">
                {mostrarFormGasto ? "Cerrar Formulario" : "+ Registrar Gasto"}
              </button>
            </>
          ) : (
            <>
              <button type="button" onClick={() => setSubVista("grupos")} className={subVista === "grupos" ? "btn btn-primary" : "btn btn-secondary"}>
                Mis Grupos ({listadoGrupos.grupos.length})
              </button>
              <button type="button" onClick={() => setSubVista("amigos")} className={subVista === "amigos" ? "btn btn-primary" : "btn btn-secondary"}>
                Amigos ({amistades.amigos.length})
              </button>
              <button type="button" onClick={() => setSubVista("solicitudes")} className={subVista === "solicitudes" ? "btn btn-primary" : "btn btn-secondary"}>
                Solicitudes ({amistades.invitaciones_recibidas.length + amistades.invitaciones_enviadas.length + listadoGrupos.invitaciones_recibidas.length})
              </button>
              {subVista === "grupos" && (
                <button
                  type="button"
                  onClick={() => {
                    setMostrarFormGrupo(!mostrarFormGrupo);
                    setBusquedaAmigosGrupo("");
                    setAmigosSeleccionadosGrupo([]);
                  }}
                  className="btn btn-secondary"
                  style={{ fontWeight: 700, color: "var(--emerald-primary)", borderColor: "var(--emerald-primary)" }}
                >
                  {mostrarFormGrupo ? "Cancelar" : "+ Crear Grupo"}
                </button>
              )}
            </>
          )}
        </div>
      </div>
      {aviso && (
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "12px 16px", borderRadius: "10px", fontSize: "13px", fontWeight: 600, backgroundColor: aviso.tipo === "exito" ? "#ECFDF5" : "#FEF2F2", color: aviso.tipo === "exito" ? "#065F46" : "#991B1B", border: `1px solid ${aviso.tipo === "exito" ? "#A7F3D0" : "#FCA5A5"}` }}>
          <span>{aviso.texto}</span>
          <button type="button" onClick={() => setAviso(null)} style={{ background: "none", border: "none", fontWeight: 700, cursor: "pointer", color: "inherit" }}>✕</button>
        </div>
      )}
      {/* VISTA SOLICITUDES */}
      {subVista === "solicitudes" && (
        <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
          <div style={cardStyle}>
            <h3 style={{ fontSize: "15px", fontWeight: 800, marginBottom: "10px" }}>
              Solicitudes de Amistad Recibidas ({amistades.invitaciones_recibidas.length})
            </h3>
            {amistades.invitaciones_recibidas.length === 0 ? (
              <p style={{ fontSize: "13px", color: "var(--text-secondary)" }}>No tienes solicitudes de amistad recibidas pendientes.</p>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                {amistades.invitaciones_recibidas.map((inv) => (
                  <div key={inv.amistad_id} style={rowStyle}>
                    <span><strong>{inv.nombre_visible}</strong> (@{inv.nombre_usuario})</span>
                    <div style={{ display: "flex", gap: "8px" }}>
                      <button type="button" className="btn btn-secondary" onClick={() => ejecutarAccion(async () => { await responderInvitacionAmistad(inv.amistad_id, false); await cargarDatos(); }, "Solicitud rechazada.")}>Rechazar</button>
                      <button type="button" className="btn btn-primary" onClick={() => ejecutarAccion(async () => { await responderInvitacionAmistad(inv.amistad_id, true); await cargarDatos(); }, "Solicitud aceptada.")}>Aceptar</button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
          <div style={cardStyle}>
            <h3 style={{ fontSize: "15px", fontWeight: 800, marginBottom: "10px" }}>
              Solicitudes de Amistad Enviadas ({amistades.invitaciones_enviadas.length})
            </h3>
            {amistades.invitaciones_enviadas.length === 0 ? (
              <p style={{ fontSize: "13px", color: "var(--text-secondary)" }}>No tienes solicitudes de amistad enviadas pendientes.</p>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                {amistades.invitaciones_enviadas.map((env) => (
                  <div key={env.amistad_id} style={rowStyle}>
                    <span><strong>{env.nombre_visible}</strong> (@{env.nombre_usuario})</span>
                    <span style={{ fontSize: "12px", fontWeight: 700, padding: "4px 10px", borderRadius: "12px", backgroundColor: "#FEF3C7", color: "#92400E" }}>
                      Pendiente de respuesta
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
          <div style={cardStyle}>
            <h3 style={{ fontSize: "15px", fontWeight: 800, marginBottom: "10px" }}>
              Invitaciones a Grupos Recibidas ({listadoGrupos.invitaciones_recibidas.length})
            </h3>
            {listadoGrupos.invitaciones_recibidas.length === 0 ? (
              <p style={{ fontSize: "13px", color: "var(--text-secondary)" }}>No tienes invitaciones a grupos pendientes.</p>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                {listadoGrupos.invitaciones_recibidas.map((inv) => (
                  <div key={inv.id} style={rowStyle}>
                    <span>Invitación al grupo #{inv.grupo_id.slice(0, 8)}</span>
                    <div style={{ display: "flex", gap: "8px" }}>
                      <button type="button" className="btn btn-secondary" onClick={() => ejecutarAccion(async () => { await responderInvitacionGrupo(inv.grupo_id, false); await cargarDatos(); }, "Invitación rechazada.")}>Rechazar</button>
                      <button type="button" className="btn btn-primary" onClick={() => ejecutarAccion(async () => { await responderInvitacionGrupo(inv.grupo_id, true); await cargarDatos(); }, "Te uniste al grupo.")}>Aceptar</button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
      {/* VISTA AMIGOS */}
      {subVista === "amigos" && (
        <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
          <div style={cardStyle}>
            <h3 style={{ fontSize: "15px", fontWeight: 800, marginBottom: "8px" }}>Enviar Solicitud de Amistad (Búsqueda Exacta por @usuario)</h3>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                setUsuarioEncontrado(null);
                ejecutarAccion(async () => setUsuarioEncontrado(await buscarUsuarioExacto(busquedaUsername)));
              }}
              style={{ display: "flex", gap: "10px" }}
            >
              <input type="text" value={busquedaUsername} onChange={(e) => setBusquedaUsername(e.target.value.toLowerCase().replace(/\s/g, ""))} placeholder="Escribe el @nombre_usuario exacto (ej. beto)" className="form-input" style={{ flex: 1 }} />
              <button type="submit" className="btn btn-primary">Buscar</button>
            </form>
            {usuarioEncontrado && (
              <div style={{ ...rowStyle, marginTop: "12px", backgroundColor: "var(--surface-hover)" }}>
                <div>
                  <strong>{usuarioEncontrado.nombre_visible}</strong> <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>@{usuarioEncontrado.nombre_usuario}</span>
                </div>
                <button type="button" className="btn btn-primary" onClick={() => ejecutarAccion(async () => { await enviarInvitacionAmistad(usuarioEncontrado.nombre_usuario); setUsuarioEncontrado(null); setBusquedaUsername(""); await cargarDatos(); }, "Solicitud de amistad enviada.")}>
                  Enviar invitación
                </button>
              </div>
            )}
          </div>
          {/* Tarjeta dedicada de Solicitudes de Amistad siempre visible en la pestaña de Amigos */}
          <div style={cardStyle}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
              <h3 style={{ fontSize: "15px", fontWeight: 800 }}>
                Solicitudes de Amistad ({amistades.invitaciones_recibidas.length + amistades.invitaciones_enviadas.length})
              </h3>
              <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
                {amistades.invitaciones_recibidas.length} recibida(s) • {amistades.invitaciones_enviadas.length} enviada(s)
              </span>
            </div>
            {amistades.invitaciones_recibidas.length === 0 && amistades.invitaciones_enviadas.length === 0 ? (
              <p style={{ fontSize: "13px", color: "var(--text-secondary)" }}>
                No tienes solicitudes de amistad pendientes en este momento.
              </p>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
                {amistades.invitaciones_recibidas.map((inv) => (
                  <div key={inv.amistad_id} style={{ ...rowStyle, backgroundColor: "#F8FAFC" }}>
                    <div>
                      <strong>{inv.nombre_visible}</strong>{" "}
                      <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>@{inv.nombre_usuario}</span>
                      <span style={{ marginLeft: "8px", fontSize: "11px", fontWeight: 700, padding: "2px 8px", borderRadius: "10px", backgroundColor: "#DBEAFE", color: "#1E40AF" }}>
                        Recibida
                      </span>
                    </div>
                    <div style={{ display: "flex", gap: "8px" }}>
                      <button type="button" className="btn btn-secondary" onClick={() => ejecutarAccion(async () => { await responderInvitacionAmistad(inv.amistad_id, false); await cargarDatos(); }, "Solicitud rechazada.")}>Rechazar</button>
                      <button type="button" className="btn btn-primary" onClick={() => ejecutarAccion(async () => { await responderInvitacionAmistad(inv.amistad_id, true); await cargarDatos(); }, "Solicitud aceptada.")}>Aceptar</button>
                    </div>
                  </div>
                ))}
                {amistades.invitaciones_enviadas.map((env) => (
                  <div key={env.amistad_id} style={rowStyle}>
                    <div>
                      <strong>{env.nombre_visible}</strong>{" "}
                      <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>@{env.nombre_usuario}</span>
                    </div>
                    <span style={{ fontSize: "11.5px", fontWeight: 700, padding: "4px 10px", borderRadius: "12px", backgroundColor: "#FEF3C7", color: "#92400E" }}>
                      Enviada (Pendiente)
                    </span>
                  </div>
                ))}
              </div>
            )}
          </div>
          {amigoPorEliminar && (
            <div style={{ ...rowStyle, backgroundColor: "#FEF2F2", borderColor: "#FCA5A5", color: "#991B1B" }}>
              <span>¿Eliminar a <strong>{amigoPorEliminar.nombre_visible} (@{amigoPorEliminar.nombre_usuario})</strong> de tus amigos?</span>
              <div style={{ display: "flex", gap: "8px" }}>
                <button type="button" className="btn btn-secondary" onClick={() => setAmigoPorEliminar(null)}>Cancelar</button>
                <button type="button" className="btn" style={{ backgroundColor: "#DC2626", color: "#FFF" }} onClick={() => ejecutarAccion(async () => { await eliminarAmistad(amigoPorEliminar.amistad_id); setAmigoPorEliminar(null); await cargarDatos(); }, "Amistad eliminada.")}>Sí, eliminar</button>
              </div>
            </div>
          )}
          <div style={cardStyle}>
            <h3 style={{ fontSize: "15px", fontWeight: 800, marginBottom: "10px" }}>Mis Amigos ({amistades.amigos.length})</h3>
            {amistades.amigos.length === 0 ? (
              <p style={{ fontSize: "13px", color: "var(--text-secondary)" }}>Aún no tienes amigos agregados.</p>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: "8px", maxHeight: "260px", overflowY: "auto", paddingRight: "4px" }}>
                {[...amistades.amigos]
                  .sort((a, b) => a.nombre_usuario.toLowerCase().localeCompare(b.nombre_usuario.toLowerCase()))
                  .map((a) => (
                    <div key={a.amistad_id} style={rowStyle}>
                      <span><strong>{a.nombre_visible}</strong> (@{a.nombre_usuario})</span>
                      <button type="button" onClick={() => setAmigoPorEliminar(a)} style={{ fontSize: "12px", fontWeight: 700, color: "#DC2626", backgroundColor: "#FEE2E2", padding: "5px 10px", borderRadius: "6px" }}>Eliminar</button>
                    </div>
                  ))}
              </div>
            )}
          </div>
        </div>
      )}
      {/* VISTA GRUPOS */}
      {subVista === "grupos" && (
        <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
          {mostrarFormGrupo && (
            <form
              onSubmit={(e) => {
                e.preventDefault();
                ejecutarAccion(async () => {
                  const creado = await crearGrupo({
                    nombre: formGrupo.nombre.trim(),
                    moneda: formGrupo.moneda,
                    descripcion: formGrupo.descripcion.trim(),
                  });
                  for (const username of amigosSeleccionadosGrupo) {
                    try {
                      await invitarMiembroGrupo(creado.id, username);
                    } catch {}
                  }
                  setFormGrupo({ nombre: "", moneda: "USD", descripcion: "", guardando: false });
                  setBusquedaAmigosGrupo("");
                  setAmigosSeleccionadosGrupo([]);
                  setMostrarFormGrupo(false);
                  await cargarDatos();
                  await abrirGrupo(creado.id);
                }, amigosSeleccionadosGrupo.length > 0 ? `Grupo creado e invitaciones enviadas a ${amigosSeleccionadosGrupo.length} amigo(s).` : "Grupo creado exitosamente.");
              }}
              style={{ ...cardStyle, border: "2px solid var(--emerald-primary)", display: "flex", flexDirection: "column", gap: "14px" }}
            >
              <h3 style={{ fontSize: "16px", fontWeight: 800 }}>Crear Nuevo Grupo</h3>
              <div style={{ display: "grid", gridTemplateColumns: "2fr 1fr", gap: "12px" }}>
                <input type="text" required value={formGrupo.nombre} onChange={(e) => setFormGrupo({ ...formGrupo, nombre: e.target.value })} placeholder="Nombre del grupo (ej. Viaje a Mérida)" className="form-input" />
                <select value={formGrupo.moneda} onChange={(e) => setFormGrupo({ ...formGrupo, moneda: e.target.value })} className="form-select">
                  {monedas.map((m) => <option key={m.codigo} value={m.codigo}>{m.codigo} — {m.nombre}</option>)}
                </select>
              </div>
              <input type="text" value={formGrupo.descripcion} onChange={(e) => setFormGrupo({ ...formGrupo, descripcion: e.target.value })} placeholder="Descripción opcional" className="form-input" />
              {/* Selector de amigos con barra de búsqueda parcial */}
              <div style={{ borderTop: "1px solid var(--border-subtle)", paddingTop: "12px", display: "flex", flexDirection: "column", gap: "10px" }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <label style={{ fontSize: "13.5px", fontWeight: 700, color: "var(--text-primary)" }}>
                    Invitar amigos al grupo ({amigosSeleccionadosGrupo.length} seleccionado{amigosSeleccionadosGrupo.length === 1 ? "" : "s"})
                  </label>
                  <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
                    Búsqueda parcial entre tus amigos
                  </span>
                </div>
                <input
                  type="text"
                  value={busquedaAmigosGrupo}
                  onChange={(e) => setBusquedaAmigosGrupo(e.target.value)}
                  placeholder="🔍 Buscar en tu lista de amigos por nombre o @usuario..."
                  className="form-input"
                />
                {amistades.amigos.length === 0 ? (
                  <p style={{ fontSize: "12.5px", color: "var(--text-secondary)" }}>
                    Aún no tienes amigos agregados en tu lista para invitar. Puedes crear el grupo e invitarlos más adelante.
                  </p>
                ) : (
                  <div style={{ display: "flex", flexDirection: "column", gap: "6px", maxHeight: "200px", overflowY: "auto", paddingRight: "4px" }}>
                    {[...amistades.amigos]
                      .filter((a) => {
                        const q = busquedaAmigosGrupo.trim().toLowerCase().replace(/^@/, "");
                        if (!q) return true;
                        return (
                          a.nombre_usuario.toLowerCase().includes(q) ||
                          a.nombre_visible.toLowerCase().includes(q)
                        );
                      })
                      .sort((a, b) => a.nombre_usuario.toLowerCase().localeCompare(b.nombre_usuario.toLowerCase()))
                      .map((a) => {
                        const seleccionado = amigosSeleccionadosGrupo.includes(a.nombre_usuario);
                        return (
                          <label
                            key={a.amistad_id}
                            style={{
                              ...rowStyle,
                              cursor: "pointer",
                              backgroundColor: seleccionado ? "#ECFDF5" : "var(--surface-card)",
                              borderColor: seleccionado ? "var(--emerald-primary)" : "var(--border-subtle)",
                            }}
                          >
                            <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                              <input
                                type="checkbox"
                                checked={seleccionado}
                                onChange={(e) =>
                                  setAmigosSeleccionadosGrupo((prev) =>
                                    e.target.checked
                                      ? [...prev, a.nombre_usuario]
                                      : prev.filter((u) => u !== a.nombre_usuario)
                                  )
                                }
                              />
                              <div>
                                <strong style={{ fontSize: "13.5px" }}>{a.nombre_visible}</strong>{" "}
                                <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>@{a.nombre_usuario}</span>
                              </div>
                            </div>
                            <span style={{ fontSize: "11.5px", fontWeight: 700, color: seleccionado ? "var(--emerald-primary)" : "var(--text-secondary)" }}>
                              {seleccionado ? "Se enviará invitación" : "Clic para seleccionar"}
                            </span>
                          </label>
                        );
                      })}
                  </div>
                )}
              </div>
              <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
                <button type="button" onClick={() => setMostrarFormGrupo(false)} className="btn btn-secondary">Cancelar</button>
                <button type="submit" className="btn btn-primary">Guardar Grupo</button>
              </div>
            </form>
          )}
          {listadoGrupos.invitaciones_recibidas.length > 0 && (
            <div style={cardStyle}>
              <h3 style={{ fontSize: "14px", fontWeight: 800, marginBottom: "10px" }}>Invitaciones a Grupos Pendientes</h3>
              {listadoGrupos.invitaciones_recibidas.map((inv) => (
                <div key={inv.id} style={rowStyle}>
                  <span>Invitación al grupo #{inv.grupo_id.slice(0, 8)}</span>
                  <div style={{ display: "flex", gap: "8px" }}>
                    <button type="button" className="btn btn-secondary" onClick={() => ejecutarAccion(async () => { await responderInvitacionGrupo(inv.grupo_id, false); await cargarDatos(); }, "Invitación rechazada.")}>Rechazar</button>
                    <button type="button" className="btn btn-primary" onClick={() => ejecutarAccion(async () => { await responderInvitacionGrupo(inv.grupo_id, true); await cargarDatos(); }, "Te uniste al grupo.")}>Aceptar</button>
                  </div>
                </div>
              ))}
            </div>
          )}
          {listadoGrupos.grupos.length === 0 ? (
            <div style={{ ...cardStyle, textAlign: "center", padding: "40px 20px" }}>
              <p style={{ fontSize: "14px", color: "var(--text-secondary)", marginBottom: "14px" }}>No perteneces a ningún grupo todavía.</p>
              <button type="button" onClick={() => setMostrarFormGrupo(true)} className="btn btn-primary">+ Crear mi primer grupo</button>
            </div>
          ) : (
            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(260px, 1fr))", gap: "14px" }}>
              {listadoGrupos.grupos.map((g) => (
                <div key={g.id} onClick={() => abrirGrupo(g.id)} style={{ ...cardStyle, cursor: "pointer" }}>
                  <div style={{ display: "flex", justifyContent: "space-between", fontWeight: 800 }}>
                    <span>{g.nombre_grupo}</span>
                    <span style={{ color: "var(--emerald-primary)" }}>{g.moneda}</span>
                  </div>
                  <p style={{ fontSize: "12.5px", color: "var(--text-secondary)", marginTop: "6px" }}>{g.descripcion || "Sin descripción"}</p>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
      {/* DETALLE DE GRUPO */}
      {subVista === "detalle_grupo" && grupoActivo && (
        <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
          {mostrarFormGasto && (
            <form onSubmit={handleGuardarGasto} style={{ ...cardStyle, border: "2px solid var(--emerald-primary)", display: "flex", flexDirection: "column", gap: "14px" }}>
              <h3 style={{ fontSize: "16px", fontWeight: 800 }}>Registrar Gasto Compartido ({grupoActivo.moneda})</h3>
              <div style={{ display: "grid", gridTemplateColumns: "2fr 1fr 1fr", gap: "10px" }}>
                <input type="text" required value={conceptoGasto} onChange={(e) => setConceptoGasto(e.target.value)} placeholder="Concepto (ej. Cena)" className="form-input" />
                <input type="text" required value={montoGasto} onChange={(e) => setMontoGasto(e.target.value)} placeholder="Monto (ej. 100.00)" className="form-input" />
                <select value={pagadorId} onChange={(e) => setPagadorId(e.target.value)} className="form-select">
                  {grupoActivo.miembros.map((m) => <option key={m.usuario_id} value={m.usuario_id}>Pagó: {m.nombre_visible}</option>)}
                </select>
              </div>
              <div style={{ display: "flex", gap: "8px" }}>
                {(["IGUALES", "PORCENTAJES", "MONTOS"] as const).map((r) => (
                  <button key={r} type="button" onClick={() => setReglaDivision(r)} className={reglaDivision === r ? "btn btn-primary" : "btn btn-secondary"} style={{ fontSize: "12px", padding: "6px 12px" }}>
                    {r === "IGUALES" ? "Partes iguales" : r === "PORCENTAJES" ? "Porcentajes (%)" : "Montos fijos"}
                  </button>
                ))}
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: "6px", maxHeight: "220px", overflowY: "auto", paddingRight: "4px" }}>
                {grupoActivo.miembros.map((m) => {
                  const inc = participantesIds.includes(m.usuario_id);
                  return (
                    <div key={m.usuario_id} style={rowStyle}>
                      <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer" }}>
                        <input type="checkbox" checked={inc} onChange={(e) => setParticipantesIds((prev) => e.target.checked ? [...prev, m.usuario_id] : prev.filter((id) => id !== m.usuario_id))} />
                        <span>{m.nombre_visible} (@{m.nombre_usuario})</span>
                      </label>
                      {inc && reglaDivision !== "IGUALES" && (
                        <input type="text" value={valoresCustom[m.usuario_id] ?? ""} onChange={(e) => setValoresCustom({ ...valoresCustom, [m.usuario_id]: e.target.value })} placeholder={reglaDivision === "PORCENTAJES" ? "%" : grupoActivo.moneda} className="form-input" style={{ width: "95px", textAlign: "right" }} />
                      )}
                    </div>
                  );
                })}
              </div>
              <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
                <button type="button" onClick={() => setMostrarFormGasto(false)} className="btn btn-secondary">Cancelar</button>
                <button type="submit" disabled={guardandoGasto} className="btn btn-primary">{guardandoGasto ? "Guardando..." : "Guardar Gasto"}</button>
              </div>
            </form>
          )}
          <div style={{ display: "flex", gap: "8px" }}>
            <button type="button" onClick={() => setTabGrupo("gastos")} className={tabGrupo === "gastos" ? "btn btn-primary" : "btn btn-secondary"}>Gastos ({grupoActivo.gastos.length})</button>
            <button type="button" onClick={() => setTabGrupo("balances")} className={tabGrupo === "balances" ? "btn btn-primary" : "btn btn-secondary"}>Balances</button>
            <button type="button" onClick={() => setTabGrupo("miembros")} className={tabGrupo === "miembros" ? "btn btn-primary" : "btn btn-secondary"}>Miembros ({grupoActivo.miembros.length})</button>
          </div>
          {tabGrupo === "gastos" && (
            <div style={cardStyle}>
              {grupoActivo.gastos.length === 0 ? (
                <p style={{ fontSize: "13px", color: "var(--text-secondary)" }}>No hay gastos registrados en este grupo.</p>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
                  {grupoActivo.gastos.map((g) => {
                    const pagador = grupoActivo.miembros.find((m) => m.usuario_id === g.pagador_id);
                    const cuotasOrdenadas = [...g.cuotas].sort((a, b) =>
                      a.nombre_usuario.toLowerCase().localeCompare(b.nombre_usuario.toLowerCase())
                    );
                    return (
                      <div
                        key={g.id}
                        style={{
                          padding: "18px 20px",
                          borderRadius: "12px",
                          border: "1px solid var(--border-subtle)",
                          backgroundColor: "#FFFFFF",
                          boxShadow: "0 1px 3px rgba(0, 0, 0, 0.03)",
                          display: "flex",
                          flexDirection: "column",
                          gap: "14px",
                        }}
                      >
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "12px" }}>
                          <div>
                            <div style={{ fontSize: "16px", fontWeight: 800, color: "var(--text-primary)" }}>
                              {g.concepto}
                            </div>
                            <div style={{ fontSize: "12.5px", color: "var(--text-secondary)", marginTop: "3px" }}>
                              Pagado por <strong>{pagador?.nombre_visible || `@${pagador?.nombre_usuario || "miembro"}`}</strong>
                              {g.fecha ? ` • ${g.fecha.split("T")[0]}` : ""}
                            </div>
                          </div>
                          <div
                            style={{
                              backgroundColor: "#ECFDF5",
                              border: "1px solid #A7F3D0",
                              padding: "6px 14px",
                              borderRadius: "10px",
                              textAlign: "right",
                            }}
                          >
                            <div style={{ fontSize: "10px", fontWeight: 700, color: "#065F46", letterSpacing: "0.5px" }}>
                              TOTAL
                            </div>
                            <div style={{ fontSize: "16px", fontWeight: 800, color: "var(--emerald-primary)" }}>
                              {formatearMonto(Number(g.monto), grupoActivo.moneda)}
                            </div>
                          </div>
                        </div>
                        <div
                          style={{
                            paddingTop: "12px",
                            borderTop: "1px solid var(--border-subtle)",
                            display: "grid",
                            gridTemplateColumns: "repeat(auto-fill, minmax(175px, 1fr))",
                            gap: "10px",
                          }}
                        >
                          {cuotasOrdenadas.map((c) => {
                            const miembroInfo = grupoActivo.miembros.find((m) => m.usuario_id === c.usuario_id);
                            return (
                              <div
                                key={`${g.id}-${c.usuario_id}`}
                                style={{
                                  display: "flex",
                                  justifyContent: "space-between",
                                  alignItems: "center",
                                  padding: "9px 12px",
                                  borderRadius: "9px",
                                  backgroundColor: "var(--surface-hover)",
                                  border: "1px solid var(--border-subtle)",
                                }}
                              >
                                <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                                  <div
                                    style={{
                                      width: "24px",
                                      height: "24px",
                                      borderRadius: "50%",
                                      backgroundColor: "#E2E8F0",
                                      color: "var(--text-primary)",
                                      display: "flex",
                                      alignItems: "center",
                                      justifyContent: "center",
                                      fontSize: "11px",
                                      fontWeight: 800,
                                    }}
                                  >
                                    {(miembroInfo?.nombre_visible?.[0] || c.nombre_usuario[0] || "U").toUpperCase()}
                                  </div>
                                  <div style={{ lineHeight: 1.15 }}>
                                    <div style={{ fontSize: "12.5px", fontWeight: 700, color: "var(--text-primary)" }}>
                                      {miembroInfo?.nombre_visible || `@${c.nombre_usuario}`}
                                    </div>
                                    <div style={{ fontSize: "11px", color: "var(--text-secondary)" }}>
                                      @{c.nombre_usuario}
                                    </div>
                                  </div>
                                </div>
                                <span style={{ fontSize: "13.5px", fontWeight: 800, color: "var(--text-primary)" }}>
                                  {formatearMonto(Number(c.monto), grupoActivo.moneda)}
                                </span>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}
          {tabGrupo === "balances" && (
            <div style={cardStyle}>
              <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
                {[...grupoActivo.miembros]
                  .sort((a, b) => a.nombre_usuario.toLowerCase().localeCompare(b.nombre_usuario.toLowerCase()))
                  .map((m) => {
                    const bal = Number(m.balance);
                    const esPositivo = bal > 0.001;
                    const esNegativo = bal < -0.001;
                    return (
                      <div key={m.usuario_id} style={{ ...rowStyle, padding: "12px 16px" }}>
                        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                          <div
                            style={{
                              width: "30px",
                              height: "30px",
                              borderRadius: "50%",
                              backgroundColor: esPositivo ? "#ECFDF5" : esNegativo ? "#FEF2F2" : "var(--surface-hover)",
                              color: esPositivo ? "#059669" : esNegativo ? "#DC2626" : "var(--text-secondary)",
                              display: "flex",
                              alignItems: "center",
                              justifyContent: "center",
                              fontSize: "12px",
                              fontWeight: 800,
                            }}
                          >
                            {(m.nombre_visible[0] || m.nombre_usuario[0] || "U").toUpperCase()}
                          </div>
                          <div>
                            <div style={{ fontWeight: 700, fontSize: "14px" }}>{m.nombre_visible}</div>
                            <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>@{m.nombre_usuario}</div>
                          </div>
                        </div>
                        <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
                          <span
                            style={{
                              fontSize: "11.5px",
                              fontWeight: 700,
                              padding: "3px 9px",
                              borderRadius: "12px",
                              backgroundColor: esPositivo ? "#ECFDF5" : esNegativo ? "#FEF2F2" : "var(--surface-hover)",
                              color: esPositivo ? "#059669" : esNegativo ? "#DC2626" : "var(--text-secondary)",
                            }}
                          >
                            {esPositivo ? "A favor" : esNegativo ? "Por pagar" : "Al día"}
                          </span>
                          <span
                            style={{
                              fontWeight: 800,
                              fontSize: "15px",
                              minWidth: "90px",
                              textAlign: "right",
                              color: esPositivo ? "#059669" : esNegativo ? "#DC2626" : "var(--text-primary)",
                            }}
                          >
                            {esPositivo ? "+" : ""}
                            {formatearMonto(bal, grupoActivo.moneda)}
                          </span>
                        </div>
                      </div>
                    );
                  })}
              </div>
            </div>
          )}
          {tabGrupo === "miembros" && (
            <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
              <div style={{ ...cardStyle, display: "flex", flexDirection: "column", gap: "12px" }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <h3 style={{ fontSize: "15px", fontWeight: 800 }}>Invitar Amigos al Grupo</h3>
                  <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>Búsqueda parcial entre tus amigos</span>
                </div>
                <input
                  type="text"
                  value={busquedaAmigosMiembros}
                  onChange={(e) => setBusquedaAmigosMiembros(e.target.value)}
                  placeholder="🔍 Buscar en tu lista de amigos por nombre o @usuario..."
                  className="form-input"
                />
                {amistades.amigos.length === 0 ? (
                  <p style={{ fontSize: "13px", color: "var(--text-secondary)" }}>
                    Aún no tienes amigos en tu lista. Agrégalos primero en la pestaña Amigos.
                  </p>
                ) : (
                  <div style={{ display: "flex", flexDirection: "column", gap: "8px", maxHeight: "220px", overflowY: "auto" }}>
                    {[...amistades.amigos]
                      .filter((a) => {
                        const q = busquedaAmigosMiembros.trim().toLowerCase().replace(/^@/, "");
                        if (!q) return true;
                        return (
                          a.nombre_usuario.toLowerCase().includes(q) ||
                          a.nombre_visible.toLowerCase().includes(q)
                        );
                      })
                      .sort((a, b) => a.nombre_usuario.toLowerCase().localeCompare(b.nombre_usuario.toLowerCase()))
                      .map((a) => {
                        const yaEsMiembro = grupoActivo.miembros.some(
                          (m) => m.nombre_usuario.toLowerCase() === a.nombre_usuario.toLowerCase()
                        );
                        return (
                          <div key={a.amistad_id} style={rowStyle}>
                            <div>
                              <strong>{a.nombre_visible}</strong>{" "}
                              <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>@{a.nombre_usuario}</span>
                            </div>
                            {yaEsMiembro ? (
                              <span style={{ fontSize: "12px", fontWeight: 700, color: "#059669", backgroundColor: "#ECFDF5", padding: "4px 10px", borderRadius: "10px" }}>
                                Ya en el grupo
                              </span>
                            ) : (
                              <button
                                type="button"
                                className="btn btn-primary"
                                style={{ fontSize: "12.5px", padding: "6px 12px" }}
                                onClick={() =>
                                  ejecutarAccion(async () => {
                                    await invitarMiembroGrupo(grupoActivo.id, a.nombre_usuario);
                                    setGrupoActivo(await obtenerDetalleGrupo(grupoActivo.id));
                                  }, `Invitación enviada a @${a.nombre_usuario}.`)
                                }
                              >
                                Invitar al grupo
                              </button>
                            )}
                          </div>
                        );
                      })}
                  </div>
                )}
              </div>
              <div style={{ ...cardStyle, display: "flex", flexDirection: "column", gap: "10px" }}>
                <h3 style={{ fontSize: "15px", fontWeight: 800 }}>Miembros Actuales ({grupoActivo.miembros.length})</h3>
                <div style={{ display: "flex", flexDirection: "column", gap: "8px", maxHeight: "240px", overflowY: "auto", paddingRight: "4px" }}>
                  {[...grupoActivo.miembros]
                    .sort((a, b) => a.nombre_usuario.toLowerCase().localeCompare(b.nombre_usuario.toLowerCase()))
                    .map((m) => (
                      <div key={m.usuario_id} style={rowStyle}>
                        <span><strong>{m.nombre_visible}</strong> (@{m.nombre_usuario})</span>
                        <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>{m.es_administrador ? "Administrador" : "Miembro"}</span>
                      </div>
                    ))}
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
