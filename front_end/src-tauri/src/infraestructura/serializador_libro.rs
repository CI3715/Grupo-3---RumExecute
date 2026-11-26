use crate::aplicacion::puertos::sincronizacion::{
    DocumentoLibro, ErrorFormatoLibro, SerializadorLibro, VERSION_FORMATO_LIBRO,
};
use crate::dominio::evento::EventoLibro;

const PREFIJO_FORMATO: &str = "CUENTAS_CLARAS_LIBRO v";
const PREFIJO_FECHA: &str = "FECHA_EXPORTACION: ";
const PREFIJO_CANTIDAD: &str = "CANTIDAD_EVENTOS: ";
const SEPARADOR_ENCABEZADO: &str = "---";

/// Implementación de `SerializadorLibro` en formato de texto estructurado abierto (v1).
///
/// Estructura:
/// - Línea 1: `CUENTAS_CLARAS_LIBRO v1`
/// - Línea 2: `FECHA_EXPORTACION: <ISO-8601 UTC>`
/// - Línea 3: `CANTIDAD_EVENTOS: <N>`
/// - Línea 4: `---`
/// - Líneas 5..N+4: Un objeto JSON por cada `EventoLibro` (sin saltos de línea internos).
#[derive(Debug, Default, Clone, Copy)]
pub struct SerializadorTextoLibro;

impl SerializadorTextoLibro {
    pub fn nuevo() -> Self {
        Self
    }
}

impl SerializadorLibro for SerializadorTextoLibro {
    fn serializar(&self, documento: &DocumentoLibro) -> Result<String, ErrorFormatoLibro> {
        if documento.version_formato != VERSION_FORMATO_LIBRO {
            return Err(ErrorFormatoLibro::VersionNoSoportada(
                documento.version_formato.to_string(),
            ));
        }

        let fecha = documento.fecha_exportacion.trim();
        if fecha.is_empty() || fecha.contains('\n') || fecha.contains('\r') {
            return Err(ErrorFormatoLibro::ArchivoMalformado);
        }

        if documento.cantidad_eventos != documento.eventos.len() {
            return Err(ErrorFormatoLibro::ArchivoMalformado);
        }

        let mut salida = String::new();
        salida.push_str(&format!("{PREFIJO_FORMATO}{VERSION_FORMATO_LIBRO}\n"));
        salida.push_str(&format!("{PREFIJO_FECHA}{fecha}\n"));
        salida.push_str(&format!("{PREFIJO_CANTIDAD}{}\n", documento.cantidad_eventos));
        salida.push_str(SEPARADOR_ENCABEZADO);
        salida.push('\n');

        for evento in &documento.eventos {
            let linea = serde_json::to_string(evento)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            if linea.contains('\n') || linea.contains('\r') {
                return Err(ErrorFormatoLibro::ArchivoMalformado);
            }
            salida.push_str(&linea);
            salida.push('\n');
        }

        Ok(salida)
    }

    fn deserializar(&self, contenido: &str) -> Result<DocumentoLibro, ErrorFormatoLibro> {
        let mut lineas = contenido.lines();

        let linea_version = lineas.next().ok_or(ErrorFormatoLibro::ArchivoMalformado)?;
        let version_str = linea_version
            .strip_prefix(PREFIJO_FORMATO)
            .ok_or(ErrorFormatoLibro::ArchivoMalformado)?
            .trim();
        let version_formato: u32 = version_str
            .parse()
            .map_err(|_| ErrorFormatoLibro::VersionNoSoportada(version_str.to_string()))?;
        if version_formato != VERSION_FORMATO_LIBRO {
            return Err(ErrorFormatoLibro::VersionNoSoportada(
                version_str.to_string(),
            ));
        }

        let linea_fecha = lineas.next().ok_or(ErrorFormatoLibro::ArchivoMalformado)?;
        let fecha_exportacion = linea_fecha
            .strip_prefix(PREFIJO_FECHA)
            .ok_or(ErrorFormatoLibro::ArchivoMalformado)?
            .trim()
            .to_string();
        if fecha_exportacion.is_empty() {
            return Err(ErrorFormatoLibro::ArchivoMalformado);
        }

        let linea_cantidad = lineas.next().ok_or(ErrorFormatoLibro::ArchivoMalformado)?;
        let cantidad_str = linea_cantidad
            .strip_prefix(PREFIJO_CANTIDAD)
            .ok_or(ErrorFormatoLibro::ArchivoMalformado)?
            .trim();
        let cantidad_eventos: usize = cantidad_str
            .parse()
            .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;

        let linea_sep = lineas.next().ok_or(ErrorFormatoLibro::ArchivoMalformado)?;
        if linea_sep.trim() != SEPARADOR_ENCABEZADO {
            return Err(ErrorFormatoLibro::ArchivoMalformado);
        }

        let mut eventos = Vec::with_capacity(cantidad_eventos);
        for linea in lineas {
            let limpia = linea.trim();
            if limpia.is_empty() {
                continue;
            }
            let evento: EventoLibro = serde_json::from_str(limpia)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            eventos.push(evento);
        }

        if eventos.len() != cantidad_eventos {
            return Err(ErrorFormatoLibro::ArchivoMalformado);
        }

        Ok(DocumentoLibro {
            version_formato,
            fecha_exportacion,
            cantidad_eventos,
            eventos,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::categoria::{Categoria, TipoCategoria};
    use crate::dominio::cuenta::TipoCuenta;
    use crate::dominio::evento::{
        DatosCategoriaCreada, DatosCuentaCreada, DatosGastoDeGrupoAplicado,
        DatosMonedaBaseCambiada, DatosMonedaCreada, DatosMovimientoCorregido,
        DatosMovimientoRegistrado, DatosTasaEliminada, DatosTasaRegistrada,
    };
    use crate::dominio::moneda::CodigoMoneda;
    use crate::dominio::monto::Monto;
    use crate::dominio::movimiento::TipoMovimiento;
    use crate::dominio::tasa_cambio::ParMonedas;
    use rust_decimal::Decimal;

    fn crear_todas_las_variantes() -> Vec<EventoLibro> {
        let usd = CodigoMoneda::USD;
        let ves = CodigoMoneda::VES;
        let par = ParMonedas::nuevo(usd.clone(), ves.clone()).unwrap();
        let cat = Categoria::nueva("cat-1", "Alimentación", TipoCategoria::Gasto).unwrap();

        vec![
            EventoLibro::nuevo_cuenta_creada(
                "evt-1",
                "2026-11-25T10:00:00Z",
                "disp-a",
                DatosCuentaCreada {
                    id_cuenta: "cta-1".to_string(),
                    nombre: "Efectivo USD".to_string(),
                    tipo: TipoCuenta::Efectivo,
                    moneda: usd.clone(),
                    saldo_inicial: Monto::new(Decimal::new(20000, 2), usd.clone()),
                },
            ),
            EventoLibro::nuevo_categoria_creada(
                "evt-2",
                "2026-11-25T10:01:00Z",
                "disp-a",
                DatosCategoriaCreada {
                    id_categoria: "cat-1".to_string(),
                    nombre: "Alimentación".to_string(),
                    tipo: TipoCategoria::Gasto,
                },
            ),
            EventoLibro::nuevo_movimiento_registrado(
                "evt-3",
                "2026-11-25T10:02:00Z",
                "disp-a",
                DatosMovimientoRegistrado {
                    id_movimiento: "mov-1".to_string(),
                    id_cuenta: "cta-1".to_string(),
                    tipo: TipoMovimiento::Gasto,
                    monto: Monto::new(Decimal::new(1550, 2), usd.clone()),
                    categoria: Some(cat),
                    fecha: "2026-11-25".to_string(),
                    descripcion: "Almuerzo, café y \"propina\"".to_string(),
                },
            ),
            EventoLibro::nuevo_movimiento_corregido(
                "evt-4",
                "2026-11-25T10:03:00Z",
                "disp-a",
                DatosMovimientoCorregido {
                    id_ajuste: "adj-1".to_string(),
                    id_cuenta: "cta-1".to_string(),
                    id_movimiento_original: "mov-1".to_string(),
                    monto_ajuste: Monto::new(Decimal::new(50, 2), usd.clone()),
                    monto_original: Monto::new(Decimal::new(1550, 2), usd.clone()),
                    monto_nuevo: Monto::new(Decimal::new(1500, 2), usd.clone()),
                    fecha: "2026-11-25".to_string(),
                    descripcion: "Ajuste por error".to_string(),
                },
            ),
            EventoLibro::nuevo_tasa_registrada(
                "evt-5",
                "2026-11-25T10:04:00Z",
                "disp-a",
                DatosTasaRegistrada {
                    par_monedas: par,
                    valor: Decimal::new(4550, 2),
                    fecha: "2026-11-25".to_string(),
                    fuente: "BCV".to_string(),
                },
            ),
            EventoLibro::nuevo_tasa_eliminada(
                "evt-6",
                "2026-11-25T10:05:00Z",
                "disp-a",
                DatosTasaEliminada {
                    par_monedas: par,
                    fecha: "2026-11-25".to_string(),
                    fuente: "BCV".to_string(),
                    id_tasa: Some("tasa-1".to_string()),
                },
            ),
            EventoLibro::nuevo_moneda_base_cambiada(
                "evt-7",
                "2026-11-25T10:06:00Z",
                "disp-a",
                DatosMonedaBaseCambiada {
                    nueva_moneda: ves,
                },
            ),
            EventoLibro::nuevo_moneda_creada(
                "evt-8",
                "2026-11-25T10:07:00Z",
                "disp-a",
                DatosMonedaCreada {
                    codigo: CodigoMoneda::parse("EUR").unwrap(),
                    nombre: "Euro".to_string(),
                    simbolo: "€".to_string(),
                    decimales: 2,
                },
            ),
            EventoLibro::nuevo_gasto_de_grupo_aplicado(
                "2026-11-25T10:08:00Z",
                "disp-a",
                DatosGastoDeGrupoAplicado {
                    id_evento_servidor: "srv-evt-1".to_string(),
                    id_movimiento: "mov-grp-1".to_string(),
                    id_cuenta: "cta-1".to_string(),
                    grupo_id: "grp-1".to_string(),
                    nombre_grupo: "Viaje a Mérida".to_string(),
                    gasto_id: "gasto-1".to_string(),
                    concepto: "Hotel".to_string(),
                    monto: Monto::new(Decimal::new(3333, 2), usd),
                    fecha: "2026-11-25".to_string(),
                },
            )
            .unwrap(),
        ]
    }

    #[test]
    fn round_trip_todas_las_variantes_de_evento() {
        let serializador = SerializadorTextoLibro::nuevo();
        let eventos = crear_todas_las_variantes();
        let doc = DocumentoLibro::nuevo("2026-11-25T18:00:00Z", eventos.clone());

        let texto = serializador.serializar(&doc).expect("Debe serializar");
        let lineas: Vec<&str> = texto.lines().collect();
        assert_eq!(lineas[0], "CUENTAS_CLARAS_LIBRO v1");
        assert_eq!(lineas[1], "FECHA_EXPORTACION: 2026-11-25T18:00:00Z");
        assert_eq!(lineas[2], "CANTIDAD_EVENTOS: 9");
        assert_eq!(lineas[3], "---");
        assert_eq!(lineas.len(), 4 + 9);

        let recuperado = serializador
            .deserializar(&texto)
            .expect("Debe deserializar");
        assert_eq!(recuperado, doc);
    }

    #[test]
    fn round_trip_libro_vacio() {
        let serializador = SerializadorTextoLibro::nuevo();
        let doc = DocumentoLibro::nuevo("2026-11-25T18:00:00Z", vec![]);

        let texto = serializador.serializar(&doc).unwrap();
        let recuperado = serializador.deserializar(&texto).unwrap();
        assert_eq!(recuperado, doc);
        assert_eq!(recuperado.cantidad_eventos, 0);
    }

    #[test]
    fn rechaza_version_no_soportada() {
        let serializador = SerializadorTextoLibro::nuevo();
        let contenido = "\
CUENTAS_CLARAS_LIBRO v2
FECHA_EXPORTACION: 2026-11-25T18:00:00Z
CANTIDAD_EVENTOS: 0
---
";
        let err = serializador.deserializar(contenido).unwrap_err();
        assert_eq!(err, ErrorFormatoLibro::VersionNoSoportada("2".to_string()));
    }

    #[test]
    fn rechaza_encabezado_sin_separador() {
        let serializador = SerializadorTextoLibro::nuevo();
        let contenido = "\
CUENTAS_CLARAS_LIBRO v1
FECHA_EXPORTACION: 2026-11-25T18:00:00Z
CANTIDAD_EVENTOS: 0
SIN_SEPARADOR
";
        let err = serializador.deserializar(contenido).unwrap_err();
        assert_eq!(err, ErrorFormatoLibro::ArchivoMalformado);
    }

    #[test]
    fn rechaza_si_cantidad_eventos_no_coincide() {
        let serializador = SerializadorTextoLibro::nuevo();
        let doc = DocumentoLibro::nuevo(
            "2026-11-25T18:00:00Z",
            vec![crear_todas_las_variantes().remove(0)],
        );
        let texto = serializador.serializar(&doc).unwrap();
        let alterado = texto.replace("CANTIDAD_EVENTOS: 1", "CANTIDAD_EVENTOS: 2");

        let err = serializador.deserializar(&alterado).unwrap_err();
        assert_eq!(err, ErrorFormatoLibro::ArchivoMalformado);
    }

    #[test]
    fn rechaza_linea_de_evento_corrupta() {
        let serializador = SerializadorTextoLibro::nuevo();
        let contenido = "\
CUENTAS_CLARAS_LIBRO v1
FECHA_EXPORTACION: 2026-11-25T18:00:00Z
CANTIDAD_EVENTOS: 1
---
{\"clase_evento\":\"NoExiste\"}
";
        let err = serializador.deserializar(contenido).unwrap_err();
        assert_eq!(err, ErrorFormatoLibro::ArchivoMalformado);
    }
}
