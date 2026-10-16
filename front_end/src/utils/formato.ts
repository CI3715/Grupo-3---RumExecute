export const SIMBOLOS_MONEDA: Record<string, string> = {
  VES: "Bs.",
  USD: "$",
  EUR: "€",
  USDT: "USDT",
};

export const NOMBRES_MONEDA: Record<string, string> = {
  VES: "Bolívar",
  USD: "Dólar",
  EUR: "Euro",
  USDT: "Tether",
};

/**
 * Formatea un valor monetario (string o number) al estándar contable:
 * separador de miles con punto (.) y decimales con coma (,).
 * Ejemplo: 3600.5 en VES -> "Bs. 3.600,50"
 */
export function formatearMonto(cantidad: string | number, moneda: string): string {
  const simbolo = SIMBOLOS_MONEDA[moneda] || moneda;
  const str = String(cantidad ?? "0").trim().replace(",", ".");
  const num = Number(str);

  if (isNaN(num)) {
    return `${simbolo} 0,00`;
  }

  // Dividir en parte entera y parte decimal
  const partes = num.toFixed(2).split(".");
  const entero = partes[0].replace(/\B(?=(\d{3})+(?!\d))/g, ".");
  const decimal = partes[1] || "00";

  return `${simbolo} ${entero},${decimal}`;
}
