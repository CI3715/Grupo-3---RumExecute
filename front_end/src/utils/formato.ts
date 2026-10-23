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

export const CATEGORIAS_GASTO: string[] = [
  "Alimentación",
  "Transporte",
  "Servicios",
  "Salud",
  "Educación",
  "Entretenimiento",
  "Otros gastos",
];

export const CATEGORIAS_INGRESO: string[] = [
  "Salario",
  "Ventas",
  "Regalos",
  "Otros ingresos",
];

export const ICONOS_CATEGORIAS: Record<string, string> = {
  Alimentación: "🛒",
  Transporte: "🚗",
  Servicios: "💡",
  Salud: "💊",
  Educación: "📚",
  Entretenimiento: "🎬",
  "Otros gastos": "📦",
  Salario: "💼",
  Ventas: "📈",
  Regalos: "🎁",
  "Otros ingresos": "💵",
};

export function formatearMonto(cantidad: string | number, moneda: string): string {
  const simbolo = SIMBOLOS_MONEDA[moneda] || moneda;
  const num = Number(String(cantidad ?? "0").trim().replace(",", "."));
  if (isNaN(num)) return `${simbolo} 0,00`;

  const [enteroRaw, decimal = "00"] = Math.abs(num).toFixed(2).split(".");
  const entero = enteroRaw.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
  return num < 0 ? `- ${simbolo} ${entero},${decimal}` : `${simbolo} ${entero},${decimal}`;
}

export function formatearMontoConSigno(
  cantidad: string | number,
  moneda: string,
  tipo: "gasto" | "ingreso" | "ajuste" | string
): string {
  const simbolo = SIMBOLOS_MONEDA[moneda] || moneda;
  const num = Number(String(cantidad ?? "0").trim().replace(",", "."));
  if (isNaN(num)) return `${simbolo} 0,00`;

  const [enteroRaw, decimal = "00"] = Math.abs(num).toFixed(2).split(".");
  const entero = enteroRaw.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
  const signo = tipo === "gasto" ? "-" : tipo === "ingreso" ? "+" : num >= 0 ? "+" : "-";
  return `${signo} ${simbolo} ${entero},${decimal}`;
}

export function normalizarMontoInput(valor: string): string {
  if (!valor) return "";
  let limpio = valor.replace(/[^0-9,\.]/g, "").replace(/\./g, ",");
  if (limpio.startsWith(",")) limpio = "0" + limpio;

  const partes = limpio.split(",");
  if (partes.length > 2) limpio = partes[0] + "," + partes.slice(1).join("");

  const [enteros, decimales] = limpio.split(",");
  const enterosNorm =
    enteros.length > 1 && enteros.startsWith("0") ? enteros.replace(/^0+/, "") || "0" : enteros;

  return decimales !== undefined ? `${enterosNorm},${decimales.slice(0, 2)}` : enterosNorm;
}

export function completarDecimalesOnBlur(valor: string): string {
  const normal = normalizarMontoInput(valor);
  if (!normal || normal === "0" || normal === "0,") return "0,00";
  if (!normal.includes(",")) return `${normal},00`;

  const [enteros, decimales] = normal.split(",");
  return `${enteros || "0"},${(decimales + "00").slice(0, 2)}`;
}

export function calcularTotalesCuenta(
  saldoInicial: number,
  movs: Array<{ tipo: string; monto_num: number }> = []
) {
  let totalIngresos = 0;
  let totalGastos = 0;
  let totalAjustes = 0;

  for (const m of movs) {
    if (m.tipo === "ingreso") totalIngresos += m.monto_num;
    else if (m.tipo === "gasto") totalGastos += m.monto_num;
    else if (m.tipo === "ajuste") totalAjustes += m.monto_num;
  }

  return {
    totalIngresos,
    totalGastos,
    totalAjustes,
    saldoActual: saldoInicial + totalIngresos - totalGastos + totalAjustes,
  };
}
