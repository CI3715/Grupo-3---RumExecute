import { invoke } from "@tauri-apps/api/core";
export interface DatosUsuario {
  id: string;
  nombre_usuario: string;
  nombre_visible: string;
  correo: string;
  fecha_alta?: string;
}
export interface RespuestaSesion {
  access_token: string;
  token_type?: string;
  expires_at?: string;
  usuario: DatosUsuario;
}
export interface EntradaRegistro {
  nombre_usuario: string;
  nombre_visible: string;
  correo: string;
  contrasena: string;
}
export interface EntradaLogin {
  correo: string;
  contrasena: string;
}
const BACKEND_URL = "http://127.0.0.1:8000";
const STORAGE_KEY_TOKEN = "cuentas_claras_session_token";
const STORAGE_KEY_USUARIO = "cuentas_claras_session_user";
const MSG_SIN_CONEXION = "Sin conexión con el servidor. Tu libro personal sigue funcionando normalmente.";
const enTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
export function obtenerUsuarioCacheado(): DatosUsuario | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY_USUARIO);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}
function guardarSesionCacheada(token: string, usuario: DatosUsuario) {
  if (typeof window === "undefined") return;
  localStorage.setItem(STORAGE_KEY_TOKEN, token);
  localStorage.setItem(STORAGE_KEY_USUARIO, JSON.stringify(usuario));
}
export async function verificarEstadoServidor(): Promise<boolean> {
  try {
    if (enTauri()) {
      await invoke("ping_servidor");
      return true;
    }
    const res = await fetch(`${BACKEND_URL}/ping`, { signal: AbortSignal.timeout(1500) });
    return res.ok;
  } catch {
    return false;
  }
}
export async function registrarUsuario(entrada: EntradaRegistro): Promise<DatosUsuario> {
  if (enTauri()) {
    try {
      return await invoke<DatosUsuario>("registrar_usuario", { entrada });
    } catch (e) {
      throw new Error(String(e));
    }
  }
  let res: Response;
  try {
    res = await fetch(`${BACKEND_URL}/usuarios`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(entrada),
    });
  } catch {
    throw new Error(MSG_SIN_CONEXION);
  }
  if (!res.ok) {
    const errorData = await res.json().catch(() => ({}));
    const detalle = errorData?.detail;
    const err = new Error(
      detalle?.mensaje || (typeof detalle === "string" ? detalle : `Error ${res.status}`)
    ) as Error & { campo?: string };
    if (detalle?.campo) err.campo = detalle.campo;
    throw err;
  }
  return await res.json();
}
export async function iniciarSesion(entrada: EntradaLogin): Promise<RespuestaSesion> {
  if (enTauri()) {
    try {
      const sesion = await invoke<RespuestaSesion>("iniciar_sesion", { entrada });
      guardarSesionCacheada(sesion.access_token, sesion.usuario);
      return sesion;
    } catch (e) {
      throw new Error(String(e));
    }
  }
  let res: Response;
  try {
    res = await fetch(`${BACKEND_URL}/sesiones`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(entrada),
    });
  } catch {
    throw new Error(MSG_SIN_CONEXION);
  }
  if (!res.ok) {
    throw new Error("Correo o contraseña incorrectos.");
  }
  const sesion: RespuestaSesion = await res.json();
  guardarSesionCacheada(sesion.access_token, sesion.usuario);
  return sesion;
}
export async function obtenerPerfil(): Promise<DatosUsuario> {
  const cacheado = obtenerUsuarioCacheado();
  const token = typeof window !== "undefined" ? localStorage.getItem(STORAGE_KEY_TOKEN) : null;
  if (enTauri()) {
    try {
      const perfil = await invoke<DatosUsuario>("obtener_perfil_propio");
      if (typeof window !== "undefined") {
        localStorage.setItem(STORAGE_KEY_USUARIO, JSON.stringify(perfil));
      }
      return perfil;
    } catch (e) {
      const msg = String(e);
      if (msg.includes("venció")) {
        if (typeof window !== "undefined") {
          localStorage.removeItem(STORAGE_KEY_TOKEN);
          localStorage.removeItem(STORAGE_KEY_USUARIO);
        }
      }
      if (cacheado && !msg.includes("venció")) return cacheado;
      throw new Error(msg);
    }
  }
  if (!token) throw new Error("No hay una sesión activa guardada.");
  let res: Response;
  try {
    res = await fetch(`${BACKEND_URL}/usuarios/yo`, {
      headers: { Authorization: `Bearer ${token}` },
    });
  } catch {
    if (cacheado) return cacheado;
    throw new Error(MSG_SIN_CONEXION);
  }
  if (!res.ok) {
    if (res.status === 401) {
      if (typeof window !== "undefined") {
        localStorage.removeItem(STORAGE_KEY_TOKEN);
        localStorage.removeItem(STORAGE_KEY_USUARIO);
      }
      throw new Error("Tu sesión venció, inicia sesión de nuevo.");
    }
    if (cacheado) return cacheado;
    throw new Error("No se pudo verificar la sesión.");
  }
  const perfil: DatosUsuario = await res.json();
  guardarSesionCacheada(token, perfil);
  return perfil;
}
export async function cerrarSesion(): Promise<void> {
  if (enTauri()) {
    await invoke("cerrar_sesion").catch(() => {});
  }
  if (typeof window !== "undefined") {
    localStorage.removeItem(STORAGE_KEY_TOKEN);
    localStorage.removeItem(STORAGE_KEY_USUARIO);
  }
}
