"use client";

import { invoke } from "@tauri-apps/api/core";

const enTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export interface EventoRechazadoImportacion {
  id_evento?: string;
  motivo: string;
}

export interface ResumenImportacion {
  eventos_nuevos: number;
  eventos_existentes: number;
  eventos_rechazados: EventoRechazadoImportacion[];
  archivo?: string;
}

export interface InformeSincronizacionDispositivos {
  enviados: number;
  recibidos: number;
  rechazados: EventoRechazadoImportacion[];
  fecha_ultima_sincronizacion: string;
}

export type TipoDestinoSync = "git" | "carpeta";

export interface ConfiguracionSyncDispositivos {
  configurado: boolean;
  dispositivo_id?: string;
  tipo: TipoDestinoSync;
  ruta_o_url: string;
  usuario_git?: string;
  fecha_ultima_sincronizacion?: string | null;
}

export interface EntradaExportarCopia {
  ruta_destino?: string;
  cifrado: boolean;
  frase_paso?: string;
  confirmacion_frase_paso?: string;
}

export interface EntradaImportarCopia {
  ruta_origen?: string;
  contenido_archivo?: string;
  nombre_archivo?: string;
  frase_paso?: string;
}

export interface EntradaConfigurarSync {
  tipo: TipoDestinoSync;
  ruta_o_url: string;
  usuario_git?: string;
  token_o_clave_git?: string;
  frase_paso: string;
}

export interface EntradaActualizarCredencialesGit {
  usuario_git: string;
  token_o_clave_git: string;
}

interface ResultadoExportacionTauri {
  cantidad_eventos: number;
  cifrada: boolean;
}

interface ResultadoImportacionTauri {
  eventos_nuevos: number;
  eventos_existentes: number;
  eventos_rechazados: EventoRechazadoImportacion[];
}

interface EstadoSincronizacionPersonalTauri {
  dispositivo_id: string;
  destino: {
    tipo: TipoDestinoSync;
    ubicacion: string;
  } | null;
  fecha_ultima_sincronizacion: string | null;
}

const STORAGE_KEY_LIBRO = "cuentas_claras_libro_local_v1";
const STORAGE_KEY_SYNC_CFG = "cuentas_claras_e8_sync_config_v1";
const STORAGE_KEY_DISPOSITIVO = "cuentas_claras_e8_dispositivo_id";

function obtenerDispositivoIdLocal(): string {
  if (typeof window === "undefined") return "dispositivo-web";
  let id = localStorage.getItem(STORAGE_KEY_DISPOSITIVO);
  if (!id) {
    id =
      typeof crypto !== "undefined" && crypto.randomUUID
        ? `web-${crypto.randomUUID().slice(0, 8)}`
        : `web-${Date.now().toString(36)}`;
    localStorage.setItem(STORAGE_KEY_DISPOSITIVO, id);
  }
  return id;
}

function normalizarErrorE8(err: any, fallback: string): Error {
  const msg =
    typeof err === "string"
      ? err
      : err?.mensaje || err?.message || fallback;
  const error = new Error(msg) as Error & { codigo?: string; campo?: string };
  if (err?.codigo) error.codigo = err.codigo;
  if (err?.campo) error.campo = err.campo;
  return error;
}

export async function exportarCopiaSeguridad(
  entrada: EntradaExportarCopia
): Promise<{
  exito: boolean;
  ruta_o_nombre: string;
  cantidad_eventos: number;
  cifrado: boolean;
  contenido_descarga?: string;
}> {
  if (entrada.cifrado) {
    if (!entrada.frase_paso || entrada.frase_paso.length < 12) {
      throw new Error("La frase de paso debe tener al menos 12 caracteres.");
    }
    if (
      entrada.confirmacion_frase_paso !== undefined &&
      entrada.frase_paso !== entrada.confirmacion_frase_paso
    ) {
      throw new Error("Las dos frases de paso no coinciden.");
    }
  }

  if (enTauri()) {
    const ruta = (entrada.ruta_destino || "").trim();
    if (!ruta) {
      throw new Error("Indica la ruta donde deseas guardar la copia de seguridad.");
    }
    try {
      const res = await invoke<ResultadoExportacionTauri>("exportar_copia_seguridad", {
        ruta,
        frasePaso: entrada.cifrado ? entrada.frase_paso : null,
        confirmacionFrasePaso: entrada.cifrado
          ? (entrada.confirmacion_frase_paso ?? entrada.frase_paso)
          : null,
      });
      return {
        exito: true,
        ruta_o_nombre: ruta,
        cantidad_eventos: res.cantidad_eventos,
        cifrado: res.cifrada,
      };
    } catch (e) {
      throw normalizarErrorE8(e, "No se pudo exportar la copia de seguridad.");
    }
  }

  const rawLibro = typeof window !== "undefined" ? localStorage.getItem(STORAGE_KEY_LIBRO) : null;
  const libro = rawLibro ? JSON.parse(rawLibro) : { cuentas: [], movimientosPorCuenta: {} };
  const cuentas: any[] = libro.cuentas || [];
  const movsPorCuenta: Record<string, any[]> = libro.movimientosPorCuenta || {};
  const dispositivo = obtenerDispositivoIdLocal();
  const ahora = new Date().toISOString();

  const eventosLineas: string[] = [];
  for (const cta of cuentas) {
    eventosLineas.push(
      JSON.stringify({
        id: `evt-cta-${cta.id}`,
        tipo: "CuentaCreada",
        momento_registro: ahora,
        dispositivo_origen: dispositivo,
        datos: {
          id_cuenta: cta.id,
          nombre: cta.nombre,
          tipo: cta.tipo,
          moneda: cta.moneda,
          saldo_inicial: libro.saldosIniciales?.[cta.id] ?? 0,
        },
      })
    );
    for (const m of movsPorCuenta[cta.id] || []) {
      eventosLineas.push(
        JSON.stringify({
          id: m.id_evento_servidor ? `srv-${m.id_evento_servidor}` : `evt-mov-${m.id}`,
          tipo: m.es_gasto_grupo ? "GastoDeGrupoAplicado" : "MovimientoRegistrado",
          momento_registro: ahora,
          dispositivo_origen: dispositivo,
          datos: m,
        })
      );
    }
  }

  const textoPlano = [
    `CUENTAS_CLARAS_LIBRO v1`,
    `FECHA_EXPORTACION: ${ahora}`,
    `CANTIDAD_EVENTOS: ${eventosLineas.length}`,
    `---`,
    ...eventosLineas,
  ].join("\n");

  let contenidoFinal = textoPlano;
  if (entrada.cifrado && entrada.frase_paso) {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(textoPlano);
    const keyBytes = encoder.encode(entrada.frase_paso);
    const cifradoBytes = bytes.map((b, i) => b ^ keyBytes[i % keyBytes.length]);
    const b64 = btoa(String.fromCharCode(...cifradoBytes));
    const huella = btoa(entrada.frase_paso).slice(0, 12);
    contenidoFinal = [
      "CC_CIFRADO_V1",
      `H=${huella}`,
      b64,
    ].join("\n");
  }

  const nombre = entrada.cifrado
    ? `cuentas-claras-respaldo-${ahora.slice(0, 10)}.ccbak`
    : `cuentas-claras-respaldo-${ahora.slice(0, 10)}.txt`;

  return {
    exito: true,
    ruta_o_nombre: entrada.ruta_destino?.trim() || nombre,
    cantidad_eventos: eventosLineas.length,
    cifrado: entrada.cifrado,
    contenido_descarga: contenidoFinal,
  };
}

export async function importarCopiaSeguridad(
  entrada: EntradaImportarCopia
): Promise<ResumenImportacion> {
  if (enTauri()) {
    const ruta = (entrada.ruta_origen || "").trim();
    if (!ruta) {
      throw new Error("Indica la ruta del archivo de copia de seguridad.");
    }
    try {
      const res = await invoke<ResultadoImportacionTauri>("importar_copia_seguridad", {
        ruta,
        frasePaso: entrada.frase_paso?.trim() ? entrada.frase_paso : null,
      });
      return {
        eventos_nuevos: res.eventos_nuevos,
        eventos_existentes: res.eventos_existentes,
        eventos_rechazados: res.eventos_rechazados || [],
        archivo: ruta,
      };
    } catch (e) {
      throw normalizarErrorE8(e, "Frase de paso incorrecta o archivo dañado.");
    }
  }

  const contenido = (entrada.contenido_archivo || "").trim();
  if (!contenido) {
    throw new Error("El archivo seleccionado está vacío o no pudo leerse.");
  }

  let textoPlano = contenido;
  if (contenido.startsWith("CC_CIFRADO_V1")) {
    if (!entrada.frase_paso) {
      throw new Error("Se requiere la frase de paso para abrir esta copia cifrada.");
    }
    const lineas = contenido.split(/\r?\n/);
    const meta = lineas[1] || "";
    const matchH = meta.match(/H=([^;\s]+)/);
    const huellaEsperada = btoa(entrada.frase_paso).slice(0, 12);
    if (!matchH || matchH[1] !== huellaEsperada) {
      throw new Error("Frase de paso incorrecta o archivo dañado.");
    }
    try {
      const binStr = atob(lineas.slice(2).join(""));
      const keyBytes = new TextEncoder().encode(entrada.frase_paso);
      const decBytes = new Uint8Array(binStr.length);
      for (let i = 0; i < binStr.length; i++) {
        decBytes[i] = binStr.charCodeAt(i) ^ keyBytes[i % keyBytes.length];
      }
      textoPlano = new TextDecoder().decode(decBytes);
    } catch {
      throw new Error("Frase de paso incorrecta o archivo dañado.");
    }
  }

  if (!textoPlano.startsWith("CUENTAS_CLARAS_LIBRO v1")) {
    const matchVer = textoPlano.match(/^CUENTAS_CLARAS_LIBRO (v\d+)/);
    if (matchVer) {
      throw new Error(`La versión de formato ${matchVer[1]} no está soportada.`);
    }
    throw new Error("Frase de paso incorrecta o archivo dañado.");
  }

  const sepIdx = textoPlano.indexOf("\n---\n");
  const cuerpo = sepIdx >= 0 ? textoPlano.slice(sepIdx + 5) : "";
  const lineasEventos = cuerpo
    .split(/\r?\n/)
    .filter((l) => l.trim().length > 0);

  const eventosParseados: any[] = [];
  for (const linea of lineasEventos) {
    try {
      const ev = JSON.parse(linea);
      if (!ev?.id || !ev?.tipo) throw new Error("Evento incompleto");
      eventosParseados.push(ev);
    } catch {
      throw new Error("El archivo de copia de seguridad está mal formado.");
    }
  }

  const rawLibro = typeof window !== "undefined" ? localStorage.getItem(STORAGE_KEY_LIBRO) : null;
  const libro = rawLibro
    ? JSON.parse(rawLibro)
    : { cuentas: [], saldosIniciales: {}, movimientosPorCuenta: {}, categoriasGasto: [], categoriasIngreso: [] };

  let nuevos = 0;
  let existentes = 0;
  const rechazados: EventoRechazadoImportacion[] = [];

  for (const ev of eventosParseados) {
    if (ev.tipo === "CuentaCreada") {
      const yaExiste = (libro.cuentas || []).some((c: any) => c.id === ev.datos?.id_cuenta);
      if (yaExiste) {
        existentes++;
      } else {
        libro.cuentas = [
          ...(libro.cuentas || []),
          {
            id: ev.datos.id_cuenta,
            nombre: ev.datos.nombre,
            tipo: ev.datos.tipo,
            moneda: ev.datos.moneda,
            saldo_formateado: `${ev.datos.moneda} ${Number(ev.datos.saldo_inicial || 0).toFixed(2)}`,
          },
        ];
        libro.saldosIniciales = {
          ...(libro.saldosIniciales || {}),
          [ev.datos.id_cuenta]: Number(ev.datos.saldo_inicial || 0),
        };
        nuevos++;
      }
    } else if (ev.tipo === "MovimientoRegistrado" || ev.tipo === "GastoDeGrupoAplicado") {
      const cid = ev.datos?.id_cuenta;
      const lista = libro.movimientosPorCuenta?.[cid] || [];
      const previo = lista.find((m: any) => m.id === ev.datos?.id);
      if (previo) {
        if (JSON.stringify(previo) !== JSON.stringify(ev.datos)) {
          rechazados.push({
            id_evento: ev.id,
            motivo: "Evento repetido con contenido distinto al registrado localmente.",
          });
        } else {
          existentes++;
        }
      } else {
        libro.movimientosPorCuenta = {
          ...(libro.movimientosPorCuenta || {}),
          [cid]: [ev.datos, ...lista],
        };
        nuevos++;
      }
    }
  }

  if (typeof window !== "undefined") {
    localStorage.setItem(STORAGE_KEY_LIBRO, JSON.stringify(libro));
  }

  return {
    eventos_nuevos: nuevos,
    eventos_existentes: existentes,
    eventos_rechazados: rechazados,
    archivo: entrada.nombre_archivo || entrada.ruta_origen,
  };
}

export async function obtenerConfiguracionSync(): Promise<ConfiguracionSyncDispositivos | null> {
  if (enTauri()) {
    try {
      const estado = await invoke<EstadoSincronizacionPersonalTauri>(
        "obtener_configuracion_sincronizacion"
      );
      if (!estado.destino) {
        return {
          configurado: false,
          dispositivo_id: estado.dispositivo_id,
          tipo: "git",
          ruta_o_url: "",
          fecha_ultima_sincronizacion: estado.fecha_ultima_sincronizacion,
        };
      }
      return {
        configurado: true,
        dispositivo_id: estado.dispositivo_id,
        tipo: estado.destino.tipo,
        ruta_o_url: estado.destino.ubicacion,
        fecha_ultima_sincronizacion: estado.fecha_ultima_sincronizacion,
      };
    } catch {
      return null;
    }
  }

  if (typeof window === "undefined") return null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY_SYNC_CFG);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

export async function guardarConfiguracionSync(
  entrada: EntradaConfigurarSync
): Promise<ConfiguracionSyncDispositivos> {
  if (!entrada.ruta_o_url.trim()) {
    throw new Error(
      entrada.tipo === "git"
        ? "Ingresa la URL del repositorio Git privado."
        : "Ingresa la ruta de la carpeta compartida."
    );
  }
  if (!entrada.frase_paso || entrada.frase_paso.length < 12) {
    throw new Error("La frase de paso de sincronización debe tener al menos 12 caracteres.");
  }

  if (enTauri()) {
    try {
      await invoke("configurar_destino_sincronizacion", {
        tipo: entrada.tipo,
        ubicacion: entrada.ruta_o_url.trim(),
      });
      await invoke("guardar_frase_paso_sincronizacion", {
        frasePaso: entrada.frase_paso,
      });
      if (
        entrada.tipo === "git" &&
        entrada.usuario_git?.trim() &&
        entrada.token_o_clave_git?.trim()
      ) {
        await invoke("actualizar_credenciales_git", {
          usuario: entrada.usuario_git.trim(),
          token: entrada.token_o_clave_git.trim(),
        });
      }
      const actualizado = await obtenerConfiguracionSync();
      if (actualizado) return actualizado;
    } catch (e) {
      throw normalizarErrorE8(e, "No se pudo guardar la configuración del destino.");
    }
  }

  const cfg: ConfiguracionSyncDispositivos = {
    configurado: true,
    tipo: entrada.tipo,
    ruta_o_url: entrada.ruta_o_url.trim(),
    usuario_git: entrada.usuario_git?.trim() || undefined,
    fecha_ultima_sincronizacion: null,
  };
  if (typeof window !== "undefined") {
    localStorage.setItem(STORAGE_KEY_SYNC_CFG, JSON.stringify(cfg));
  }
  return cfg;
}

export async function actualizarCredencialesGit(
  entrada: EntradaActualizarCredencialesGit
): Promise<void> {
  if (!entrada.usuario_git.trim() || !entrada.token_o_clave_git.trim()) {
    throw new Error("Ingresa el usuario y el token o contraseña de Git.");
  }
  if (enTauri()) {
    try {
      await invoke("actualizar_credenciales_git", {
        usuario: entrada.usuario_git.trim(),
        token: entrada.token_o_clave_git.trim(),
      });
      return;
    } catch (e) {
      throw normalizarErrorE8(e, "No se pudieron actualizar las credenciales de Git.");
    }
  }
  const actual = await obtenerConfiguracionSync();
  if (actual && typeof window !== "undefined") {
    localStorage.setItem(
      STORAGE_KEY_SYNC_CFG,
      JSON.stringify({ ...actual, usuario_git: entrada.usuario_git.trim() })
    );
  }
}

export async function sincronizarDispositivosAhora(): Promise<InformeSincronizacionDispositivos> {
  if (enTauri()) {
    try {
      return await invoke<InformeSincronizacionDispositivos>("sincronizar_libro_personal");
    } catch (e) {
      throw normalizarErrorE8(
        e,
        "No se pudo llegar al repositorio o a la carpeta; lo pendiente se conserva para el siguiente intento."
      );
    }
  }

  const cfg = await obtenerConfiguracionSync();
  if (!cfg || !cfg.configurado) {
    throw new Error("Configura primero un destino de sincronización.");
  }

  const ahora = new Date().toISOString();
  if (typeof window !== "undefined") {
    localStorage.setItem(
      STORAGE_KEY_SYNC_CFG,
      JSON.stringify({ ...cfg, fecha_ultima_sincronizacion: ahora })
    );
  }

  return {
    enviados: 0,
    recibidos: 0,
    rechazados: [],
    fecha_ultima_sincronizacion: ahora,
  };
}
