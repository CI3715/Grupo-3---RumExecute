use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntradaEncolarGastoDto {
    pub grupo_id: String,
    pub concepto: String,
    pub pagador_id: String,
    pub monto: String,
    pub tipo_division: String,
    pub participantes: Vec<String>,
    pub valores_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperacionGastoOffline {
    pub id: String,
    pub idempotency_key: String,
    pub grupo_id: String,
    pub concepto: String,
    pub pagador_id: String,
    pub monto: String,
    pub tipo_division: String,
    pub participantes: Vec<String>,
    pub valores_json: Option<String>,
    pub creado_en: String,
    pub estado: String, // "PENDIENTE", "ERROR"
    pub ultimo_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultadoDespachoColaDto {
    pub operaciones_procesadas: usize,
    pub operaciones_exitosas: usize,
    pub operaciones_fallidas: usize,
    pub errores: Vec<String>,
}

// --- Puertos (Traits) ---

/// Puerto para enviar el gasto al servidor HTTP mediante POST /grupos/{id}/gastos
pub trait ClienteEnvioGastoOffline: Send + Sync {
    fn enviar_gasto(&self, op: &OperacionGastoOffline) -> Result<(), String>;
}

/// Puerto para persistir y gestionar la cola local en SQLite
pub trait RepositorioColaOffline: Send + Sync {
    fn encolar(&mut self, op: &OperacionGastoOffline) -> Result<(), String>;
    fn listar_pendientes_fifo(&self) -> Result<Vec<OperacionGastoOffline>, String>;
    fn eliminar(&mut self, id: &str) -> Result<(), String>;
    fn registrar_error(&mut self, id: &str, error: &str) -> Result<(), String>;
}

// --- Casos de Uso ---

/// Caso de Uso 1: Encolar gasto registrado sin conexión (RNF-17, ESC-09)
pub struct EncolarGastoOffline<'a> {
    repositorio: &'a mut dyn RepositorioColaOffline,
}

impl<'a> EncolarGastoOffline<'a> {
    pub fn new(repositorio: &'a mut dyn RepositorioColaOffline) -> Self {
        Self { repositorio }
    }

    pub fn ejecutar(
        &mut self,
        entrada: EntradaEncolarGastoDto,
        momento_actual: &str,
    ) -> Result<OperacionGastoOffline, String> {
        if entrada.grupo_id.trim().is_empty() {
            return Err("El id del grupo es obligatorio".into());
        }
        if entrada.concepto.trim().is_empty() {
            return Err("El concepto del gasto es obligatorio".into());
        }
        if entrada.participantes.is_empty() {
            return Err("Debe incluir al menos un participante".into());
        }

        // Regla RNF-17: Generar clave de idempotencia una sola vez para esta operación
        let idempotency_key = Uuid::new_v4().to_string();
        let id_operacion = Uuid::new_v4().to_string();

        let operacion = OperacionGastoOffline {
            id: id_operacion,
            idempotency_key,
            grupo_id: entrada.grupo_id,
            concepto: entrada.concepto,
            pagador_id: entrada.pagador_id,
            monto: entrada.monto,
            tipo_division: entrada.tipo_division,
            participantes: entrada.participantes,
            valores_json: entrada.valores_json,
            creado_en: momento_actual.to_string(),
            estado: "PENDIENTE".to_string(),
            ultimo_error: None,
        };

        // INV-16: Solo se guardan los datos del gasto compartido, nada del libro personal
        self.repositorio.encolar(&operacion)?;

        Ok(operacion)
    }
}

/// Caso de Uso 2: Despachar la cola en orden FIFO al volver la conexión (RNF-17, ESC-09)
pub struct DespacharColaOffline<'a> {
    cliente_envio: &'a dyn ClienteEnvioGastoOffline,
    repositorio: &'a mut dyn RepositorioColaOffline,
}

impl<'a> DespacharColaOffline<'a> {
    pub fn new(
        cliente_envio: &'a dyn ClienteEnvioGastoOffline,
        repositorio: &'a mut dyn RepositorioColaOffline,
    ) -> Self {
        Self { cliente_envio, repositorio }
    }

    pub fn ejecutar(&mut self) -> Result<ResultadoDespachoColaDto, String> {
        let operaciones = self.repositorio.listar_pendientes_fifo()?;

        let mut exitosas = 0;
        let mut fallidas = 0;
        let mut errores = Vec::new();

        for op in &operaciones {
            match self.cliente_envio.enviar_gasto(op) {
                Ok(()) => {
                    // Si el servidor aceptó (201) o confirmó idempotencia (200), sale de la cola
                    let _ = self.repositorio.eliminar(&op.id);
                    exitosas += 1;
                }
                Err(motivo) => {
                    // Regla RNF-17 / ESC-09:
                    // Si el servidor rechaza (ej. porcentajes inválidos), sale de la cola reportando
                    // el motivo al usuario, ¡SIN BLOQUEAR LAS DEMÁS OPERACIONES!
                    let _ = self.repositorio.eliminar(&op.id);
                    let _ = self.repositorio.registrar_error(&op.id, &motivo);
                    errores.push(format!("Gasto '{}': {}", op.concepto, motivo));
                    fallidas += 1;
                    // El bucle continúa normalmente con la siguiente operación
                }
            }
        }

        Ok(ResultadoDespachoColaDto {
            operaciones_procesadas: operaciones.len(),
            operaciones_exitosas: exitosas,
            operaciones_fallidas: fallidas,
            errores,
        })
    }
}

// --- Pruebas Unitarias ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockRepoCola {
        cola: Vec<OperacionGastoOffline>,
        eliminados: Vec<String>,
    }

    impl RepositorioColaOffline for MockRepoCola {
        fn encolar(&mut self, op: &OperacionGastoOffline) -> Result<(), String> {
            self.cola.push(op.clone());
            Ok(())
        }

        fn listar_pendientes_fifo(&self) -> Result<Vec<OperacionGastoOffline>, String> {
            Ok(self.cola.clone())
        }

        fn eliminar(&mut self, id: &str) -> Result<(), String> {
            self.eliminados.push(id.to_string());
            self.cola.retain(|op| op.id != id);
            Ok(())
        }

        fn registrar_error(&mut self, _id: &str, _error: &str) -> Result<(), String> {
            Ok(())
        }
    }

    struct MockClienteEnvio {
        respuestas: HashMap<String, Result<(), String>>,
    }

    impl ClienteEnvioGastoOffline for MockClienteEnvio {
        fn enviar_gasto(&self, op: &OperacionGastoOffline) -> Result<(), String> {
            self.respuestas
                .get(&op.id)
                .cloned()
                .unwrap_or(Ok(()))
        }
    }

    #[test]
    fn test_encolar_genera_clave_idempotencia_y_estado_pendiente() {
        let mut repo = MockRepoCola {
            cola: vec![],
            eliminados: vec![],
        };
        let mut caso_uso = EncolarGastoOffline::new(&mut repo);

        let entrada = EntradaEncolarGastoDto {
            grupo_id: "g-1".into(),
            concepto: "Cena sin conexión".into(),
            pagador_id: "u-1".into(),
            monto: "60.00".into(),
            tipo_division: "IGUALES".into(),
            participantes: vec!["u-1".into(), "u-2".into()],
            valores_json: None,
        };

        let op = caso_uso.ejecutar(entrada, "2026-11-22T15:00:00Z").unwrap();

        assert_eq!(op.estado, "PENDIENTE");
        assert!(!op.idempotency_key.is_empty());
        assert_eq!(repo.cola.len(), 1);
        assert_eq!(repo.cola[0].idempotency_key, op.idempotency_key);
    }

    #[test]
    fn test_despacho_fifo_exitoso_elimina_de_cola() {
        let op1 = OperacionGastoOffline {
            id: "op-1".into(),
            idempotency_key: "key-1".into(),
            grupo_id: "g-1".into(),
            concepto: "Almuerzo".into(),
            pagador_id: "u-1".into(),
            monto: "30.00".into(),
            tipo_division: "IGUALES".into(),
            participantes: vec!["u-1".into()],
            valores_json: None,
            creado_en: "2026-11-22T10:00:00Z".into(),
            estado: "PENDIENTE".into(),
            ultimo_error: None,
        };
        let op2 = OperacionGastoOffline {
            id: "op-2".into(),
            idempotency_key: "key-2".into(),
            grupo_id: "g-1".into(),
            concepto: "Cena".into(),
            pagador_id: "u-1".into(),
            monto: "40.00".into(),
            tipo_division: "IGUALES".into(),
            participantes: vec!["u-1".into()],
            valores_json: None,
            creado_en: "2026-11-22T11:00:00Z".into(),
            estado: "PENDIENTE".into(),
            ultimo_error: None,
        };

        let mut repo = MockRepoCola {
            cola: vec![op1, op2],
            eliminados: vec![],
        };
        let cliente = MockClienteEnvio {
            respuestas: HashMap::new(),
        };

        let mut caso_uso = DespacharColaOffline::new(&cliente, &mut repo);
        let res = caso_uso.ejecutar().unwrap();

        assert_eq!(res.operaciones_procesadas, 2);
        assert_eq!(res.operaciones_exitosas, 2);
        assert_eq!(res.operaciones_fallidas, 0);
        assert_eq!(repo.cola.len(), 0);
        assert_eq!(repo.eliminados, vec!["op-1", "op-2"]);
    }

    #[test]
    fn test_operacion_rechazada_no_bloquea_las_siguientes_rnf_17() {
        let op_invalida = OperacionGastoOffline {
            id: "op-invalida".into(),
            idempotency_key: "key-bad".into(),
            grupo_id: "g-1".into(),
            concepto: "Gasto con error".into(),
            pagador_id: "u-1".into(),
            monto: "50.00".into(),
            tipo_division: "PORCENTAJES".into(),
            participantes: vec!["u-1".into()],
            valores_json: None,
            creado_en: "2026-11-22T10:00:00Z".into(),
            estado: "PENDIENTE".into(),
            ultimo_error: None,
        };
        let op_valida = OperacionGastoOffline {
            id: "op-valida".into(),
            idempotency_key: "key-good".into(),
            grupo_id: "g-1".into(),
            concepto: "Gasto correcto".into(),
            pagador_id: "u-1".into(),
            monto: "20.00".into(),
            tipo_division: "IGUALES".into(),
            participantes: vec!["u-1".into()],
            valores_json: None,
            creado_en: "2026-11-22T11:00:00Z".into(),
            estado: "PENDIENTE".into(),
            ultimo_error: None,
        };

        let mut repo = MockRepoCola {
            cola: vec![op_invalida, op_valida],
            eliminados: vec![],
        };

        let mut respuestas = HashMap::new();
        respuestas.insert("op-invalida".into(), Err("400 Porcentajes no suman 100%".into()));
        respuestas.insert("op-valida".into(), Ok(()));

        let cliente = MockClienteEnvio { respuestas };

        let mut caso_uso = DespacharColaOffline::new(&cliente, &mut repo);
        let res = caso_uso.ejecutar().unwrap();

        assert_eq!(res.operaciones_procesadas, 2);
        assert_eq!(res.operaciones_exitosas, 1);
        assert_eq!(res.operaciones_fallidas, 1);
        assert_eq!(res.errores.len(), 1);
        assert!(res.errores[0].contains("Porcentajes no suman 100%"));

        assert_eq!(repo.cola.len(), 0);
    }
}