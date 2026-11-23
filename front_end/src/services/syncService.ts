import { invoke } from "@tauri-apps/api/core";
import { formatearMontoConSigno } from "../utils/formato";

const BACKEND_URL = "http://127.0.0.1:8000";
const STORAGE_KEY_TOKEN = "cuentas_claras_session_token";
const STORAGE_KEY_LIBRO_LOCAL = "cuentas_claras_libro_local_v1";

// Claves de almacenamiento local en el dispositivo por sesión social para permitir
// demostrar múltiples usuarios (@ana, @beto, @carla) en el mismo navegador sin colisionar cursores,
// manteniendo el libro personal (STORAGE_KEY_LIBRO_LOCAL) compartido en el dispositivo (INV-16 / RES-02).
function obtenerSufijoUsuarioSync(): string {
  if (typeof window === "undefined") return "anon";
  try {
    const raw = localStorage.getItem("cuentas_claras_session_user");
    if (raw) {
      const u = JSON.parse(raw);
      if (u?.id) return String(u.id);
    }
  } catch {}
  return "anon";
}

const keyCuentasPorGrupo = () => `cuentas_claras_cuentas_por_grupo_v1_${obtenerSufijoUsuarioSync()}`;
const keyPartesPendientes = () => `cuentas_claras_partes_pendientes_v1_${obtenerSufijoUsuarioSync()}`;
const keyEventosAplicados = () => `cuentas_claras_eventos_servidor_aplicados_v1_${obtenerSufijoUsuarioSync()}`;
const keyCursorSync = () => `cuentas_claras_sync_cursor_v1_${obtenerSufijoUsuarioSync()}`;
const keyUltimaSync = () => `cuentas_claras_ultima_sync_v1_${obtenerSufijoUsuarioSync()}`;
const keyColaOffline = () => `cuentas_claras_cola_gastos_offline_v1_${obtenerSufijoUsuarioSync()}`;
const keyRechazosCola = () => `cuentas_claras_rechazos_cola_offline_v1_${obtenerSufijoUsuarioSync()}`;

const enTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export interface CuentaPorGrupo {
  grupo_id: string;
  cuenta_id_personal: string;
  moneda: string;
  actualizado_en: string;
}

export interface PartePendienteLocal {
  id: string;
  id_evento_servidor: string;
  posicion: number;
  grupo_id: string;
  nombre_grupo: string;
  gasto_id: string;
  concepto: string;
  monto: string;
  moneda: string;
  fecha: string;
  incluye_redondeo: boolean;
  recibido_en: string;
}

export interface OperacionGastoOffline {
  id: string;
  idempotency_key: string;
  grupo_id: string;
  concepto: string;
  monto: string;
  pagador_id: string;
  regla_division: "IGUALES" | "PORCENTAJES" | "MONTOS";
  participantes: string[];
  valores?: Record<string, string>;
  creado_en: string;
  estado: "PENDIENTE" | "ERROR" | "ENVIADO";
  error_mensaje?: string | null;
}

export interface RechazoColaOffline {
  id: string;
  grupo_id: string;
  concepto: string;
  monto: string;
  motivo: string;
  fecha: string;
}

export interface EventoCambioServidor {
  id: string;
  posicion: number;
  usuario_id: string;
  grupo_id: string;
  nombre_grupo: string;
  gasto_id: string;
  concepto: string;
  fecha: string;
  moneda: string;
  monto_parte: number | string;
  incluye_redondeo: boolean;
}

export interface ResultadoSincronizacion {
  eventos_aplicados: number;
  partes_pendientes: number;
  nuevo_cursor: number;
  quedan_mas: boolean;
  operaciones_offline_despachadas: number;
  rechazos_cola: RechazoColaOffline[];
  ultima_sincronizacion: string;
}

export interface ResultadoElegirCuenta {
  grupo_id: string;
  cuenta_id: string;
  partes_aplicadas: number;
}

function generarUUID(): string {
  if (typeof crypto !== "undefined" && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return `uuid-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

function headersAuth(idempotencyKey?: string): Record<string, string> {
  const token = typeof window !== "undefined" ? localStorage.getItem(STORAGE_KEY_TOKEN) : null;
  return {
    "Content-Type": "application/json",
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...(idempotencyKey ? { "Idempotency-Key": idempotencyKey } : {}),
  };
}

// ============================================================================
// Helpers de Lectura/Escritura Local en Modo Web (Dispositivo Local)
// ============================================================================

function leerCuentasPorGrupoLocal(): Record<string, CuentaPorGrupo> {
  if (typeof window === "undefined") return {};
  try {
    const raw = localStorage.getItem(keyCuentasPorGrupo());
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function guardarCuentasPorGrupoLocal(mapa: Record<string, CuentaPorGrupo>): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(keyCuentasPorGrupo(), JSON.stringify(mapa));
}

function leerPartesPendientesLocal(): PartePendienteLocal[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(keyPartesPendientes());
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function guardarPartesPendientesLocal(lista: PartePendienteLocal[]): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(keyPartesPendientes(), JSON.stringify(lista));
}

function leerEventosAplicadosLocal(): Record<string, string> {
  if (typeof window === "undefined") return {};
  try {
    const raw = localStorage.getItem(keyEventosAplicados());
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function guardarEventosAplicadosLocal(mapa: Record<string, string>): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(keyEventosAplicados(), JSON.stringify(mapa));
}

export function obtenerCursorLocal(): number {
  if (typeof window === "undefined") return 0;
  const val = parseInt(localStorage.getItem(keyCursorSync()) || "0", 10);
  return isNaN(val) || val < 0 ? 0 : val;
}

function guardarCursorLocal(cursor: number): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(keyCursorSync(), String(cursor));
}

export function obtenerUltimaSincronizacion(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(keyUltimaSync());
}

function guardarUltimaSincronizacion(iso: string): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(keyUltimaSync(), iso);
}

function leerColaOfflineLocal(): OperacionGastoOffline[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(keyColaOffline());
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function guardarColaOfflineLocal(cola: OperacionGastoOffline[]): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(keyColaOffline(), JSON.stringify(cola));
}

export function obtenerRechazosColaLocal(): RechazoColaOffline[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(keyRechazosCola());
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

export function descartarRechazoColaLocal(idRechazo: string): void {
  if (typeof window === "undefined") return;
  const actuales = obtenerRechazosColaLocal().filter((r) => r.id !== idRechazo);
  localStorage.setItem(keyRechazosCola(), JSON.stringify(actuales));
}

function agregarRechazoColaLocal(rechazo: RechazoColaOffline): void {
  if (typeof window === "undefined") return;
  const actuales = obtenerRechazosColaLocal();
  actuales.push(rechazo);
  localStorage.setItem(keyRechazosCola(), JSON.stringify(actuales));
}

/**
 * Aplica de forma atómica una lista de partes de gastos de grupo al libro personal local en modo Web,
 * verificando idempotencia estricta por `id_evento_servidor` (INV-13, INV-15).
 */
function aplicarPartesEnLibroLocalWeb(
  partes: Array<{
    id_evento_servidor: string;
    cuenta_id: string;
    nombre_grupo: string;
    concepto: string;
    monto_num: number;
    moneda: string;
    fecha: string;
  }>,
  nuevoCursorOpcional?: number
): number {
  if (typeof window === "undefined") return 0;

  const eventosAplicados = leerEventosAplicadosLocal();
  const rawLibro =
    localStorage.getItem(STORAGE_KEY_LIBRO_LOCAL) ||
    localStorage.getItem("cuentas_claras_libro_v2_sin_sesion");

  const libro = rawLibro
    ? JSON.parse(rawLibro)
    : {
        cuentas: [],
        saldosIniciales: {},
        movimientosPorCuenta: {},
        categoriasGasto: [],
        categoriasIngreso: [],
      };

  if (!libro.movimientosPorCuenta) {
    libro.movimientosPorCuenta = {};
  }

  let aplicadosCount = 0;

  for (const p of partes) {
    // INV-13 / INV-15: Si el id_evento_servidor ya fue aplicado antes, no hace nada
    if (eventosAplicados[p.id_evento_servidor]) {
      continue;
    }

    // Verificar que la cuenta exista en el libro local y coincida la moneda (INV-18)
    const cuenta = Array.isArray(libro.cuentas)
      ? libro.cuentas.find((c: any) => c.id === p.cuenta_id && c.moneda === p.moneda)
      : null;

    if (!cuenta) {
      continue;
    }

    const movId = `mov-grp-${p.id_evento_servidor}`;
    const fechaCorta = (p.fecha || new Date().toISOString()).split("T")[0];
    const conceptoCompuesto = `${p.nombre_grupo.trim()} · ${p.concepto.trim()}`;

    const nuevoMovimiento = {
      id: movId,
      id_cuenta: p.cuenta_id,
      tipo: "gasto" as const,
      monto_num: p.monto_num,
      monto_formateado: formatearMontoConSigno(p.monto_num, p.moneda, "gasto"),
      fecha: fechaCorta,
      categoria: "Gasto de grupo",
      nota: conceptoCompuesto,
      estado: "vigente" as const,
      es_gasto_grupo: true,
      id_evento_servidor: p.id_evento_servidor,
    };

    const listaActual = Array.isArray(libro.movimientosPorCuenta[p.cuenta_id])
      ? libro.movimientosPorCuenta[p.cuenta_id]
      : [];

    if (!listaActual.some((m: any) => m.id_evento_servidor === p.id_evento_servidor || m.id === movId)) {
      libro.movimientosPorCuenta[p.cuenta_id] = [nuevoMovimiento, ...listaActual];
    }

    eventosAplicados[p.id_evento_servidor] = movId;
    aplicadosCount += 1;
  }

  // Guardado atómico en localStorage: libro + registro de eventos aplicados + cursor (Sección 2.3)
  localStorage.setItem(STORAGE_KEY_LIBRO_LOCAL, JSON.stringify(libro));
  guardarEventosAplicadosLocal(eventosAplicados);
  if (typeof nuevoCursorOpcional === "number") {
    guardarCursorLocal(nuevoCursorOpcional);
  }

  // Notificar al hook useCuentas para que refresque en vivo las cuentas y saldos
  window.dispatchEvent(new CustomEvent("cuentas-claras:libro-actualizado"));

  return aplicadosCount;
}

// ============================================================================
// 1. Cuenta del Grupo en el Libro Personal (Sección 2.2 / INV-18)
// ============================================================================

export async function obtenerCuentaGrupo(grupoId: string): Promise<CuentaPorGrupo | null> {
  if (enTauri()) {
    try {
      const res = await invoke<CuentaPorGrupo | null>("obtener_cuenta_grupo", { grupoId }).catch(() =>
        invoke<CuentaPorGrupo | null>("obtener_cuenta_grupo", { grupo_id: grupoId })
      );
      if (res) return res;
    } catch {
      // Fallback a almacenamiento local
    }
  }
  const mapa = leerCuentasPorGrupoLocal();
  return mapa[grupoId] || null;
}

export async function elegirCuentaGrupo(
  grupoId: string,
  cuentaId: string,
  monedaGrupo: string
): Promise<ResultadoElegirCuenta> {
  const ahora = new Date().toISOString();

  if (enTauri()) {
    try {
      const res = await invoke<ResultadoElegirCuenta>("elegir_cuenta_grupo", {
        grupoId,
        cuentaId,
        moneda: monedaGrupo,
      }).catch(() =>
        invoke<ResultadoElegirCuenta>("elegir_cuenta_grupo", {
          grupo_id: grupoId,
          cuenta_id: cuentaId,
          moneda: monedaGrupo,
        })
      );
      const mapa = leerCuentasPorGrupoLocal();
      mapa[grupoId] = {
        grupo_id: grupoId,
        cuenta_id_personal: cuentaId,
        moneda: monedaGrupo,
        actualizado_en: ahora,
      };
      guardarCuentasPorGrupoLocal(mapa);
      return res;
    } catch {
      // Fallback a lógica local web si el comando falla
    }
  }

  // Guardar elección en el dispositivo (INV-16 / INV-18)
  const mapa = leerCuentasPorGrupoLocal();
  mapa[grupoId] = {
    grupo_id: grupoId,
    cuenta_id_personal: cuentaId,
    moneda: monedaGrupo,
    actualizado_en: ahora,
  };
  guardarCuentasPorGrupoLocal(mapa);

  // Si había partes pendientes de aplicar para este grupo, aplicarlas inmediatamente (Sección 2.2)
  const pendientes = leerPartesPendientesLocal();
  const delGrupo = pendientes.filter((p) => p.grupo_id === grupoId && p.moneda === monedaGrupo);
  const restantes = pendientes.filter((p) => !(p.grupo_id === grupoId && p.moneda === monedaGrupo));

  let partesAplicadas = 0;
  if (delGrupo.length > 0) {
    partesAplicadas = aplicarPartesEnLibroLocalWeb(
      delGrupo.map((p) => ({
        id_evento_servidor: p.id_evento_servidor,
        cuenta_id: cuentaId,
        nombre_grupo: p.nombre_grupo,
        concepto: p.concepto,
        monto_num: parseFloat(String(p.monto).replace(",", ".")) || 0,
        moneda: p.moneda,
        fecha: p.fecha,
      }))
    );
    guardarPartesPendientesLocal(restantes);
  }

  return {
    grupo_id: grupoId,
    cuenta_id: cuentaId,
    partes_aplicadas: partesAplicadas,
  };
}

export async function listarPartesPendientes(): Promise<PartePendienteLocal[]> {
  if (enTauri()) {
    try {
      const res = await invoke<PartePendienteLocal[]>("listar_partes_pendientes");
      if (Array.isArray(res)) return res;
    } catch {
      // Fallback
    }
  }
  return leerPartesPendientesLocal();
}

// ============================================================================
// 2. Cola de Gastos de Grupo sin Conexión (Sección 2.5 / RNF-17 / ESC-09)
// ============================================================================

export async function listarColaOffline(): Promise<OperacionGastoOffline[]> {
  if (enTauri()) {
    try {
      const res = await invoke<OperacionGastoOffline[]>("listar_cola_offline");
      if (Array.isArray(res)) return res;
    } catch {
      // Fallback
    }
  }
  return leerColaOfflineLocal();
}

export async function encolarGastoOffline(entrada: {
  grupo_id: string;
  concepto: string;
  monto: string;
  pagador_id: string;
  regla_division: "IGUALES" | "PORCENTAJES" | "MONTOS";
  participantes: string[];
  valores?: Record<string, string>;
  idempotency_key?: string;
}): Promise<OperacionGastoOffline> {
  if (enTauri()) {
    try {
      const res = await invoke<OperacionGastoOffline>("encolar_gasto_offline", {
        entrada: {
          grupo_id: entrada.grupo_id,
          concepto: entrada.concepto,
          monto: entrada.monto,
          pagador_id: entrada.pagador_id,
          regla_division: entrada.regla_division,
          participantes: entrada.participantes,
          valores: entrada.valores || null,
        },
      });
      return res;
    } catch {
      // Fallback a cola local
    }
  }

  const nuevaOp: OperacionGastoOffline = {
    id: generarUUID(),
    idempotency_key: entrada.idempotency_key || generarUUID(),
    grupo_id: entrada.grupo_id,
    concepto: entrada.concepto,
    monto: entrada.monto,
    pagador_id: entrada.pagador_id,
    regla_division: entrada.regla_division,
    participantes: entrada.participantes,
    valores: entrada.valores,
    creado_en: new Date().toISOString(),
    estado: "PENDIENTE",
    error_mensaje: null,
  };

  const cola = leerColaOfflineLocal();
  cola.push(nuevaOp);
  guardarColaOfflineLocal(cola);
  return nuevaOp;
}

export async function despacharColaOffline(): Promise<{
  exitosas: number;
  rechazos: RechazoColaOffline[];
}> {
  if (enTauri()) {
    try {
      const res = await invoke<{ total_procesadas: number; exitosas: number; con_error: number }>(
        "despachar_cola_offline"
      );
      // Si alguna quedó en estado ERROR en SQLite, extraerla como rechazo y limpiarla para no bloquear
      const colaRestante = await listarColaOffline();
      const rechazosTauri: RechazoColaOffline[] = colaRestante
        .filter((op) => op.estado === "ERROR")
        .map((op) => ({
          id: op.id,
          grupo_id: op.grupo_id,
          concepto: op.concepto,
          monto: op.monto,
          motivo: op.error_mensaje || "Rechazado por el servidor",
          fecha: op.creado_en,
        }));
      return { exitosas: res.exitosas, rechazos: rechazosTauri };
    } catch {
      // Fallback a despacho web
    }
  }

  const cola = leerColaOfflineLocal();
  if (cola.length === 0) {
    return { exitosas: 0, rechazos: obtenerRechazosColaLocal() };
  }

  const colaPendienteAun: OperacionGastoOffline[] = [];
  let exitosas = 0;

  for (let i = 0; i < cola.length; i++) {
    const op = cola[i];
    const valoresNum: Record<string, number> | undefined = op.valores
      ? Object.fromEntries(
          Object.entries(op.valores).map(([k, v]) => [k, parseFloat(String(v).replace(",", ".")) || 0])
        )
      : undefined;

    try {
      const res = await fetch(`${BACKEND_URL}/grupos/${op.grupo_id}/gastos`, {
        method: "POST",
        headers: headersAuth(op.idempotency_key),
        body: JSON.stringify({
          concepto: op.concepto,
          monto: parseFloat(String(op.monto).replace(",", ".")),
          pagador_id: op.pagador_id,
          regla_division: op.regla_division,
          participantes: op.participantes,
          valores: op.regla_division === "IGUALES" ? undefined : valoresNum,
        }),
      });

      if (res.ok) {
        // 201 Created o 200 OK (idempotencia): sale de la cola exitosamente
        exitosas += 1;
      } else if (res.status === 401) {
        // Token vencido: se detiene el envío sin perder la operación en cola
        colaPendienteAun.push(...cola.slice(i));
        guardarColaOfflineLocal(colaPendienteAun);
        throw new Error("Tu sesión venció, inicia sesión de nuevo.");
      } else {
        // El servidor rechazó la operación (400, 403, 404, 409):
        // Sección 2.5: "Si el servidor rechaza una operación, sale de la cola y se muestra al usuario con el motivo, sin bloquear las demás."
        const data = await res.json().catch(() => ({}));
        const d = data?.detail;
        const motivo = typeof d === "string" ? d : d?.mensaje || `Operación rechazada (HTTP ${res.status})`;
        agregarRechazoColaLocal({
          id: op.id,
          grupo_id: op.grupo_id,
          concepto: op.concepto,
          monto: op.monto,
          motivo,
          fecha: new Date().toISOString(),
        });
      }
    } catch (err: any) {
      if (err?.message?.includes("venció")) {
        throw err;
      }
      // Sigue sin red: mantener esta operación y las siguientes en la cola
      colaPendienteAun.push(...cola.slice(i));
      break;
    }
  }

  guardarColaOfflineLocal(colaPendienteAun);
  return { exitosas, rechazos: obtenerRechazosColaLocal() };
}

// ============================================================================
// 3. Sincronización de Cambios por Cursor (Sección 2.1, 2.3, 2.4)
// ============================================================================

export async function sincronizarCambios(): Promise<ResultadoSincronizacion> {
  // 1. Si hay operaciones en la cola offline, enviarlas primero en orden (Sección 2.5)
  const despacho = await despacharColaOffline();

  if (enTauri()) {
    try {
      let totalAplicados = 0;
      let totalPendientes = 0;
      let cursorFinal = obtenerCursorLocal();
      let quedanMas = true;
      let iteraciones = 0;

      while (quedanMas && iteraciones < 20) {
        iteraciones += 1;
        const res = await invoke<{
          eventos_aplicados: number;
          partes_pendientes: number;
          nuevo_cursor: number;
          quedan_mas: boolean;
          operaciones_offline_despachadas: number;
        }>("sincronizar_cambios");

        totalAplicados += res.eventos_aplicados;
        totalPendientes = res.partes_pendientes;
        cursorFinal = res.nuevo_cursor;
        quedanMas = res.quedan_mas;
      }

      const ahora = new Date().toISOString();
      guardarCursorLocal(cursorFinal);
      guardarUltimaSincronizacion(ahora);
      if (typeof window !== "undefined") {
        window.dispatchEvent(new CustomEvent("cuentas-claras:libro-actualizado"));
      }

      return {
        eventos_aplicados: totalAplicados,
        partes_pendientes: totalPendientes,
        nuevo_cursor: cursorFinal,
        quedan_mas: false,
        operaciones_offline_despachadas: despacho.exitosas,
        rechazos_cola: despacho.rechazos,
        ultima_sincronizacion: ahora,
      };
    } catch (err: any) {
      const msg = typeof err === "string" ? err : err?.message || "";
      if (msg.includes("401") || msg.toLowerCase().includes("venció") || msg.toLowerCase().includes("token")) {
        throw new Error("Tu sesión venció, inicia sesión de nuevo.");
      }
      // Si Tauri no tiene token en keyring pero el navegador sí, continúa por el flujo HTTP directo
    }
  }

  // Flujo HTTP directo + almacenamiento local del dispositivo
  let cursorActual = obtenerCursorLocal();
  let quedanMas = true;
  let totalAplicados = 0;
  let iteraciones = 0;

  while (quedanMas && iteraciones < 25) {
    iteraciones += 1;
    const res = await fetch(`${BACKEND_URL}/cambios?desde=${cursorActual}`, {
      headers: headersAuth(),
    });

    if (res.status === 401) {
      throw new Error("Tu sesión venció, inicia sesión de nuevo.");
    }

    if (!res.ok) {
      const data = await res.json().catch(() => ({}));
      const d = data?.detail;
      throw new Error(typeof d === "string" ? d : d?.mensaje || "No se pudo sincronizar con el servidor.");
    }

    const lote: {
      eventos: EventoCambioServidor[];
      nuevo_cursor: number;
      quedan_mas: boolean;
    } = await res.json();

    const eventos = Array.isArray(lote.eventos) ? lote.eventos : [];
    const nuevoCursor = typeof lote.nuevo_cursor === "number" ? lote.nuevo_cursor : cursorActual;
    quedanMas = Boolean(lote.quedan_mas);

    const cuentasPorGrupo = leerCuentasPorGrupoLocal();
    const eventosAplicados = leerEventosAplicadosLocal();
    const pendientesActuales = leerPartesPendientesLocal();

    const partesParaAplicar: Array<{
      id_evento_servidor: string;
      cuenta_id: string;
      nombre_grupo: string;
      concepto: string;
      monto_num: number;
      moneda: string;
      fecha: string;
    }> = [];

    for (const ev of eventos) {
      // INV-14: Ignorar sin fallar eventos que no tengan la estructura de gasto de grupo
      if (!ev || !ev.id || !ev.grupo_id || ev.monto_parte === undefined) {
        continue;
      }

      // INV-13 / INV-15: Si ya fue aplicado anteriormente, se ignora sin duplicar
      if (eventosAplicados[ev.id]) {
        continue;
      }

      const asignacion = cuentasPorGrupo[ev.grupo_id];
      const montoNum = parseFloat(String(ev.monto_parte).replace(",", ".")) || 0;

      if (asignacion && asignacion.cuenta_id_personal && asignacion.moneda === ev.moneda) {
        partesParaAplicar.push({
          id_evento_servidor: ev.id,
          cuenta_id: asignacion.cuenta_id_personal,
          nombre_grupo: ev.nombre_grupo || "Grupo",
          concepto: ev.concepto || "Gasto compartido",
          monto_num: montoNum,
          moneda: ev.moneda,
          fecha: ev.fecha,
        });
      } else {
        // Sección 2.2: Si llega una parte de un grupo sin cuenta elegida, queda en pendientes de aplicar
        if (!pendientesActuales.some((p) => p.id_evento_servidor === ev.id)) {
          pendientesActuales.push({
            id: `pend-${ev.id}`,
            id_evento_servidor: ev.id,
            posicion: ev.posicion,
            grupo_id: ev.grupo_id,
            nombre_grupo: ev.nombre_grupo || "Grupo",
            gasto_id: ev.gasto_id,
            concepto: ev.concepto || "Gasto compartido",
            monto: String(ev.monto_parte),
            moneda: ev.moneda,
            fecha: ev.fecha,
            incluye_redondeo: Boolean(ev.incluye_redondeo),
            recibido_en: new Date().toISOString(),
          });
        }
      }
    }

    guardarPartesPendientesLocal(pendientesActuales);
    const aplicadosLote = aplicarPartesEnLibroLocalWeb(partesParaAplicar, nuevoCursor);
    totalAplicados += aplicadosLote;
    cursorActual = nuevoCursor;

    if (eventos.length === 0) {
      break;
    }
  }

  const ahora = new Date().toISOString();
  guardarUltimaSincronizacion(ahora);
  const pendientesFinales = leerPartesPendientesLocal();

  return {
    eventos_aplicados: totalAplicados,
    partes_pendientes: pendientesFinales.length,
    nuevo_cursor: cursorActual,
    quedan_mas: false,
    operaciones_offline_despachadas: despacho.exitosas,
    rechazos_cola: despacho.rechazos,
    ultima_sincronizacion: ahora,
  };
}

export async function reiniciarCursorSincronizacion(): Promise<void> {
  if (enTauri()) {
    try {
      await invoke("reiniciar_cursor");
    } catch {
      // Fallback
    }
  }
  guardarCursorLocal(0);
}
