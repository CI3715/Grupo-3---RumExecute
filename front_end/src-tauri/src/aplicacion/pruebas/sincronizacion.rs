use crate::aplicacion::exportar_libro::{
    exportar_libro, EntradaExportarLibro, ErrorExportarLibro,
};
use crate::aplicacion::importar_copia::{importar_copia, ErrorImportarCopia};
use crate::aplicacion::puertos::sincronizacion::{
    AlmacenSecretosSincronizacion, CifradorLibro, CredencialesGit, DocumentoLibro,
    ErrorAlmacenSecretos, ErrorCifradoLibro, ErrorFormatoLibro,
    SerializadorLibro,
};
use crate::aplicacion::puertos::{GeneradorId, RepositorioLibro, Reloj};
use crate::aplicacion::sincronizar_libro::{sincronizar_libro, ErrorSincronizarLibro};
use crate::dominio::cuenta::TipoCuenta;
use crate::dominio::evento::{DatosCuentaCreada, EventoLibro};
use crate::dominio::moneda::CodigoMoneda;
use crate::dominio::monto::Monto;
use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;
use crate::infraestructura::transporte_memoria::TransporteMemoria;

const FRASE: &str = "frase-de-prueba-segura";
const FECHA: &str = "2026-11-25T18:00:00.000Z";

struct FormatoJson;

impl SerializadorLibro for FormatoJson {
    fn serializar(&self, documento: &DocumentoLibro) -> Result<String, ErrorFormatoLibro> {
        serde_json::to_string(documento).map_err(|_| ErrorFormatoLibro::ArchivoMalformado)
    }

    fn deserializar(&self, contenido: &str) -> Result<DocumentoLibro, ErrorFormatoLibro> {
        serde_json::from_str(contenido).map_err(|_| ErrorFormatoLibro::ArchivoMalformado)
    }
}

struct CifradorPrueba;

impl CifradorLibro for CifradorPrueba {
    fn esta_cifrado(&self, contenido: &[u8]) -> bool {
        contenido.starts_with(b"ENC:")
    }

    fn cifrar(&self, contenido: &[u8], frase_paso: &str) -> Result<Vec<u8>, ErrorCifradoLibro> {
        if frase_paso.chars().count() < 12 {
            return Err(ErrorCifradoLibro::OperacionFallida);
        }
        let mut cifrado = b"ENC:".to_vec();
        cifrado.extend_from_slice(contenido);
        Ok(cifrado)
    }

    fn descifrar(
        &self,
        contenido: &[u8],
        frase_paso: &str,
    ) -> Result<Vec<u8>, ErrorCifradoLibro> {
        if frase_paso != FRASE || !self.esta_cifrado(contenido) {
            return Err(ErrorCifradoLibro::FraseIncorrectaODatosDanados);
        }
        Ok(contenido[4..].to_vec())
    }
}

struct SecretosPrueba {
    frase: Option<String>,
}

impl AlmacenSecretosSincronizacion for SecretosPrueba {
    fn obtener_frase_paso(&self) -> Result<Option<String>, ErrorAlmacenSecretos> {
        Ok(self.frase.clone())
    }

    fn guardar_frase_paso(&mut self, frase_paso: &str) -> Result<(), ErrorAlmacenSecretos> {
        self.frase = Some(frase_paso.to_string());
        Ok(())
    }

    fn obtener_credenciales_git(&self) -> Result<Option<CredencialesGit>, ErrorAlmacenSecretos> {
        Ok(None)
    }

    fn guardar_credenciales_git(
        &mut self,
        _credenciales: &CredencialesGit,
    ) -> Result<(), ErrorAlmacenSecretos> {
        Ok(())
    }

    fn eliminar_credenciales_git(&mut self) -> Result<(), ErrorAlmacenSecretos> {
        Ok(())
    }
}

struct RelojFijo;

impl Reloj for RelojFijo {
    fn ahora(&self) -> String {
        FECHA.to_string()
    }
}

#[derive(Default)]
struct IdsSecuenciales(u32);

impl GeneradorId for IdsSecuenciales {
    fn nuevo_id(&mut self) -> String {
        self.0 += 1;
        format!("lote-{}", self.0)
    }
}

fn evento_cuenta(id_evento: &str, id_cuenta: &str, nombre: &str) -> EventoLibro {
    EventoLibro::nuevo_cuenta_creada(
        id_evento,
        FECHA,
        "dispositivo-prueba",
        DatosCuentaCreada {
            id_cuenta: id_cuenta.to_string(),
            nombre: nombre.to_string(),
            tipo: TipoCuenta::Efectivo,
            moneda: CodigoMoneda::USD,
            saldo_inicial: Monto::cero(CodigoMoneda::USD),
        },
    )
}

#[test]
fn exporta_el_libro_sin_modificarlo_y_no_incluye_secretos() {
    let mut repo = RepositorioLibroEnMemoria::nuevo();
    repo.guardar(evento_cuenta("evento-1", "cuenta-1", "Ahorros"))
        .unwrap();
    let serializador = FormatoJson;
    let copia = exportar_libro(
        &repo,
        &RelojFijo,
        &serializador,
        &CifradorPrueba,
        EntradaExportarLibro::default(),
    )
    .unwrap();

    let eventos: Vec<EventoLibro> = repo.listar_eventos().unwrap();
    let contenido = String::from_utf8(copia.contenido).unwrap();
    assert_eq!(copia.cantidad_eventos, 1);
    assert!(!copia.cifrada);
    assert_eq!(eventos.len(), 1);
    assert!(!contenido.contains(FRASE));
    assert!(!contenido.contains("token"));
    assert!(!contenido.contains("password"));
}

#[test]
fn exportacion_cifrada_exige_frase_valida_y_confirmacion_coincidente() {
    let repo = RepositorioLibroEnMemoria::nuevo();
    let serializador = FormatoJson;

    let error = exportar_libro(
        &repo,
        &RelojFijo,
        &serializador,
        &CifradorPrueba,
        EntradaExportarLibro {
            frase_paso: Some("demasiado-corta".to_string()),
            confirmacion_frase_paso: Some("otra-frase-larga".to_string()),
        },
    )
    .unwrap_err();
    assert!(matches!(error, ErrorExportarLibro::FrasesPasoNoCoinciden));

    let error = exportar_libro(
        &repo,
        &RelojFijo,
        &serializador,
        &CifradorPrueba,
        EntradaExportarLibro {
            frase_paso: Some("corta".to_string()),
            confirmacion_frase_paso: Some("corta".to_string()),
        },
    )
    .unwrap_err();
    assert!(matches!(error, ErrorExportarLibro::FrasePasoInvalida));

    let cifrada = exportar_libro(
        &repo,
        &RelojFijo,
        &serializador,
        &CifradorPrueba,
        EntradaExportarLibro {
            frase_paso: Some(FRASE.to_string()),
            confirmacion_frase_paso: Some(FRASE.to_string()),
        },
    )
    .unwrap();
    assert!(cifrada.cifrada);
    assert!(cifrada.contenido.starts_with(b"ENC:"));
}

#[test]
fn importar_es_idempotente_y_rechaza_version_antes_de_modificar_el_libro() {
    let serializador = FormatoJson;
    let cifrador = CifradorPrueba;
    let documento = DocumentoLibro::nuevo(
        FECHA,
        vec![evento_cuenta("evento-1", "cuenta-1", "Ahorros")],
    );
    let archivo = serializador.serializar(&documento).unwrap().into_bytes();
    let mut repo = RepositorioLibroEnMemoria::nuevo();

    let primera = importar_copia(&mut repo, &serializador, &cifrador, &archivo, None).unwrap();
    let segunda = importar_copia(&mut repo, &serializador, &cifrador, &archivo, None).unwrap();
    assert_eq!(primera.eventos_nuevos, 1);
    assert_eq!(segunda.eventos_existentes, 1);
    assert_eq!(repo.listar_eventos().unwrap().len(), 1);

    let mut version_invalida = documento;
    version_invalida.version_formato += 1;
    let archivo_invalido = serializador
        .serializar(&version_invalida)
        .unwrap()
        .into_bytes();
    let antes = repo.listar_eventos().unwrap();
    let error = importar_copia(
        &mut repo,
        &serializador,
        &cifrador,
        &archivo_invalido,
        None,
    )
    .unwrap_err();

    assert!(matches!(error, ErrorImportarCopia::VersionNoSoportada(_)));
    assert_eq!(repo.listar_eventos().unwrap(), antes);
}

#[test]
fn importar_rechaza_carga_invalida_antes_de_modificar_el_libro() {
    let serializador = FormatoJson;
    let evento_invalido = evento_cuenta("evento-invalido", "cuenta-1", "   ");
    let archivo = serializador
        .serializar(&DocumentoLibro::nuevo(FECHA, vec![evento_invalido]))
        .unwrap()
        .into_bytes();
    let mut repo = RepositorioLibroEnMemoria::nuevo();

    let error = importar_copia(
        &mut repo,
        &serializador,
        &CifradorPrueba,
        &archivo,
        None,
    )
    .unwrap_err();

    assert!(matches!(error, ErrorImportarCopia::ArchivoMalformado));
    assert!(repo.listar_eventos().unwrap().is_empty());
}

#[test]
fn importar_archivo_cifrado_rechaza_frase_incorrecta_sin_tocar_el_libro() {
    let serializador = FormatoJson;
    let cifrador = CifradorPrueba;
    let texto = serializador
        .serializar(&DocumentoLibro::nuevo(
            FECHA,
            vec![evento_cuenta("evento-1", "cuenta-1", "Ahorros")],
        ))
        .unwrap();
    let cifrado = cifrador.cifrar(texto.as_bytes(), FRASE).unwrap();
    let mut repo = RepositorioLibroEnMemoria::nuevo();

    let error = importar_copia(
        &mut repo,
        &serializador,
        &cifrador,
        &cifrado,
        Some("frase-incorrecta"),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ErrorImportarCopia::FraseIncorrectaODatosDanados
    ));
    assert!(repo.listar_eventos().unwrap().is_empty());

    let resumen = importar_copia(
        &mut repo,
        &serializador,
        &cifrador,
        &cifrado,
        Some(FRASE),
    )
    .unwrap();
    assert_eq!(resumen.eventos_nuevos, 1);
    assert_eq!(repo.listar_eventos().unwrap().len(), 1);
}

#[test]
fn importar_reporta_conflictos_y_conserva_eventos_locales() {
    let serializador = FormatoJson;
    let cifrador = CifradorPrueba;
    let evento_local = evento_cuenta("evento-1", "cuenta-local", "Cuenta local");
    let mut repo = RepositorioLibroEnMemoria::nuevo();
    repo.guardar(evento_local.clone()).unwrap();

    let archivo = serializador
        .serializar(&DocumentoLibro::nuevo(
            FECHA,
            vec![
                evento_cuenta("evento-1", "cuenta-remota", "Cuenta conflictiva"),
                evento_cuenta("evento-2", "cuenta-remota-2", "Cuenta nueva"),
            ],
        ))
        .unwrap()
        .into_bytes();
    let resumen = importar_copia(&mut repo, &serializador, &cifrador, &archivo, None).unwrap();
    let eventos = repo.listar_eventos().unwrap();

    assert_eq!(resumen.eventos_nuevos, 1);
    assert_eq!(resumen.eventos_rechazados.len(), 1);
    assert_eq!(resumen.eventos_rechazados[0].id_evento, "evento-1");
    assert!(eventos.contains(&evento_local));
    assert!(eventos.iter().any(|evento| evento.id() == "evento-2"));
}

#[test]
fn sincroniza_lotes_cifrados_reintenta_conflictos_y_no_reenvia_eventos_conocidos() {
    let serializador = FormatoJson;
    let cifrador = CifradorPrueba;
    let secretos = SecretosPrueba {
        frase: Some(FRASE.to_string()),
    };
    let mut transporte = TransporteMemoria::nuevo();
    let mut repo_a = RepositorioLibroEnMemoria::nuevo();
    repo_a
        .guardar(evento_cuenta("evento-1", "cuenta-1", "Ahorros"))
        .unwrap();
    transporte.simular_conflictos_de_publicacion(2);

    let informe_a = sincronizar_libro(
        &mut repo_a,
        &mut transporte,
        &serializador,
        &cifrador,
        &secretos,
        &RelojFijo,
        &mut IdsSecuenciales::default(),
        "dispositivo-a",
    )
    .unwrap();
    assert_eq!(informe_a.enviados, 1);
    assert_eq!(transporte.lotes_publicados().len(), 1);
    assert!(transporte.lotes_publicados()[0]
        .contenido
        .starts_with(b"ENC:"));

    let mut repo_b = RepositorioLibroEnMemoria::nuevo();
    let informe_b = sincronizar_libro(
        &mut repo_b,
        &mut transporte,
        &serializador,
        &cifrador,
        &secretos,
        &RelojFijo,
        &mut IdsSecuenciales::default(),
        "dispositivo-b",
    )
    .unwrap();
    assert_eq!(informe_b.recibidos, 1);
    assert_eq!(informe_b.enviados, 0);
    assert_eq!(repo_b.listar_eventos().unwrap().len(), 1);

    let segundo_informe = sincronizar_libro(
        &mut repo_b,
        &mut transporte,
        &serializador,
        &cifrador,
        &secretos,
        &RelojFijo,
        &mut IdsSecuenciales::default(),
        "dispositivo-b",
    )
    .unwrap();
    assert_eq!(segundo_informe.recibidos, 0);
    assert_eq!(transporte.lotes_publicados().len(), 1);
}

#[test]
fn sincronizacion_aborta_con_error_claro_tras_tres_conflictos() {
    let mut transporte = TransporteMemoria::nuevo();
    transporte.simular_conflictos_de_publicacion(3);
    let mut repo = RepositorioLibroEnMemoria::nuevo();
    repo.guardar(evento_cuenta("evento-1", "cuenta-1", "Ahorros"))
        .unwrap();

    let error = sincronizar_libro(
        &mut repo,
        &mut transporte,
        &FormatoJson,
        &CifradorPrueba,
        &SecretosPrueba {
            frase: Some(FRASE.to_string()),
        },
        &RelojFijo,
        &mut IdsSecuenciales::default(),
        "dispositivo-a",
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ErrorSincronizarLibro::ConflictosRepetidos
    ));
    assert!(transporte.lotes_publicados().is_empty());
}

#[test]
fn sincronizacion_falla_con_mensaje_claro_si_falta_la_frase_de_paso() {
    let mut repo = RepositorioLibroEnMemoria::nuevo();
    let mut transporte = TransporteMemoria::nuevo();
    let error = sincronizar_libro(
        &mut repo,
        &mut transporte,
        &FormatoJson,
        &CifradorPrueba,
        &SecretosPrueba { frase: None },
        &RelojFijo,
        &mut IdsSecuenciales::default(),
        "dispositivo-a",
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ErrorSincronizarLibro::FrasePasoNoConfigurada
    ));
}
