# Documento Técnico Explicativo — Entrega 4: The Exchange Flow

**Proyecto:** Cuentas Claras · **Equipo:** Grupo 3 — RumExecute · **Asignatura:** CI-3715 Ingeniería de Software (USB)

## 1. Resumen y alcance

La Entrega 4 incorpora el manejo de múltiples monedas, el registro manual de tasas de cambio históricas, la conversión directa e inversa y la pantalla de Patrimonio consolidado en una Moneda Base elegida por el usuario.

| Requerimiento | Qué se implementa |
|---|---|
| RF-CAM-01 | Catálogo de fábrica (VES, USD, EUR, USDT) protegido y monedas propias |
| RF-CAM-02 | Moneda base configurable (por defecto USD) sin mutar saldos |
| RF-CAM-03 | Registro de tasas, tasa inversa `1/valor`, selección por fecha |
| RF-CAM-04 | Detección estricta de tasa faltante |
| RF-CAM-06 (básica) | Historial por par y eliminación mediante evento |
| RF-INF-01 | Pantalla de Patrimonio consolidado |

Invariantes: INV-01 (par cantidad+moneda), INV-02 (no sumar monedas distintas), INV-03 (decimal exacto), INV-13 (eventos inmutables), INV-16 (local-first), INV-18 (moneda de cuenta inmutable).

## 2. Arquitectura por capas

Se mantiene la arquitectura hexagonal (ADR-002) en `front_end/src-tauri/src/`:

- **`dominio/`** (sin SQLite ni Tauri): `moneda.rs` (catálogo y monedas propias), `tasa_cambio.rs` (par de monedas, tasa, historial, inversa, desempate), `conversion.rs` (conversión con redondeo por decimales de la moneda destino), `evento.rs` (eventos nuevos).
- **`aplicacion/`**: casos de uso `registrar_tasa`, `eliminar_tasa`, `cambiar_moneda_base`, `crear_moneda`, `consultar_patrimonio`, `listar_historial_tasas`, además de `dto.rs` y `proyeccion_monedas.rs`. Dependen solo de los puertos `RepositorioLibro`, `Reloj` y `GeneradorId`.
- **`infraestructura/`**: `RepositorioLibroSqlite` (tabla `eventos_libro` con triggers) y repositorio en memoria.
- **`comandos.rs` / `lib.rs`**: adaptadores Tauri IPC.
- **Interfaz (`front_end/src/`)**: Next.js; `VistaPatrimonio.tsx`, `VistaTasas.tsx`, `tasasService.ts` y `types/tasas.ts`. La interfaz solo presenta y recoge entrada; las reglas de negocio viven en Rust.

## 3. Modelo de eventos

Se añaden cuatro eventos inmutables al enum `EventoLibro`, cada uno con `id`, `tipo`, `momento_registro`, `dispositivo_origen` y `datos`:

| Evento | Lo emite |
|---|---|
| `MonedaCreada` | `crear_moneda` |
| `MonedaBaseCambiada` | `cambiar_moneda_base` |
| `TasaRegistrada` | `registrar_tasa` |
| `TasaEliminada` | `eliminar_tasa` |

El estado (catálogo de monedas, moneda base, historial de tasas) se reconstruye por **replay** de los eventos en orden de secuencia. Eliminar una tasa no borra nada: el evento `TasaEliminada` la excluye del historial proyectado. Los triggers `BEFORE UPDATE` y `BEFORE DELETE` de `eventos_libro` rechazan cualquier mutación (INV-13).

## 4. Algoritmo de `consultar_patrimonio`

1. Se resuelve la fecha de consulta (por defecto, la fecha actual del reloj; no se aceptan fechas futuras) y se reconstruyen las cuentas a esa fecha: una cuenta existe desde el día en que fue registrada, y cada movimiento o ajuste cuenta desde su fecha declarada. Los eventos con `id` repetido se ignoran (INV-13). Se proyectan también la moneda base y las tasas activas (las eliminadas se excluyen).
2. Se agrupan los saldos por moneda original en **subtotales separados** (INV-02).
3. Para cada cuenta se busca la tasa vigente hacia la moneda base:
   - Si la moneda de la cuenta es la base, la tasa aplicada es 1.
   - Se busca primero una tasa **directa** A→B vigente. Solo si no existe, se usa la **inversa** B→A como `1/valor`. No se compara la fecha entre ambas direcciones.
   - La tasa vigente es la de mayor `fecha_vigencia <= T`; con la misma fecha gana la de mayor `orden_registro` (la registrada después).
   - No existe conversión triangular: nunca se infiere A→C a partir de A→B y B→C.
4. Cada saldo se convierte con aritmética `Decimal` sin redondear, se acumula, y el redondeo a los decimales de la moneda base se aplica **una sola vez** al total final.
5. **RF-CAM-04:** si una cuenta con saldo no nulo no tiene tasa, se devuelven cuentas y subtotales, `total_consolidado = None` y `tasa_faltante` con el par y la fecha requerida. Solo se reporta el primer par faltante encontrado. Las cuentas con saldo cero no disparan la alerta.

## 5. Contratos IPC

| Comando | Entrada | Resultado |
|---|---|---|
| `consultar_patrimonio` | `fechaConsulta` (opcional, `YYYY-MM-DD`) | `PatrimonioDto` |
| `registrar_tasa` | `RegistrarTasaDto { origen, destino, valor, fecha_vigencia, fuente }` | `{ exito, id_tasa }` |
| `eliminar_tasa` | `idTasa` | `{ exito }` |
| `cambiar_moneda_base` | `nuevoCodigo` | `{ exito, moneda_base }` |
| `crear_moneda` | `CrearMonedaDto { codigo, nombre, simbolo, decimales }` | `{ exito, codigo }` |
| `listar_historial_tasas` | `ListarHistorialTasasEntrada { origen, destino }` (ambos vacíos = todos los pares; si se indica uno, el otro es obligatorio) | Lista de `TasaCambioDto { id, origen, destino, valor, fecha_vigencia, fuente, eliminada }`; incluye tasas eliminadas marcadas con `eliminada: true` |
| `listar_monedas` | — | catálogo de fábrica + monedas propias |

Los errores de validación devuelven `{ campo, mensaje }` en español, sin trazas técnicas (RNF-18).
`PatrimonioDto` incluye además `tasas_usadas` (tasa registrada, tasa efectiva, fecha, fuente y si fue inversa).

## 6. Estrategia de pruebas

Total: **138 tests en Rust, todos en verde** (`cargo test`).

- **Dominio (equipo, inline):** `tasa_cambio`, `conversion` y `moneda` cubren validaciones de tasa (valor ≤ 0, fecha futura, más de 8 decimales, fuente vacía, monedas iguales), inversa, tasa vigente, desempate, eliminación, conversión con redondeo y monedas propias.
- **`aplicacion/pruebas_entrega4.rs` (14 tests, repositorio en memoria):** escenario oficial 2.3 (75.00 a t₁ y 70.00 a t₂), RF-CAM-04 con cuenta en EUR, inversa al cambiar la base a VES, ausencia de triangulación, cambio de base sin mutar saldos, tasa eliminada, validaciones de `registrar_tasa` y 7 tests de `crear_moneda`.
- **`infraestructura/pruebas_entrega4_sqlite.rs` (2 tests, SQLite real):** los cuatro eventos nuevos se guardan y rechazan `UPDATE` y `DELETE` desde una segunda conexión; el escenario 2.3 persiste y se reconstruye tras cerrar y reabrir la base (tasa, tasa eliminada, moneda base y moneda propia).
- **Frontend:** `front_end/tests/tasasYPatrimonio.test.ts`, ejecutable con `pnpm test`.

## 7. Reproducción de la demo en vivo

Sigue el checklist de la Sección 5 de la guía de la entrega:

1. Verificar en la interfaz que VES, USD, EUR y USDT no se pueden editar ni borrar; crear la moneda `CAD` con 2 decimales.
2. Cambiar la moneda base y confirmar que los saldos originales no cambian.
3. Con una cuenta VES de 1.000 y otra USD de 50 (creadas antes de t₁ si se quieren consultar fechas pasadas; si se crean el día de la demo, consultar a hoy): consultar sin tasa (alerta VES→USD), registrar 0.025 con vigencia t₁ (total 75.00), registrar 0.020 con vigencia t₂ (total 70.00 a la fecha más reciente; a t₁ sigue 0.025 si las cuentas existían), eliminar la de 0.020 y verificar que vuelve a aplicar 0.025.
4. Con solo VES→USD registrada, cambiar la base a VES y verificar la inversa.
5. Con VES→USD y EUR→USD, cambiar la base a EUR y verificar que no se inventa VES→EUR.
6. Intentar tasas con fecha futura, valor ≤ 0 o monedas iguales y verificar el rechazo.
7. Consultar el historial de un par y eliminar una tasa; verificar que deja de aplicar y que el evento persiste.
8. Ejecutar `cargo test`.

## 8. Limitaciones conocidas

- La alerta de tasa faltante informa un único par por consulta.
- Una tasa directa antigua prevalece sobre una inversa más reciente.
- El usuario debe registrar manualmente cada par; no hay triangulación.
- El replay procesa todo el log en cada consulta (ver ADR-003).
- Los saldos se proyectan desde los eventos hasta la fecha de consulta: una cuenta existe a partir de la fecha en que fue registrada, y cada movimiento o ajuste cuenta desde su fecha declarada. Esa misma fecha selecciona la tasa aplicable. No se permiten fechas de consulta futuras.