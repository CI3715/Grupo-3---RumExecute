# ADR-003: Almacenamiento Local, SQLite y Event Sourcing

## Estado
Aceptado

## Fecha
2026-10-16

## Contexto
La propuesta de valor de **Cuentas Claras** exige que los datos del usuario sean manejados de manera local. Los registros de cuentas, ingresos y gastos no deben enviarse a servidores externos ni depender de una conexión a internet. Asimismo, se exige un modelo de **Event Sourcing** (Nunca se borran ni se actualizan datos existentes) para respaldar la inmutabilidad y la trazabilidad de los movimientos financieros.

Habiéndose seleccionado SQLite (`rusqlite`) como motor de persistencia local en el ADR-001, se requiere definir el diseño técnico de la persistencia: cómo se estructurará el registro inmutable de eventos en la base de datos, cómo se garantizará la integridad transaccional en disco y cómo la capa de aplicación proyectará el estado actual (saldos y balances) a partir del historial de eventos.

## Decisiones Adoptadas

### Diseño de Persistencia Append-Only sobre SQLite
* **Razón de Elección:**
  * **Inmutabilidad Estricta por Diseño:** La tabla principal de la base de datos (`eventos_libro`) se diseña con un esquema estrictamente acumulativo. Las operaciones del repositorio de infraestructura (`RepositorioLibroSqlite`) se limitan a `INSERT` y `SELECT`.
  * **Prohibición de `UPDATE` y `DELETE`:** Se evita la sobrescritura o eliminación de registros financieros a nivel de base de datos. Cualquier corrección de saldo o ajuste de cuenta se representa registrando un nuevo evento compensatorio en el log.
  * **Garantía Transaccional ACID Local:** El motor sincrónico de `rusqlite` se ejecuta dentro de transacciones atómicas. Si la aplicación sufre una falla o corte de energía a mitad del procesamiento, la base de datos no queda en un estado inconsistente o parcialmente escrito.

### Reconstrucción de Estado mediante Proyección de Eventos en Memoria
* **Razón de Elección:**

  * **Cómputo en Capa de Aplicación:** En lugar de mantener tablas de totales sincronizadas mediante triggers o consultas complejas, la capa de aplicación lee la secuencia ordenada de eventos desde el repositorio y aplica las reglas de dominio para proyectar los subtotales y balances en tiempo de ejecución.

  * **Trazabilidad y Auditoría Histórica:** Permite reconstruir el estado completo de cualquier cuenta en cualquier punto específico del tiempo, facilitando futuras funcionalidades de auditoría, exportación o conciliación de gastos compartidos.

  * **Aislamiento en Ruta Local:** La base de datos se aloja en el directorio de datos privado gestionado por el sistema operativo, garantizando que el archivo `.db` permanezca inaccesible para otras aplicaciones y totalmente aislado de la red.

## Consecuencias

### Positivas
* Cumplimiento estricto con las políticas de privacidad y soberanía de datos.
* Desempeño óptimo en lecturas y escrituras locales al evitar latencia de red.
* Facilidad de respaldo: el usuario puede migrar o respaldar su información copiando un único archivo `.db`.
* Facilidad para realizar pruebas unitarias y de integración: el comportamiento del repositorio de producción con SQLite en disco puede replicarse idénticamente con el repositorio en memoria (`memoria_repo.rs`) durante `cargo test`

### Negativas
* Crecimiento del Log de Eventos: Con el paso del tiempo y miles de transacciones, leer todo el historial de eventos desde el inicio puede impactar el rendimiento al calcular saldos.

* Evolución de Esquemas y Serde: Los eventos guardados en SQLite como estructuras serializadas requieren garantizar compatibilidad hacia atrás en Rust. Se mitiga definiendo versiones claras en los enums de eventos (`evento_libro`).
