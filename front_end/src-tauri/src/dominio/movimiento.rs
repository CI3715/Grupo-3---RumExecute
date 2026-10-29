use serde::{Deserialize, Serialize};

use super::categoria::{
    catalogo_categorias, validar_categoria_para_tipo_en, Categoria, TipoCategoria,
};
use super::cuenta::Cuenta;
use super::errores::ErrorDominio;
use super::moneda::CodigoMoneda;
use super::monto::Monto;

/// Tipo de movimiento en el libro contable (RF-LIB-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoMovimiento {
    Gasto,
    Ingreso,
    Ajuste,
}

impl TipoMovimiento {
    pub fn as_str(&self) -> &'static str {
        match self {
            TipoMovimiento::Gasto => "gasto",
            TipoMovimiento::Ingreso => "ingreso",
            TipoMovimiento::Ajuste => "ajuste",
        }
    }
}

/// Entidad Movimiento (RF-LIB-03, INV-04).
///
/// Invariante INV-04: Inmutabilidad estricta.
/// Un movimiento registrado nunca se modifica ni se elimina.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Movimiento {
    pub id: String,
    pub id_cuenta: String,
    pub tipo: TipoMovimiento,
    pub monto: Monto,
    pub categoria: Option<Categoria>,
    pub fecha: String,
    pub descripcion: String,
    /// Presente si es un Ajuste (INV-04), referencia al movimiento original.
    pub id_movimiento_referenciado: Option<String>,
}

impl Movimiento {
    /// Crea y valida un nuevo movimiento de tipo Gasto con catálogo oficial de fábrica.
    pub fn nuevo_gasto(
        id: impl Into<String>,
        cuenta: &Cuenta,
        monto: Monto,
        categoria_nombre_o_id: &str,
        fecha: &str,
        descripcion: impl Into<String>,
        momento_actual_utc: &str,
    ) -> Result<Self, ErrorDominio> {
        Self::nuevo_gasto_con_catalogo(
            id,
            cuenta,
            monto,
            categoria_nombre_o_id,
            fecha,
            descripcion,
            momento_actual_utc,
            &catalogo_categorias(),
        )
    }

    /// Crea y valida un nuevo movimiento de tipo Gasto verificando contra un catálogo dado.
    #[allow(clippy::too_many_arguments)]
    pub fn nuevo_gasto_con_catalogo(
        id: impl Into<String>,
        cuenta: &Cuenta,
        monto: Monto,
        categoria_nombre_o_id: &str,
        fecha: &str,
        descripcion: impl Into<String>,
        momento_actual_utc: &str,
        catalogo: &[Categoria],
    ) -> Result<Self, ErrorDominio> {
        validar_monto_movimiento(&monto, cuenta.moneda)?;
        validar_fecha_no_futura(fecha, momento_actual_utc)?;
        let categoria =
            validar_categoria_para_tipo_en(categoria_nombre_o_id, TipoCategoria::Gasto, catalogo)?;

        Ok(Self {
            id: id.into(),
            id_cuenta: cuenta.id.clone(),
            tipo: TipoMovimiento::Gasto,
            monto,
            categoria: Some(categoria),
            fecha: fecha.trim().to_string(),
            descripcion: descripcion.into().trim().to_string(),
            id_movimiento_referenciado: None,
        })
    }

    /// Crea y valida un nuevo movimiento de tipo Ingreso con catálogo oficial de fábrica.
    pub fn nuevo_ingreso(
        id: impl Into<String>,
        cuenta: &Cuenta,
        monto: Monto,
        categoria_nombre_o_id: &str,
        fecha: &str,
        descripcion: impl Into<String>,
        momento_actual_utc: &str,
    ) -> Result<Self, ErrorDominio> {
        Self::nuevo_ingreso_con_catalogo(
            id,
            cuenta,
            monto,
            categoria_nombre_o_id,
            fecha,
            descripcion,
            momento_actual_utc,
            &catalogo_categorias(),
        )
    }

    /// Crea y valida un nuevo movimiento de tipo Ingreso verificando contra un catálogo dado.
    #[allow(clippy::too_many_arguments)]
    pub fn nuevo_ingreso_con_catalogo(
        id: impl Into<String>,
        cuenta: &Cuenta,
        monto: Monto,
        categoria_nombre_o_id: &str,
        fecha: &str,
        descripcion: impl Into<String>,
        momento_actual_utc: &str,
        catalogo: &[Categoria],
    ) -> Result<Self, ErrorDominio> {
        validar_monto_movimiento(&monto, cuenta.moneda)?;
        validar_fecha_no_futura(fecha, momento_actual_utc)?;
        let categoria = validar_categoria_para_tipo_en(
            categoria_nombre_o_id,
            TipoCategoria::Ingreso,
            catalogo,
        )?;

        Ok(Self {
            id: id.into(),
            id_cuenta: cuenta.id.clone(),
            tipo: TipoMovimiento::Ingreso,
            monto,
            categoria: Some(categoria),
            fecha: fecha.trim().to_string(),
            descripcion: descripcion.into().trim().to_string(),
            id_movimiento_referenciado: None,
        })
    }

    /// Crea y valida un movimiento de Ajuste para corregir un movimiento erróneo (INV-04).
    ///
    /// El ajuste compensa matemáticamente la diferencia entre el valor original y el correcto.
    /// Referencia inmutablemente al movimiento original mediante `id_movimiento_referenciado`.
    pub fn nuevo_ajuste(
        id: impl Into<String>,
        cuenta: &Cuenta,
        monto_ajuste: Monto,
        id_movimiento_original: impl Into<String>,
        fecha: &str,
        descripcion: impl Into<String>,
        momento_actual_utc: &str,
    ) -> Result<Self, ErrorDominio> {
        let id_original = id_movimiento_original.into().trim().to_string();
        if id_original.is_empty() {
            return Err(ErrorDominio::new(
                "movimiento_referenciado",
                "Un movimiento de ajuste debe referenciar al identificador del movimiento original.",
            ));
        }

        if monto_ajuste.moneda != cuenta.moneda {
            return Err(ErrorDominio::new(
                "moneda",
                format!(
                    "La moneda del ajuste ({}) no coincide con la moneda de la cuenta ({}).",
                    monto_ajuste.moneda, cuenta.moneda
                ),
            ));
        }

        let decimales_admitidos = cuenta.moneda.decimales();
        if monto_ajuste.cantidad.normalize().scale() > decimales_admitidos {
            return Err(ErrorDominio::monto_invalido(format!(
                "El monto de ajuste {} tiene más de {} decimales permitidos.",
                monto_ajuste.cantidad, decimales_admitidos
            )));
        }

        validar_fecha_no_futura(fecha, momento_actual_utc)?;

        Ok(Self {
            id: id.into(),
            id_cuenta: cuenta.id.clone(),
            tipo: TipoMovimiento::Ajuste,
            monto: monto_ajuste,
            categoria: None,
            fecha: fecha.trim().to_string(),
            descripcion: descripcion.into().trim().to_string(),
            id_movimiento_referenciado: Some(id_original),
        })
    }

    /// Crea un movimiento de Ajuste a partir de un movimiento original y el nuevo monto corregido (INV-04).
    ///
    /// Calcula automáticamente la diferencia matemática compensatoria y referencia al movimiento original.
    pub fn crear_ajuste_por_correccion(
        id: impl Into<String>,
        cuenta: &Cuenta,
        movimiento_original: &Movimiento,
        monto_correcto: &Monto,
        fecha: &str,
        descripcion: impl Into<String>,
        momento_actual_utc: &str,
    ) -> Result<Self, ErrorDominio> {
        let monto_ajuste = calcular_diferencia_ajuste(movimiento_original, monto_correcto)?;
        Self::nuevo_ajuste(
            id,
            cuenta,
            monto_ajuste,
            &movimiento_original.id,
            fecha,
            descripcion,
            momento_actual_utc,
        )
    }
}

/// Valida que el monto del movimiento sea > 0, no exceda los decimales de la moneda
/// y pertenezca a la misma moneda de la cuenta.
pub fn validar_monto_movimiento(
    monto: &Monto,
    moneda_cuenta: CodigoMoneda,
) -> Result<(), ErrorDominio> {
    if monto.moneda != moneda_cuenta {
        return Err(ErrorDominio::new(
            "moneda",
            format!(
                "La moneda del monto ({}) no coincide con la moneda de la cuenta ({}).",
                monto.moneda, moneda_cuenta
            ),
        ));
    }

    if monto.cantidad <= rust_decimal::Decimal::ZERO {
        return Err(ErrorDominio::monto_invalido(
            "El monto del movimiento debe ser estrictamente mayor que cero.",
        ));
    }

    let decimales_admitidos = moneda_cuenta.decimales();
    if monto.cantidad.normalize().scale() > decimales_admitidos {
        return Err(ErrorDominio::monto_invalido(format!(
            "El monto {} tiene más de {} decimales, lo cual no está permitido para la moneda {}.",
            monto.cantidad, decimales_admitidos, moneda_cuenta
        )));
    }

    Ok(())
}

fn extraer_componentes_fecha(s: &str) -> Result<(u32, u32, u32, &str), ErrorDominio> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(ErrorDominio::fecha_invalida(
            "La fecha del movimiento no puede estar vacía.",
        ));
    }

    let ymd_part = trimmed
        .split(|c: char| c == 'T' || c.is_whitespace())
        .next()
        .unwrap_or(trimmed);

    if ymd_part.len() != 10 {
        return Err(ErrorDominio::fecha_invalida(
            "La fecha del movimiento debe tener el formato YYYY-MM-DD.",
        ));
    }

    let partes: Vec<&str> = ymd_part.split('-').collect();
    if partes.len() != 3 || partes[0].len() != 4 || partes[1].len() != 2 || partes[2].len() != 2 {
        return Err(ErrorDominio::fecha_invalida(
            "La fecha del movimiento debe tener el formato YYYY-MM-DD.",
        ));
    }

    let anio: u32 = partes[0]
        .parse()
        .map_err(|_| ErrorDominio::fecha_invalida("Año inválido en la fecha del movimiento."))?;
    let mes: u32 = partes[1]
        .parse()
        .map_err(|_| ErrorDominio::fecha_invalida("Mes inválido en la fecha del movimiento."))?;
    let dia: u32 = partes[2]
        .parse()
        .map_err(|_| ErrorDominio::fecha_invalida("Día inválido en la fecha del movimiento."))?;

    if !(1..=12).contains(&mes) {
        return Err(ErrorDominio::fecha_invalida(
            "El mes en la fecha debe estar entre 01 y 12.",
        ));
    }

    let dias_en_mes = match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let es_bisiesto =
                (anio.is_multiple_of(4) && !anio.is_multiple_of(100)) || anio.is_multiple_of(400);
            if es_bisiesto {
                29
            } else {
                28
            }
        }
        _ => unreachable!(),
    };

    if dia < 1 || dia > dias_en_mes {
        return Err(ErrorDominio::fecha_invalida(format!(
            "El día {:02} no es válido para el mes {:02} del año {:04}.",
            dia, mes, anio
        )));
    }

    Ok((anio, mes, dia, ymd_part))
}

/// Valida que una fecha dada (en formato YYYY-MM-DD o ISO-8601) no sea futura respecto al momento actual UTC.
///
/// Si la fecha es futura o tiene un formato no válido, retorna `ErrorDominio` con `campo: "fecha"`.
pub fn validar_fecha_no_futura(fecha: &str, momento_actual_utc: &str) -> Result<(), ErrorDominio> {
    let (f_anio, f_mes, f_dia, f_ymd) = extraer_componentes_fecha(fecha)?;
    let (act_anio, act_mes, act_dia, act_ymd) = extraer_componentes_fecha(momento_actual_utc)?;

    if (f_anio, f_mes, f_dia) > (act_anio, act_mes, act_dia) {
        return Err(ErrorDominio::fecha_invalida(format!(
            "La fecha '{}' es futura respecto a la fecha actual del sistema ('{}').",
            f_ymd, act_ymd
        )));
    }

    let f_trim = fecha.trim();
    let act_trim = momento_actual_utc.trim();
    if (f_anio, f_mes, f_dia) == (act_anio, act_mes, act_dia)
        && f_trim.contains('T')
        && act_trim.contains('T')
        && f_trim > act_trim
    {
        return Err(ErrorDominio::fecha_invalida(format!(
            "El momento '{}' es posterior al momento actual del sistema ('{}').",
            f_trim, act_trim
        )));
    }

    Ok(())
}

/// Calcula la compensación matemática necesaria para corregir un movimiento erróneo (INV-04).
///
/// Reglas matemáticas (INV-04, INV-05, Sección 2.2):
/// - Si el movimiento original es un GASTO:
///   Si se anotó por 500 y debía ser 50 (se restaron 450 de más), el ajuste debe sumar +450.
///   Fórmula: `monto_original - monto_correcto`.
/// - Si el movimiento original es un INGRESO:
///   Si se anotó por 100 y debía ser 150 (se sumaron 50 de menos), el ajuste debe sumar +50.
///   Fórmula: `monto_correcto - monto_original`.
///
/// Validaciones:
/// - No se puede ajustar un movimiento de tipo `Ajuste`.
/// - El monto correcto debe tener la misma moneda que el movimiento original.
/// - El monto correcto debe ser estrictamente mayor que cero (`monto > 0`).
/// - No puede tener más decimales que los que admite la moneda.
/// - El monto correcto no puede ser idéntico al monto original (no hay discrepancia que corregir).
pub fn calcular_diferencia_ajuste(
    movimiento_original: &Movimiento,
    monto_correcto: &Monto,
) -> Result<Monto, ErrorDominio> {
    if movimiento_original.tipo == TipoMovimiento::Ajuste {
        return Err(ErrorDominio::new(
            "tipo",
            "No se puede corregir un movimiento que ya es de tipo ajuste.",
        ));
    }

    if movimiento_original.monto.moneda != monto_correcto.moneda {
        return Err(ErrorDominio::new(
            "moneda",
            format!(
                "La moneda del monto correcto ({}) no coincide con la moneda del movimiento original ({}).",
                monto_correcto.moneda, movimiento_original.monto.moneda
            ),
        ));
    }

    if monto_correcto.cantidad <= rust_decimal::Decimal::ZERO {
        return Err(ErrorDominio::monto_invalido(
            "El nuevo monto corregido debe ser estrictamente mayor que cero.",
        ));
    }

    let decimales_admitidos = monto_correcto.moneda.decimales();
    if monto_correcto.cantidad.normalize().scale() > decimales_admitidos {
        return Err(ErrorDominio::monto_invalido(format!(
            "El nuevo monto {} tiene más de {} decimales permitidos para la moneda {}.",
            monto_correcto.cantidad, decimales_admitidos, monto_correcto.moneda
        )));
    }

    if movimiento_original.monto.cantidad == monto_correcto.cantidad {
        return Err(ErrorDominio::monto_invalido(
            "El nuevo monto es idéntico al monto original; no hay discrepancia que corregir.",
        ));
    }

    let diferencia = match movimiento_original.tipo {
        TipoMovimiento::Gasto => movimiento_original.monto.cantidad - monto_correcto.cantidad,
        TipoMovimiento::Ingreso => monto_correcto.cantidad - movimiento_original.monto.cantidad,
        TipoMovimiento::Ajuste => unreachable!(),
    };

    Ok(Monto::new(diferencia, movimiento_original.monto.moneda))
}

/// Aplica un movimiento a un saldo acumulado respetando la fórmula de INV-05:
/// Saldo = Saldo inicial + Ingresos - Gastos ± Ajustes
pub fn aplicar_movimiento_a_saldo(
    saldo_actual: &Monto,
    movimiento: &Movimiento,
) -> Result<Monto, ErrorDominio> {
    if saldo_actual.moneda != movimiento.monto.moneda {
        return Err(ErrorDominio::new(
            "moneda",
            format!(
                "Invariante INV-02 violado: No se puede aplicar un movimiento en {} a un saldo en {}.",
                movimiento.monto.moneda, saldo_actual.moneda
            ),
        ));
    }

    let nueva_cantidad = match movimiento.tipo {
        TipoMovimiento::Ingreso => saldo_actual.cantidad + movimiento.monto.cantidad,
        TipoMovimiento::Gasto => saldo_actual.cantidad - movimiento.monto.cantidad,
        TipoMovimiento::Ajuste => saldo_actual.cantidad + movimiento.monto.cantidad,
    };

    Ok(Monto::new(nueva_cantidad, saldo_actual.moneda))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::cuenta::TipoCuenta;
    use rust_decimal::Decimal;

    fn cuenta_ejemplo(moneda: CodigoMoneda) -> Cuenta {
        Cuenta {
            id: "cta_123".to_string(),
            nombre: "Cuenta Principal".to_string(),
            tipo: TipoCuenta::Banco,
            moneda,
            saldo_inicial: Monto::new(Decimal::new(100000, 2), moneda),
        }
    }

    #[test]
    fn test_creacion_gasto_valido() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let monto = Monto::new(Decimal::new(4550, 2), CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        let gasto = Movimiento::nuevo_gasto(
            "mov_1",
            &cuenta,
            monto,
            "Alimentación",
            "2026-10-20",
            "Almuerzo",
            hoy,
        )
        .expect("Debería crear el gasto válidamente");

        assert_eq!(gasto.id, "mov_1");
        assert_eq!(gasto.tipo, TipoMovimiento::Gasto);
        assert_eq!(gasto.monto.cantidad, Decimal::new(4550, 2));
        assert_eq!(gasto.categoria.unwrap().nombre, "Alimentación");
        assert!(gasto.id_movimiento_referenciado.is_none());
    }

    #[test]
    fn test_creacion_ingreso_valido() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::VES);
        let monto = Monto::new(Decimal::new(500000, 2), CodigoMoneda::VES);
        let hoy = "2026-10-20T12:00:00.000Z";

        let ingreso = Movimiento::nuevo_ingreso(
            "mov_2",
            &cuenta,
            monto,
            "Salario",
            "2026-10-20",
            "Nómina quincenal",
            hoy,
        )
        .expect("Debería crear el ingreso válidamente");

        assert_eq!(gasto_o_ingreso_id(&ingreso), "mov_2");
        assert_eq!(ingreso.tipo, TipoMovimiento::Ingreso);
        assert_eq!(ingreso.categoria.unwrap().nombre, "Salario");
    }

    fn gasto_o_ingreso_id(m: &Movimiento) -> &str {
        &m.id
    }

    #[test]
    fn test_rechaza_monto_cero_o_negativo_indicando_campo_monto() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        // Monto cero
        let monto_cero = Monto::new(Decimal::ZERO, CodigoMoneda::USD);
        let err_cero = Movimiento::nuevo_gasto(
            "m1",
            &cuenta,
            monto_cero,
            "Transporte",
            "2026-10-20",
            "Metro",
            hoy,
        )
        .expect_err("Debe rechazar monto cero");
        assert_eq!(err_cero.campo, "monto");
        assert!(err_cero.mensaje.contains("estrictamente mayor que cero"));

        // Monto negativo
        let monto_negativo = Monto::new(Decimal::new(-1000, 2), CodigoMoneda::USD);
        let err_negativo = Movimiento::nuevo_ingreso(
            "m2",
            &cuenta,
            monto_negativo,
            "Ventas",
            "2026-10-20",
            "Venta",
            hoy,
        )
        .expect_err("Debe rechazar monto negativo");
        assert_eq!(err_negativo.campo, "monto");
    }

    #[test]
    fn test_rechaza_monto_con_decimales_excesivos_indicando_campo_monto() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        // USD admite 2 decimales, pasamos 3
        let monto_3_dec = Monto::new(Decimal::new(25125, 3), CodigoMoneda::USD);
        let err = Movimiento::nuevo_gasto(
            "m1",
            &cuenta,
            monto_3_dec,
            "Servicios",
            "2026-10-20",
            "Luz",
            hoy,
        )
        .expect_err("Debe rechazar más de 2 decimales");

        assert_eq!(err.campo, "monto");
        assert!(err.mensaje.contains("más de 2 decimales"));
    }

    #[test]
    fn test_rechaza_moneda_distinta_a_la_de_la_cuenta() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        let monto_ves = Monto::new(Decimal::new(10000, 2), CodigoMoneda::VES);
        let err = Movimiento::nuevo_gasto(
            "m1",
            &cuenta,
            monto_ves,
            "Alimentación",
            "2026-10-20",
            "Comida",
            hoy,
        )
        .expect_err("Debe rechazar discrepancia de moneda");

        assert_eq!(err.campo, "moneda");
        assert!(err
            .mensaje
            .contains("no coincide con la moneda de la cuenta"));
    }

    #[test]
    fn test_rechaza_fecha_futura_indicando_campo_fecha() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let monto = Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        // Fecha del día de mañana
        let err =
            Movimiento::nuevo_gasto("m1", &cuenta, monto, "Salud", "2026-10-21", "Farmacia", hoy)
                .expect_err("Debe rechazar fecha futura");

        assert_eq!(err.campo, "fecha");
        assert!(err.mensaje.contains("es futura"));
    }

    #[test]
    fn test_rechaza_ingreso_con_fecha_futura_indicando_campo_fecha() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let monto = Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        let err = Movimiento::nuevo_ingreso(
            "m_futuro",
            &cuenta,
            monto,
            "Salario",
            "2026-10-21",
            "Nómina futura",
            hoy,
        )
        .expect_err("Debe rechazar ingreso con fecha futura");

        assert_eq!(err.campo, "fecha");
        assert!(err.mensaje.contains("es futura"));
    }

    #[test]
    fn test_rechaza_ingreso_con_formato_de_fecha_invalido() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let monto = Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        let err_vacia = Movimiento::nuevo_ingreso(
            "m_vacia",
            &cuenta,
            monto,
            "Salario",
            "   ",
            "Fecha vacía",
            hoy,
        )
        .expect_err("Debe rechazar fecha vacía");
        assert_eq!(err_vacia.campo, "fecha");

        let err_slash = Movimiento::nuevo_ingreso(
            "m_slash",
            &cuenta,
            monto,
            "Salario",
            "2026/10/20",
            "Formato incorrecto",
            hoy,
        )
        .expect_err("Debe rechazar fecha con /");
        assert_eq!(err_slash.campo, "fecha");

        let err_dia_invalido = Movimiento::nuevo_ingreso(
            "m_dia",
            &cuenta,
            monto,
            "Salario",
            "2026-02-30",
            "Día no existente",
            hoy,
        )
        .expect_err("Debe rechazar día inválido para el mes");
        assert_eq!(err_dia_invalido.campo, "fecha");
    }

    #[test]
    fn test_rechaza_categoria_de_tipo_equivocado_o_inexistente() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let monto = Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        // Usar categoría de Ingreso en un Gasto
        let err_tipo = Movimiento::nuevo_gasto(
            "m1",
            &cuenta,
            monto,
            "Salario",
            "2026-10-20",
            "Alquiler",
            hoy,
        )
        .expect_err("Debe rechazar categoría de ingreso en gasto");
        assert_eq!(err_tipo.campo, "categoria");

        // Usar categoría inexistente
        let err_inexistente = Movimiento::nuevo_ingreso(
            "m2",
            &cuenta,
            monto,
            "Inversiones Cripto",
            "2026-10-20",
            "Ganancia",
            hoy,
        )
        .expect_err("Debe rechazar categoría inexistente");
        assert_eq!(err_inexistente.campo, "categoria");
    }

    #[test]
    fn test_creacion_ajuste_valido() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        // Un ajuste puede tener cantidad positiva o negativa según compense
        let monto_ajuste = Monto::new(Decimal::new(-45000, 2), CodigoMoneda::USD);

        let ajuste = Movimiento::nuevo_ajuste(
            "ajuste_1",
            &cuenta,
            monto_ajuste,
            "mov_original_500",
            "2026-10-20",
            "Corrección de monto anotado por 500",
            hoy,
        )
        .expect("Debería crear el ajuste válidamente");

        assert_eq!(ajuste.tipo, TipoMovimiento::Ajuste);
        assert_eq!(
            ajuste.id_movimiento_referenciado.as_deref(),
            Some("mov_original_500")
        );
        assert!(ajuste.categoria.is_none());
    }

    #[test]
    fn test_ajuste_sin_movimiento_referenciado_falla() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";
        let monto_ajuste = Monto::new(Decimal::new(-45000, 2), CodigoMoneda::USD);

        let err = Movimiento::nuevo_ajuste(
            "ajuste_1",
            &cuenta,
            monto_ajuste,
            "   ",
            "2026-10-20",
            "Sin id original",
            hoy,
        )
        .expect_err("Debe fallar si no tiene movimiento referenciado");

        assert_eq!(err.campo, "movimiento_referenciado");
    }

    #[test]
    fn test_compensacion_matematica_ejemplo_2_2_gasto_anotado_500_corregido_a_50() {
        // Ejemplo Sección 2.2 de la Especificación:
        // Saldo inicial: 1800. Gasto anotado por 500, corregido a 50 -> saldo final 1.750.
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";
        let saldo_inicial = Monto::new(Decimal::new(180000, 2), CodigoMoneda::USD); // 1800.00

        // 1. Registro del gasto original por 500
        let gasto_500 = Movimiento::nuevo_gasto(
            "mov_original",
            &cuenta,
            Monto::new(Decimal::new(50000, 2), CodigoMoneda::USD),
            "Alimentación",
            "2026-10-20",
            "Cena",
            hoy,
        )
        .unwrap();

        // Saldo tras el gasto de 500: 1800 - 500 = 1300
        let saldo_tras_gasto = aplicar_movimiento_a_saldo(&saldo_inicial, &gasto_500).unwrap();
        assert_eq!(saldo_tras_gasto.cantidad, Decimal::new(130000, 2));

        // 2. Se corrige a 50: se debe generar un Ajuste de +450
        let monto_correcto = Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD); // 50.00
        let ajuste = Movimiento::crear_ajuste_por_correccion(
            "ajuste_corr",
            &cuenta,
            &gasto_500,
            &monto_correcto,
            "2026-10-20",
            "Corrección de gasto anotado por 500 a 50",
            hoy,
        )
        .unwrap();

        assert_eq!(ajuste.tipo, TipoMovimiento::Ajuste);
        assert_eq!(ajuste.monto.cantidad, Decimal::new(45000, 2)); // +450.00
        assert_eq!(
            ajuste.id_movimiento_referenciado.as_deref(),
            Some("mov_original")
        );

        // 3. Saldo final acumulado (INV-05): 1300 + 450 = 1750.00 exacto
        let saldo_final = aplicar_movimiento_a_saldo(&saldo_tras_gasto, &ajuste).unwrap();
        assert_eq!(saldo_final.cantidad, Decimal::new(175000, 2)); // 1750.00
    }

    #[test]
    fn test_compensacion_gasto_anotado_de_menos() {
        // Gasto anotado por 50, valor real era 500 -> ajuste debe restar -450
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";
        let gasto_50 = Movimiento::nuevo_gasto(
            "g1",
            &cuenta,
            Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD),
            "Transporte",
            "2026-10-20",
            "Taxi",
            hoy,
        )
        .unwrap();

        let monto_correcto = Monto::new(Decimal::new(50000, 2), CodigoMoneda::USD);
        let compensacion = calcular_diferencia_ajuste(&gasto_50, &monto_correcto).unwrap();
        assert_eq!(compensacion.cantidad, Decimal::new(-45000, 2)); // -450.00
    }

    #[test]
    fn test_compensacion_ingreso_anotado_de_menos_y_de_mas() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";

        // Caso 1: Ingreso anotado por 100, correcto es 150 -> ajuste debe sumar +50
        let ingreso_100 = Movimiento::nuevo_ingreso(
            "i1",
            &cuenta,
            Monto::new(Decimal::new(10000, 2), CodigoMoneda::USD),
            "Ventas",
            "2026-10-20",
            "Venta menor",
            hoy,
        )
        .unwrap();

        let monto_150 = Monto::new(Decimal::new(15000, 2), CodigoMoneda::USD);
        let comp_1 = calcular_diferencia_ajuste(&ingreso_100, &monto_150).unwrap();
        assert_eq!(comp_1.cantidad, Decimal::new(5000, 2)); // +50.00

        // Caso 2: Ingreso anotado por 200, correcto es 50 -> ajuste debe restar -150
        let ingreso_200 = Movimiento::nuevo_ingreso(
            "i2",
            &cuenta,
            Monto::new(Decimal::new(20000, 2), CodigoMoneda::USD),
            "Regalos",
            "2026-10-20",
            "Regalo",
            hoy,
        )
        .unwrap();

        let monto_50 = Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD);
        let comp_2 = calcular_diferencia_ajuste(&ingreso_200, &monto_50).unwrap();
        assert_eq!(comp_2.cantidad, Decimal::new(-15000, 2)); // -150.00
    }

    #[test]
    fn test_calcular_ajuste_rechaza_mismo_monto_o_monto_invalido() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";
        let gasto = Movimiento::nuevo_gasto(
            "g1",
            &cuenta,
            Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD),
            "Servicios",
            "2026-10-20",
            "Internet",
            hoy,
        )
        .unwrap();

        // Mismo monto
        let err_igual = calcular_diferencia_ajuste(
            &gasto,
            &Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD),
        )
        .expect_err("Debe rechazar monto idéntico");
        assert_eq!(err_igual.campo, "monto");

        // Monto <= 0
        let err_cero =
            calcular_diferencia_ajuste(&gasto, &Monto::new(Decimal::ZERO, CodigoMoneda::USD))
                .expect_err("Debe rechazar monto cero");
        assert_eq!(err_cero.campo, "monto");

        // Más decimales
        let err_dec = calcular_diferencia_ajuste(
            &gasto,
            &Monto::new(Decimal::new(50125, 3), CodigoMoneda::USD),
        )
        .expect_err("Debe rechazar exceso de decimales");
        assert_eq!(err_dec.campo, "monto");
    }

    #[test]
    fn test_rechaza_corregir_un_ajuste() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";
        let ajuste = Movimiento::nuevo_ajuste(
            "aj1",
            &cuenta,
            Monto::new(Decimal::new(1000, 2), CodigoMoneda::USD),
            "mov_orig",
            "2026-10-20",
            "Ajuste",
            hoy,
        )
        .unwrap();

        let err = calcular_diferencia_ajuste(
            &ajuste,
            &Monto::new(Decimal::new(2000, 2), CodigoMoneda::USD),
        )
        .expect_err("No se puede corregir un movimiento de tipo ajuste");

        assert_eq!(err.campo, "tipo");
    }

    #[test]
    fn test_criterio_2_todos_los_datos_invalidos_indican_campo_exacto_sin_trazas_tecnicas() {
        let cuenta = cuenta_ejemplo(CodigoMoneda::USD);
        let hoy = "2026-10-20T12:00:00.000Z";
        let monto_valido = Monto::new(Decimal::new(1000, 2), CodigoMoneda::USD);

        // 1. Monto cero -> campo "monto"
        let e1 = Movimiento::nuevo_gasto(
            "m1",
            &cuenta,
            Monto::new(Decimal::ZERO, CodigoMoneda::USD),
            "Alimentación",
            "2026-10-20",
            "Desc",
            hoy,
        )
        .unwrap_err();
        assert_eq!(e1.campo, "monto");
        assert!(!e1.mensaje.contains("stacktrace") && !e1.mensaje.contains("panic"));

        // 2. Monto negativo -> campo "monto"
        let e2 = Movimiento::nuevo_gasto(
            "m2",
            &cuenta,
            Monto::new(Decimal::new(-500, 2), CodigoMoneda::USD),
            "Alimentación",
            "2026-10-20",
            "Desc",
            hoy,
        )
        .unwrap_err();
        assert_eq!(e2.campo, "monto");

        // 3. Monto con decimales excesivos (USD admite 2, enviamos 3) -> campo "monto"
        let e3 = Movimiento::nuevo_gasto(
            "m3",
            &cuenta,
            Monto::new(Decimal::new(10125, 3), CodigoMoneda::USD),
            "Alimentación",
            "2026-10-20",
            "Desc",
            hoy,
        )
        .unwrap_err();
        assert_eq!(e3.campo, "monto");

        // 4. Categoría de tipo equivocado (Ingreso en Gasto) -> campo "categoria"
        let e4 = Movimiento::nuevo_gasto(
            "m4",
            &cuenta,
            monto_valido,
            "Salario",
            "2026-10-20",
            "Desc",
            hoy,
        )
        .unwrap_err();
        assert_eq!(e4.campo, "categoria");

        // 5. Categoría inexistente en catálogo -> campo "categoria"
        let e5 = Movimiento::nuevo_gasto(
            "m5",
            &cuenta,
            monto_valido,
            "InexistenteXYZ",
            "2026-10-20",
            "Desc",
            hoy,
        )
        .unwrap_err();
        assert_eq!(e5.campo, "categoria");

        // 6. Fecha futura (día de mañana) -> campo "fecha"
        let e6 = Movimiento::nuevo_gasto(
            "m6",
            &cuenta,
            monto_valido,
            "Alimentación",
            "2026-10-21",
            "Desc",
            hoy,
        )
        .unwrap_err();
        assert_eq!(e6.campo, "fecha");

        // 7. Fecha futura (mismo día con hora posterior en UTC) -> campo "fecha"
        let e7 = Movimiento::nuevo_gasto(
            "m7",
            &cuenta,
            monto_valido,
            "Alimentación",
            "2026-10-20T15:00:00.000Z",
            "Desc",
            hoy,
        )
        .unwrap_err();
        assert_eq!(e7.campo, "fecha");

        // 8. Fecha con formato inválido o vacía -> campo "fecha"
        let e8 = Movimiento::nuevo_gasto(
            "m8",
            &cuenta,
            monto_valido,
            "Alimentación",
            "fecha-invalida",
            "Desc",
            hoy,
        )
        .unwrap_err();
        assert_eq!(e8.campo, "fecha");

        // 9. Moneda de movimiento distinta a la de la cuenta -> campo "moneda"
        let e9 = Movimiento::nuevo_gasto(
            "m9",
            &cuenta,
            Monto::new(Decimal::new(1000, 2), CodigoMoneda::VES),
            "Alimentación",
            "2026-10-20",
            "Desc",
            hoy,
        )
        .unwrap_err();
        assert_eq!(e9.campo, "moneda");
    }

    #[test]
    fn test_parse_monto_movimiento_validaciones() {
        // Formato con punto y coma
        assert!(Monto::parse_movimiento("125.50", CodigoMoneda::USD, 2).is_ok());
        assert!(Monto::parse_movimiento("125,50", CodigoMoneda::USD, 2).is_ok());

        // Cero o negativo
        let err_cero = Monto::parse_movimiento("0,00", CodigoMoneda::USD, 2).unwrap_err();
        assert_eq!(err_cero.campo, "monto");

        let err_neg = Monto::parse_movimiento("-15.00", CodigoMoneda::USD, 2).unwrap_err();
        assert_eq!(err_neg.campo, "monto");

        // Exceso de decimales
        let err_dec = Monto::parse_movimiento("15.123", CodigoMoneda::USD, 2).unwrap_err();
        assert_eq!(err_dec.campo, "monto");

        // Texto no numérico
        let err_nan = Monto::parse_movimiento("invalido", CodigoMoneda::USD, 2).unwrap_err();
        assert_eq!(err_nan.campo, "monto");
    }

    #[test]
    fn test_error_dominio_serializa_con_contrato_exigido() {
        let err = ErrorDominio::categoria_invalida("Categoría errónea");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"campo\":\"categoria\""));
        assert!(json.contains("\"mensaje\":\"Categoría errónea\""));
    }
}
