# Documento técnico explicativo — Entrega 2

**Proyecto:** Cuentas Claras · **Equipo:** Grupo 3 — RumExecute
**Asignatura:** CI-3715 Ingeniería de Software · Universidad Simón Bolívar

## 1. Propósito y alcance

La Entrega 2 implementa la gestión de cuentas personales bajo el paradigma **Local-First** (*The Account Flow*):

1. Creación y consulta de cuentas personales (`Efectivo`, `Banco`, `Billetera digital`, `Ahorro`) indicando nombre, tipo, moneda y saldo inicial.
2. Persistencia local en el dispositivo mediante una base de datos SQLite embebida (`rusqlite`) bajo un esquema inmutable de **Event Sourcing** (`eventos_libro`).
3. Cálculo de subtotales estrictamente agrupados por moneda (`INV-02`), sin mezclar divisas distintas ni requerir conexión a internet o inicio de sesión (`RF-SOC-03`, `INV-16`).

## 2. Requerimientos e invariantes cubiertos

| Código | Descripción y cumplimiento en el sistema |
|---|---|
| `RF-LIB-01` / `RF-LIB-02` | Creación de cuentas personales con saldo inicial `>= 0` y consulta del listado de cuentas con subtotales separados por moneda. |
| `RF-SOC-03` | Apertura directa en la vista *"Mis Cuentas"* sin exigir registro ni inicio de sesión previo. |
| `INV-01` | Par indivisible: todo importe monetario se modela como `Monto { cantidad: Decimal, moneda: CodigoMoneda }`. |
| `INV-02` | Prohibición de sumar monedas distintas: los subtotales se calculan y muestran por separado para cada moneda (`VES`, `USD`, `EUR`, `USDT`). |
| `INV-03` / `RNF-08` | Aritmética decimal exacta mediante `rust_decimal::Decimal` (sin `f32`/`f64`), validando máximo 2 decimales y saldo inicial no negativo. |
| `INV-13` | Idempotencia: aplicar dos veces el mismo evento en la proyección o en SQLite deja exactamente el mismo estado que aplicarlo una vez. |
| `INV-16` / `RES-02` | Soberanía local: las cuentas personales viven exclusivamente en `cuentas_claras_libro.db` en el dispositivo y nunca se envían al servidor. |
| `INV-18` | Inmutabilidad de moneda: la moneda de una cuenta queda fijada al crearse y no puede modificarse posteriormente. |
| `RNF-18` | Errores estructurados en español que identifican el campo específico que falló (`nombre`, `tipo`, `moneda`, `saldo_inicial`) sin exponer trazas técnicas. |

## 3. Arquitectura hexagonal del cliente

El núcleo en Rust (`front_end/src-tauri/src/`) se estructura en tres capas desacopladas:

1. **Dominio (`src/dominio/`):**
   - `moneda.rs`, `monto.rs`, `cuenta.rs`, `evento.rs`, `errores.rs`.
   - Contiene las entidades puras, validaciones de formato, unicidad de nombre (insensible a mayúsculas y espacios extremos) y el catálogo de monedas de fábrica (`VES`, `USD`, `EUR`, `USDT`), sin depender de `rusqlite` ni `tauri`.
2. **Aplicación (`src/aplicacion/`):**
   - `crear_cuenta.rs`, `listar_cuentas.rs`, `listar_monedas.rs`, `puertos.rs`.
   - Define los traits `RepositorioLibro`, `Reloj` y `GeneradorId`. Reconstruye el estado de las cuentas a partir de la secuencia de eventos `EventoLibro::CuentaCreada` y calcula los subtotales por moneda.
3. **Infraestructura e IPC (`src/infraestructura/` y `src/comandos.rs`):**
   - `sqlite_repo.rs`: almacena los eventos en la tabla `eventos_libro` protegida por triggers SQLite (`BEFORE UPDATE` y `BEFORE DELETE`) que abortan cualquier intento de mutación o borrado.
   - `memoria_repo.rs` y `contrato_repo.rs`: verifican que tanto el repositorio en memoria como el repositorio SQLite cumplan exactamente el mismo contrato de persistencia.
   - `comandos.rs`: expone los comandos IPC `crear_cuenta`, `listar_cuentas` y `listar_monedas` consumidos por la interfaz Next.js.

## 4. Decisiones de diseño

- [ADR-002 — Arquitectura hexagonal y puertos](adr/ADR-002-arquitectura-hexagonal-y-puertos.md).
- [ADR-003 — Almacenamiento local SQLite y Event Sourcing](adr/ADR-003-almacenamiento-local-sqlite-y-event-sourcing.md).
