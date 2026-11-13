export interface UsuarioPublico {
  nombre_usuario: string;
  nombre_visible: string;
}
export interface AmigoItem {
  amistad_id: string;
  nombre_usuario: string;
  nombre_visible: string;
}
export interface ListadoAmistades {
  amigos: AmigoItem[];
  invitaciones_recibidas: AmigoItem[];
  invitaciones_enviadas: AmigoItem[];
}
export interface GrupoResumen {
  id: string;
  nombre_grupo: string;
  moneda: string;
  descripcion: string;
  administrador_id: string;
}
export interface InvitacionGrupoResumen {
  id: string;
  grupo_id: string;
  invitador_id: string;
  estado: string;
}
export interface ListadoGrupos {
  grupos: GrupoResumen[];
  invitaciones_recibidas: InvitacionGrupoResumen[];
}
export interface CuotaGasto {
  usuario_id: string;
  nombre_usuario: string;
  monto: number;
  lleva_residuo: boolean;
}
export interface GastoGrupoItem {
  id: string;
  grupo_id: string;
  concepto: string;
  pagador_id: string;
  monto: number;
  fecha: string;
  cuotas: CuotaGasto[];
}
export interface MiembroGrupoDetalle {
  usuario_id: string;
  nombre_usuario: string;
  nombre_visible: string;
  es_administrador: boolean;
  balance: number;
}
export interface DetalleGrupo {
  id: string;
  nombre_grupo: string;
  moneda: string;
  descripcion: string;
  miembros: MiembroGrupoDetalle[];
  gastos: GastoGrupoItem[];
}
export interface EntradaRegistrarGastoGrupo {
  concepto: string;
  monto: number;
  pagador_id: string;
  regla_division: "IGUALES" | "PORCENTAJES" | "MONTOS";
  participantes: string[];
  valores?: Record<string, number>;
}
const BACKEND_URL = "http://127.0.0.1:8000";
const STORAGE_KEY_TOKEN = "cuentas_claras_session_token";
function headersAuth(): Record<string, string> {
  const token = typeof window !== "undefined" ? localStorage.getItem(STORAGE_KEY_TOKEN) : null;
  return {
    "Content-Type": "application/json",
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
  };
}
async function procesarRespuesta<T>(res: Response, mensajeDefault: string): Promise<T> {
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    const d = data?.detail;
    const msg = typeof d === "string" ? d : d?.mensaje || mensajeDefault;
    throw new Error(msg);
  }
  if (res.status === 204) return undefined as unknown as T;
  return await res.json();
}
export async function buscarUsuarioExacto(username: string): Promise<UsuarioPublico> {
  const limpio = username.trim().toLowerCase().replace(/^@/, "");
  const res = await fetch(`${BACKEND_URL}/usuarios?usuario=${encodeURIComponent(limpio)}`, {
    headers: headersAuth(),
  });
  return procesarRespuesta<UsuarioPublico>(res, "Usuario no encontrado.");
}
export async function listarAmistades(): Promise<ListadoAmistades> {
  const res = await fetch(`${BACKEND_URL}/amistades`, { headers: headersAuth() });
  return procesarRespuesta<ListadoAmistades>(res, "No se pudieron obtener las amistades.");
}
export async function enviarInvitacionAmistad(nombreUsuario: string): Promise<void> {
  const limpio = nombreUsuario.trim().toLowerCase().replace(/^@/, "");
  const res = await fetch(`${BACKEND_URL}/amistades`, {
    method: "POST",
    headers: headersAuth(),
    body: JSON.stringify({ nombre_usuario: limpio }),
  });
  await procesarRespuesta(res, "No se pudo enviar la invitación de amistad.");
}
export async function responderInvitacionAmistad(amistadId: string, aceptar: boolean): Promise<void> {
  const res = await fetch(`${BACKEND_URL}/amistades/${amistadId}/respuesta`, {
    method: "POST",
    headers: headersAuth(),
    body: JSON.stringify({ aceptar }),
  });
  await procesarRespuesta(res, "No se pudo procesar la solicitud de amistad.");
}
export async function eliminarAmistad(amistadId: string): Promise<void> {
  const res = await fetch(`${BACKEND_URL}/amistades/${amistadId}`, {
    method: "DELETE",
    headers: headersAuth(),
  });
  await procesarRespuesta(res, "No se pudo eliminar la amistad.");
}
export async function listarGrupos(): Promise<ListadoGrupos> {
  const res = await fetch(`${BACKEND_URL}/grupos`, { headers: headersAuth() });
  return procesarRespuesta<ListadoGrupos>(res, "No se pudieron cargar los grupos.");
}
export async function crearGrupo(entrada: {
  nombre: string;
  moneda: string;
  descripcion?: string;
}): Promise<GrupoResumen> {
  const res = await fetch(`${BACKEND_URL}/grupos`, {
    method: "POST",
    headers: headersAuth(),
    body: JSON.stringify(entrada),
  });
  return procesarRespuesta<GrupoResumen>(res, "Error al crear el grupo.");
}
export async function invitarMiembroGrupo(grupoId: string, nombreUsuario: string): Promise<void> {
  const limpio = nombreUsuario.trim().toLowerCase().replace(/^@/, "");
  const res = await fetch(`${BACKEND_URL}/grupos/${grupoId}/miembros`, {
    method: "POST",
    headers: headersAuth(),
    body: JSON.stringify({ nombre_usuario: limpio }),
  });
  await procesarRespuesta(res, "No se pudo invitar al usuario al grupo.");
}
export async function responderInvitacionGrupo(grupoId: string, aceptar: boolean): Promise<void> {
  const res = await fetch(`${BACKEND_URL}/grupos/${grupoId}/miembros/yo/respuesta`, {
    method: "POST",
    headers: headersAuth(),
    body: JSON.stringify({ aceptar }),
  });
  await procesarRespuesta(res, "No se pudo responder la invitación al grupo.");
}
export async function obtenerDetalleGrupo(grupoId: string): Promise<DetalleGrupo> {
  const res = await fetch(`${BACKEND_URL}/grupos/${grupoId}`, { headers: headersAuth() });
  return procesarRespuesta<DetalleGrupo>(res, "Grupo no encontrado.");
}
export async function registrarGastoCompartido(
  grupoId: string,
  entrada: EntradaRegistrarGastoGrupo
): Promise<GastoGrupoItem> {
  const res = await fetch(`${BACKEND_URL}/grupos/${grupoId}/gastos`, {
    method: "POST",
    headers: headersAuth(),
    body: JSON.stringify(entrada),
  });
  return procesarRespuesta<GastoGrupoItem>(res, "No se pudo registrar el gasto compartido.");
}
