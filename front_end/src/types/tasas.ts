export interface MonedaItem {
  codigo: string;
  nombre: string;
  simbolo: string;
  decimales: number;
  es_fabrica?: boolean;
}

export interface TasaCambioItem {
  id: string;
  origen: string;
  destino: string;
  valor: number;
  fecha_vigencia: string; // YYYY-MM-DD
  fuente: string;
  momento_registro?: string;
  es_inversa?: boolean;
  eliminada?: boolean;
}

export interface EntradaRegistroTasa {
  origen: string;
  destino: string;
  valor: string | number;
  fecha_vigencia: string;
  fuente?: string;
}

export interface EntradaCrearMoneda {
  codigo: string;
  nombre: string;
  simbolo: string;
  decimales: number;
}

export interface CuentaPatrimonioItem {
  id: string;
  nombre: string;
  tipo: string;
  tipo_etiqueta?: string;
  moneda_original: string;
  simbolo_original: string;
  saldo_original: number;
  saldo_original_formateado: string;
  tasa_aplicada: number | null;
  tasa_texto?: string;
  saldo_convertido: number | null;
  saldo_convertido_formateado: string | null;
  fecha_tasa_vigente?: string | null;
  es_tasa_inversa?: boolean;
}

export interface TasaFaltanteInfo {
  moneda_origen: string;
  moneda_destino: string;
  fecha_requerida: string;
  mensaje: string;
}

export interface SubtotalPatrimonio {
  moneda: string;
  simbolo: string;
  nombre: string;
  subtotal: number;
  subtotal_formateado: string;
  cantidad_cuentas: number;
  tasa_vigente: number | null;
  subtotal_convertido: number | null;
  subtotal_convertido_formateado: string | null;
}

export interface TasaUsadaItem {
  moneda_origen: string;
  moneda_destino: string;
  valor: string;
  valor_registrado: string;
  fecha_vigencia: string;
  fuente: string;
  inversa: boolean;
}

export interface ResumenPatrimonioData {
  fecha_consulta: string;
  moneda_base: string;
  simbolo_base: string;
  decimales_base: number;
  cuentas: CuentaPatrimonioItem[];
  subtotales: SubtotalPatrimonio[];
  total_consolidado: number | null;
  total_consolidado_formateado: string | null;
  tasa_faltante: TasaFaltanteInfo | null;
  tasas_usadas?: TasaUsadaItem[];
}
