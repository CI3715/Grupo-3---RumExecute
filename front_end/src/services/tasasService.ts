import { invoke } from "@tauri-apps/api/core";
import {
  MonedaItem,
  TasaCambioItem,
  EntradaRegistroTasa,
  EntradaCrearMoneda,
  ResumenPatrimonioData,
  CuentaPatrimonioItem,
  SubtotalPatrimonio,
  TasaFaltanteInfo,
} from "../types/tasas";
import { formatearMonto } from "../utils/formato";

const enTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// Monedas de fábrica inmutables (RF-CAM-01)
export const MONEDAS_FABRICA: MonedaItem[] = [
  { codigo: "VES", nombre: "Bolívar", simbolo: "Bs.", decimales: 2, es_fabrica: true },
  { codigo: "USD", nombre: "Dólar estadounidense", simbolo: "$", decimales: 2, es_fabrica: true },
  { codigo: "EUR", nombre: "Euro", simbolo: "€", decimales: 2, es_fabrica: true },
  { codigo: "USDT", nombre: "Tether (dólar digital)", simbolo: "USDT", decimales: 2, es_fabrica: true },
];

const STORAGE_KEY_MONEDAS = "cuentas_claras_monedas_propias";
const STORAGE_KEY_TASAS = "cuentas_claras_tasas_cambio";
const STORAGE_KEY_MONEDA_BASE = "cuentas_claras_moneda_base";

function obtenerMonedasLocales(): MonedaItem[] {
  if (typeof window === "undefined") return MONEDAS_FABRICA;
  try {
    const raw = localStorage.getItem(STORAGE_KEY_MONEDAS);
    if (!raw) return MONEDAS_FABRICA;
    const personalizadas: MonedaItem[] = JSON.parse(raw);
    return [...MONEDAS_FABRICA, ...personalizadas.filter(p => !MONEDAS_FABRICA.some(f => f.codigo === p.codigo))];
  } catch {
    return MONEDAS_FABRICA;
  }
}

function guardarMonedaLocal(nueva: MonedaItem) {
  if (typeof window === "undefined") return;
  const personalizadas = obtenerMonedasLocales().filter(m => !m.es_fabrica);
  personalizadas.push(nueva);
  localStorage.setItem(STORAGE_KEY_MONEDAS, JSON.stringify(personalizadas));
}

function obtenerTasasLocales(): TasaCambioItem[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY_TASAS);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function guardarTasasLocales(tasas: TasaCambioItem[]) {
  if (typeof window === "undefined") return;
  localStorage.setItem(STORAGE_KEY_TASAS, JSON.stringify(tasas));
}

export function obtenerFechaHoyISO(): string {
  const d = new Date();
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export async function listarMonedas(): Promise<MonedaItem[]> {
  if (enTauri()) {
    try {
      const res = await invoke<any>("listar_monedas");
      if (Array.isArray(res) && res.length > 0) {
        return res.map((m: any) => ({
          codigo: m.codigo,
          nombre: m.nombre,
          simbolo: m.simbolo,
          decimales: Number(m.decimales ?? 2),
          es_fabrica: ["VES", "USD", "EUR", "USDT"].includes(m.codigo),
        }));
      }
    } catch {
      // Fallback
    }
  }
  return obtenerMonedasLocales();
}

export async function obtenerMonedaBase(): Promise<string> {
  if (enTauri()) {
    try {
      const base = await invoke<string>("obtener_moneda_base");
      if (base) return base;
    } catch {}
  }
  if (typeof window !== "undefined") {
    return localStorage.getItem(STORAGE_KEY_MONEDA_BASE) || "USD";
  }
  return "USD";
}

export async function cambiarMonedaBase(nuevoCodigo: string): Promise<string> {
  const codigoUpper = nuevoCodigo.toUpperCase().trim();
  if (enTauri()) {
    try {
      await invoke("cambiar_moneda_base", { nuevoCodigo: codigoUpper })
        .catch(() => invoke("cambiar_moneda_base", { nuevo_codigo: codigoUpper }));
    } catch {}
  }
  if (typeof window !== "undefined") {
    localStorage.setItem(STORAGE_KEY_MONEDA_BASE, codigoUpper);
  }
  return codigoUpper;
}

export async function crearMonedaPropia(entrada: EntradaCrearMoneda): Promise<MonedaItem> {
  const codigo = entrada.codigo.trim().toUpperCase();
  if (!/^[A-Z]{3,5}$/.test(codigo)) {
    throw new Error("El código debe tener entre 3 y 5 letras mayúsculas (A-Z).");
  }
  if (["VES", "USD", "EUR", "USDT"].includes(codigo)) {
    throw new Error(`La moneda ${codigo} es una moneda de fábrica y no se puede duplicar.`);
  }
  if (!entrada.nombre.trim()) {
    throw new Error("El nombre de la moneda es obligatorio.");
  }
  if (!entrada.simbolo.trim()) {
    throw new Error("El símbolo de la moneda es obligatorio.");
  }
  const decimales = Number(entrada.decimales);
  if (isNaN(decimales) || decimales < 0 || decimales > 8) {
    throw new Error("Los decimales permitidos deben ser un número entre 0 y 8.");
  }

  const todas = await listarMonedas();
  if (todas.some(m => m.codigo === codigo)) {
    throw new Error(`Ya existe una moneda registrada con el código ${codigo}.`);
  }

  const nuevaMoneda: MonedaItem = {
    codigo,
    nombre: entrada.nombre.trim(),
    simbolo: entrada.simbolo.trim(),
    decimales,
    es_fabrica: false,
  };

  if (enTauri()) {
    try {
      await invoke("crear_moneda", { entrada: nuevaMoneda });
    } catch {
      guardarMonedaLocal(nuevaMoneda);
    }
  } else {
    guardarMonedaLocal(nuevaMoneda);
  }

  return nuevaMoneda;
}

export async function registrarTasa(entrada: EntradaRegistroTasa): Promise<TasaCambioItem> {
  const origen = entrada.origen.trim().toUpperCase();
  const destino = entrada.destino.trim().toUpperCase();

  if (origen === destino) {
    throw new Error("Las monedas origen y destino deben ser distintas.");
  }

  const valorNum = typeof entrada.valor === "number" ? entrada.valor : parseFloat(entrada.valor.replace(",", "."));
  if (isNaN(valorNum) || valorNum <= 0) {
    throw new Error("El valor de la tasa debe ser un número estrictamente mayor a 0.");
  }

  const hoy = obtenerFechaHoyISO();
  if (entrada.fecha_vigencia > hoy) {
    throw new Error(`La fecha de vigencia (${entrada.fecha_vigencia}) no puede ser futura.`);
  }

  const nuevaTasa: TasaCambioItem = {
    id: typeof crypto !== "undefined" && crypto.randomUUID ? crypto.randomUUID() : `tasa-${Date.now()}`,
    origen,
    destino,
    valor: valorNum,
    fecha_vigencia: entrada.fecha_vigencia,
    fuente: entrada.fuente?.trim() || "Manual",
    momento_registro: new Date().toISOString(),
  };

  if (enTauri()) {
    try {
      await invoke("registrar_tasa", { entrada: nuevaTasa })
        .catch(() => invoke("registrar_tasa", {
          origen,
          destino,
          valor: valorNum.toString(),
          fecha_vigencia: entrada.fecha_vigencia,
          fuente: nuevaTasa.fuente,
        }));
    } catch {
      const lista = obtenerTasasLocales();
      lista.push(nuevaTasa);
      guardarTasasLocales(lista);
    }
  } else {
    const lista = obtenerTasasLocales();
    lista.push(nuevaTasa);
    guardarTasasLocales(lista);
  }

  return nuevaTasa;
}

export async function eliminarTasa(idTasa: string): Promise<void> {
  if (enTauri()) {
    try {
      await invoke("eliminar_tasa", { idTasa })
        .catch(() => invoke("eliminar_tasa", { id_tasa: idTasa }));
    } catch {
      const lista = obtenerTasasLocales().filter(t => t.id !== idTasa);
      guardarTasasLocales(lista);
    }
  } else {
    const lista = obtenerTasasLocales().filter(t => t.id !== idTasa);
    guardarTasasLocales(lista);
  }
}

export async function listarHistorialTasas(origen?: string, destino?: string): Promise<TasaCambioItem[]> {
  let tasas: TasaCambioItem[] = [];
  if (enTauri()) {
    try {
      const res = await invoke<any>("listar_historial_tasas", { origen, destino });
      if (Array.isArray(res)) {
        tasas = res.map((t: any) => ({
          id: t.id,
          origen: t.origen,
          destino: t.destino,
          valor: Number(t.valor),
          fecha_vigencia: t.fecha_vigencia,
          fuente: t.fuente || "Manual",
          momento_registro: t.momento_registro,
        }));
      }
    } catch {
      tasas = obtenerTasasLocales();
    }
  } else {
    tasas = obtenerTasasLocales();
  }

  if (origen && destino) {
    tasas = tasas.filter(t => (t.origen === origen && t.destino === destino) || (t.origen === destino && t.destino === origen));
  } else if (origen) {
    tasas = tasas.filter(t => t.origen === origen || t.destino === origen);
  }

  return tasas.sort((a, b) => {
    if (a.fecha_vigencia !== b.fecha_vigencia) {
      return b.fecha_vigencia.localeCompare(a.fecha_vigencia);
    }
    return (b.momento_registro || "").localeCompare(a.momento_registro || "");
  });
}

export function resolverTasaVigente(
  tasas: TasaCambioItem[],
  origen: string,
  destino: string,
  fechaConsulta: string
): { tasa: number; fecha: string; esInversa: boolean } | null {
  if (origen === destino) {
    return { tasa: 1.0, fecha: fechaConsulta, esInversa: false };
  }

  const directas = tasas.filter(
    t => t.origen === origen && t.destino === destino && t.fecha_vigencia <= fechaConsulta
  );

  if (directas.length > 0) {
    directas.sort((a, b) => b.fecha_vigencia.localeCompare(a.fecha_vigencia));
    return {
      tasa: directas[0].valor,
      fecha: directas[0].fecha_vigencia,
      esInversa: false,
    };
  }

  const inversas = tasas.filter(
    t => t.origen === destino && t.destino === origen && t.fecha_vigencia <= fechaConsulta
  );

  if (inversas.length > 0) {
    inversas.sort((a, b) => b.fecha_vigencia.localeCompare(a.fecha_vigencia));
    const valorOriginal = inversas[0].valor;
    if (valorOriginal > 0) {
      return {
        tasa: 1 / valorOriginal,
        fecha: inversas[0].fecha_vigencia,
        esInversa: true,
      };
    }
  }

  return null;
}

function adaptarResumenPatrimonioRust(res: any, fechaConsulta: string): ResumenPatrimonioData {
  const monedaBase = res.moneda_base || res.monedaBase || "USD";
  const totalRaw = res.total_consolidado ?? res.totalConsolidado;
  let totalNum: number | null = null;
  if (totalRaw !== undefined && totalRaw !== null) {
    totalNum = typeof totalRaw === "object" ? Number(totalRaw.cantidad ?? 0) : Number(totalRaw);
  }

  return {
    fecha_consulta: res.fecha_consulta || res.fechaConsulta || fechaConsulta,
    moneda_base: monedaBase,
    simbolo_base: res.simbolo_base || res.simboloBase || (monedaBase === "VES" ? "Bs." : monedaBase === "EUR" ? "€" : "$"),
    decimales_base: Number(res.decimales_base ?? res.decimalesBase ?? 2),
    cuentas: (res.cuentas || []).map((c: any) => ({
      id: c.id,
      nombre: c.nombre,
      tipo: c.tipo,
      moneda_original: c.moneda_original || c.monedaOriginal || c.moneda,
      simbolo_original: c.simbolo_original || c.simboloOriginal || "",
      saldo_original: Number(c.saldo_original ?? c.saldoOriginal ?? c.saldo ?? 0),
      saldo_original_formateado: c.saldo_original_formateado || c.saldoOriginalFormateado || formatearMonto(Number(c.saldo_original ?? c.saldo ?? 0), c.moneda_original || c.moneda),
      tasa_aplicada: c.tasa_aplicada ?? c.tasaAplicada ?? null,
      tasa_texto: c.tasa_texto || c.tasaTexto || (c.tasa_aplicada ? String(c.tasa_aplicada) : "Sin tasa"),
      saldo_convertido: c.saldo_convertido !== undefined && c.saldo_convertido !== null ? Number(c.saldo_convertido) : null,
      saldo_convertido_formateado: c.saldo_convertido_formateado || c.saldoConvertidoFormateado || null,
      fecha_tasa_vigente: c.fecha_tasa_vigente || c.fechaTasaVigente || null,
      es_tasa_inversa: Boolean(c.es_tasa_inversa ?? c.esTasaInversa),
    })),
    subtotales: (res.subtotales || []).map((s: any) => ({
      moneda: s.moneda,
      simbolo: s.simbolo || s.moneda,
      nombre: s.nombre || s.moneda,
      subtotal: Number(s.subtotal ?? 0),
      subtotal_formateado: s.subtotal_formateado || s.subtotalFormateado || formatearMonto(Number(s.subtotal ?? 0), s.moneda),
      cantidad_cuentas: Number(s.cantidad_cuentas ?? s.cantidadCuentas ?? 1),
      tasa_vigente: s.tasa_vigente ?? s.tasaVigente ?? null,
      subtotal_convertido: s.subtotal_convertido !== undefined && s.subtotal_convertido !== null ? Number(s.subtotal_convertido) : null,
      subtotal_convertido_formateado: s.subtotal_convertido_formateado || s.subtotalConvertidoFormateado || null,
    })),
    total_consolidado: totalNum,
    total_consolidado_formateado: res.total_consolidado_formateado || res.totalConsolidadoFormateado || (totalNum !== null ? formatearMonto(totalNum, monedaBase) : null),
    tasa_faltante: res.tasa_faltante || res.tasaFaltante || null,
  };
}

export async function consultarPatrimonio(
  fechaConsulta: string = obtenerFechaHoyISO(),
  cuentasEntrada?: Array<{ id: string; nombre: string; tipo: string; moneda: string; saldo: number }>
): Promise<ResumenPatrimonioData> {
  // 1. Enfoque Primario Arquitectónico: La Interfaz de Usuario delega al Backend (Tauri / Rust)
  // El cálculo de patrimonio, invariantes y tasas vigentes le corresponde al Dominio y Aplicación en Rust.
  if (enTauri()) {
    try {
      const resRust = await invoke<any>("consultar_patrimonio", { fechaConsulta })
        .catch(() => invoke<any>("consultar_patrimonio", { fecha_consulta: fechaConsulta }));
      if (resRust && (resRust.cuentas || resRust.subtotales)) {
        return adaptarResumenPatrimonioRust(resRust, fechaConsulta);
      }
    } catch {
      // Si el comando Tauri aún no está compilado en la rama local (compañeros trabajando en paralelo),
      // se utiliza el adaptador visual de previsualización sin bloquear la interfaz.
    }
  }

  // 2. Adaptador de desarrollo visual (Mock frontend solo para previsualización en navegador web)
  const monedaBase = await obtenerMonedaBase();
  const monedas = await listarMonedas();
  const tasas = await listarHistorialTasas();

  const monedaBaseObj = monedas.find(m => m.codigo === monedaBase) || {
    codigo: monedaBase,
    nombre: monedaBase,
    simbolo: monedaBase === "USD" ? "$" : monedaBase === "VES" ? "Bs." : monedaBase === "EUR" ? "€" : monedaBase,
    decimales: 2,
  };

  const listaCuentas = cuentasEntrada || [];

  const subtotalesMap: Record<string, { total: number; cantidad: number }> = {};
  listaCuentas.forEach(c => {
    if (!subtotalesMap[c.moneda]) {
      subtotalesMap[c.moneda] = { total: 0, cantidad: 0 };
    }
    subtotalesMap[c.moneda].total += c.saldo;
    subtotalesMap[c.moneda].cantidad += 1;
  });

  let tasaFaltante: TasaFaltanteInfo | null = null;
  const cuentasPatrimonio: CuentaPatrimonioItem[] = [];

  for (const c of listaCuentas) {
    const infoMoneda = monedas.find(m => m.codigo === c.moneda) || {
      codigo: c.moneda,
      simbolo: c.moneda,
      decimales: 2,
    };

    if (c.moneda === monedaBase) {
      cuentasPatrimonio.push({
        id: c.id,
        nombre: c.nombre,
        tipo: c.tipo,
        moneda_original: c.moneda,
        simbolo_original: infoMoneda.simbolo,
        saldo_original: c.saldo,
        saldo_original_formateado: formatearMonto(c.saldo, c.moneda),
        tasa_aplicada: 1.0,
        tasa_texto: "1.00 (Moneda Base)",
        saldo_convertido: c.saldo,
        saldo_convertido_formateado: formatearMonto(c.saldo, monedaBase),
        fecha_tasa_vigente: fechaConsulta,
        es_tasa_inversa: false,
      });
    } else {
      const resTasa = resolverTasaVigente(tasas, c.moneda, monedaBase, fechaConsulta);

      if (!resTasa && Math.abs(c.saldo) > 0.00000001 && !tasaFaltante) {
        tasaFaltante = {
          moneda_origen: c.moneda,
          moneda_destino: monedaBase,
          fecha_requerida: fechaConsulta,
          mensaje: `No se puede calcular el patrimonio consolidado al ${fechaConsulta} porque falta la tasa de cambio para el par ${c.moneda} → ${monedaBase} en esa fecha o anterior.`,
        };
      }

      if (resTasa) {
        const saldoConv = c.saldo * resTasa.tasa;
        const tasaStr = resTasa.esInversa
          ? `≈ ${resTasa.tasa.toFixed(4)} (inversa de ${(1 / resTasa.tasa).toFixed(2)})`
          : resTasa.tasa.toFixed(4);

        cuentasPatrimonio.push({
          id: c.id,
          nombre: c.nombre,
          tipo: c.tipo,
          moneda_original: c.moneda,
          simbolo_original: infoMoneda.simbolo,
          saldo_original: c.saldo,
          saldo_original_formateado: formatearMonto(c.saldo, c.moneda),
          tasa_aplicada: resTasa.tasa,
          tasa_texto: tasaStr,
          saldo_convertido: saldoConv,
          saldo_convertido_formateado: formatearMonto(saldoConv, monedaBase),
          fecha_tasa_vigente: resTasa.fecha,
          es_tasa_inversa: resTasa.esInversa,
        });
      } else if (Math.abs(c.saldo) <= 0.00000001) {
        cuentasPatrimonio.push({
          id: c.id,
          nombre: c.nombre,
          tipo: c.tipo,
          moneda_original: c.moneda,
          simbolo_original: infoMoneda.simbolo,
          saldo_original: 0,
          saldo_original_formateado: formatearMonto(0, c.moneda),
          tasa_aplicada: null,
          tasa_texto: "Sin tasa (Saldo cero)",
          saldo_convertido: 0,
          saldo_convertido_formateado: formatearMonto(0, monedaBase),
          fecha_tasa_vigente: null,
          es_tasa_inversa: false,
        });
      } else {
        cuentasPatrimonio.push({
          id: c.id,
          nombre: c.nombre,
          tipo: c.tipo,
          moneda_original: c.moneda,
          simbolo_original: infoMoneda.simbolo,
          saldo_original: c.saldo,
          saldo_original_formateado: formatearMonto(c.saldo, c.moneda),
          tasa_aplicada: null,
          tasa_texto: "Sin tasa vigente",
          saldo_convertido: null,
          saldo_convertido_formateado: null,
          fecha_tasa_vigente: null,
          es_tasa_inversa: false,
        });
      }
    }
  }

  const subtotales: SubtotalPatrimonio[] = Object.entries(subtotalesMap).map(([mon, item]) => {
    const mInfo = monedas.find(m => m.codigo === mon) || { nombre: mon, simbolo: mon };
    const resTasa = resolverTasaVigente(tasas, mon, monedaBase, fechaConsulta);
    const subConv = resTasa ? item.total * resTasa.tasa : (Math.abs(item.total) <= 0.00000001 ? 0 : null);

    return {
      moneda: mon,
      simbolo: mInfo.simbolo,
      nombre: mInfo.nombre,
      subtotal: item.total,
      subtotal_formateado: formatearMonto(item.total, mon),
      cantidad_cuentas: item.cantidad,
      tasa_vigente: resTasa ? resTasa.tasa : null,
      subtotal_convertido: subConv,
      subtotal_convertido_formateado: subConv !== null ? formatearMonto(subConv, monedaBase) : null,
    };
  });

  let totalConsolidado: number | null = null;
  let totalConsolidadoFormateado: string | null = null;

  if (!tasaFaltante && cuentasPatrimonio.every(c => c.saldo_convertido !== null)) {
    const suma = cuentasPatrimonio.reduce((acc, c) => acc + (c.saldo_convertido || 0), 0);
    const factor = Math.pow(10, monedaBaseObj.decimales);
    totalConsolidado = Math.round(suma * factor) / factor;
    totalConsolidadoFormateado = formatearMonto(totalConsolidado, monedaBase);
  }

  return {
    fecha_consulta: fechaConsulta,
    moneda_base: monedaBase,
    simbolo_base: monedaBaseObj.simbolo,
    decimales_base: monedaBaseObj.decimales,
    cuentas: cuentasPatrimonio,
    subtotales,
    total_consolidado: totalConsolidado,
    total_consolidado_formateado: totalConsolidadoFormateado,
    tasa_faltante: tasaFaltante,
  };
}
