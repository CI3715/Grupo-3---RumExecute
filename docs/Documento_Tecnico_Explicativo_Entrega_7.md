# Documento técnico explicativo — Entrega 7

**Proyecto:** Cuentas Claras · **Equipo:** Grupo 3 — RumExecute
**Asignatura:** CI-3715 Ingeniería de Software · Universidad Simón Bolívar

## 1. Propósito y alcance

Este documento describe la arquitectura, el protocolo de sincronización, las garantías de idempotencia y la estrategia de pruebas de la Entrega 7 (*Traer Cuentas del Grupo al Registro Personal — The Sync Flow*).

Partiendo de los grupos y gastos compartidos de la Entrega 6, la Entrega 7 conecta la aplicación cliente con el servidor para que cada gasto de grupo se refleje en el libro personal de cada participante como **exactamente un movimiento de gasto** por la parte que le corresponde, en la cuenta personal que el propio usuario eligió localmente (`RF-COM-10`, `INV-15`). Además, incorpora una cola local para registrar gastos de grupo sin conexión que se envían una sola vez al volver la red mediante claves de idempotencia (`RNF-17`, `ESC-09`).

En todo momento se preserva el principio rector: **la información solo baja, nunca sube** (`INV-16`, `RES-02`). El servidor entrega a cada usuario sus partes de los gastos de grupo; la aplicación decide en el dispositivo en qué cuenta aplicarlas y el servidor nunca conoce las cuentas personales, la cuenta elegida ni los saldos del usuario.

## 2. Requerimientos e invariantes cubiertos

| Área | Requerimientos y comportamiento |
|---|---|
| Bajada de partes al libro personal | Cada parte genera un movimiento de gasto con concepto `"<Grupo> · <Concepto>"`, marcado como gasto de grupo y no corregible desde el libro personal (`RF-COM-10`). |
| Elección de cuenta por grupo | Solo se ofrecen cuentas personales activas en la misma moneda del grupo (`INV-18`). La elección se guarda únicamente en el dispositivo (`INV-16`). |
| Partes pendientes de aplicar | Si llega una parte de un grupo sin cuenta elegida, se conserva en pendientes de aplicar con un aviso y se aplica automáticamente al elegir la cuenta. |
| Idempotencia en el cliente | Cada parte aplicada registra un evento `GastoDeGrupoAplicado` con el identificador único del evento del servidor (`INV-13`, `INV-15`). Sincronizar múltiples veces o reiniciar el cursor a cero no duplica movimientos ni altera saldos. |
| Inmutabilidad y atomicidad | La sincronización solo agrega eventos locales (`INV-14`), ignora tipos de evento desconocidos y guarda los movimientos del lote junto con el nuevo cursor en la misma transacción. |
| Aislamiento de eventos | Solo reciben el evento los participantes incluidos en la división del gasto; miembros excluidos o usuarios ajenos no reciben nada (`INV-17`). |
| Cola sin conexión e idempotencia | Los gastos de grupo registrados sin conexión se encolan localmente como `"Pendiente de envío"` con un encabezado `Idempotency-Key` único y se despachan en orden al volver la red (`RNF-17`, `ESC-09`). |

## 3. Arquitectura y responsabilidades

### 3.1. Servidor Backend (`servidor/`)

- **Dominio (`servidor/dominio/evento_cambio.py`, `idempotencia.py`):** Define las entidades inmutables `EventoCambioUsuario` / `NuevoEventoCambio` y el cálculo de huella determinista (`SHA-256`) sobre los datos del gasto para validar reintentos con `Idempotency-Key`.
- **Aplicación (`servidor/aplicacion/registrar_gasto_compartido.py`, `obtener_cambios_usuario.py`):**
  - Al registrar un gasto compartido, verifica si la `Idempotency-Key` ya fue procesada para ese usuario y grupo: si el payload coincide, devuelve el gasto existente (`es_reintento = True`, HTTP `200 OK`) sin duplicar el gasto ni emitir nuevos eventos; si el payload difiere, lanza `ErrorConflicto` (HTTP `409 Conflict`).
  - Cuando el gasto es nuevo (HTTP `201 Created`), persiste el gasto, sus cuotas y un evento en el registro de cambios para cada participante incluido en el reparto.
  - `ObtenerCambiosUsuario` valida que el cursor `desde` sea un entero `>= 0` y devuelve lotes ordenados de hasta 100 eventos junto con `nuevo_cursor` y `quedan_mas`.
- **Infraestructura y API (`servidor/infraestructura/`, `servidor/api/rutas_cambios.py`, `rutas_grupos.py`):**
  - Tablas PostgreSQL `eventos_cambio_usuario` (solo inserción y lectura por `posicion > cursor`) y `claves_idempotencia_gastos`.
  - Endpoint `GET /cambios?desde=<cursor>` y soporte del encabezado `Idempotency-Key` en `POST /grupos/{id}/gastos`.

### 3.2. Aplicación Cliente (`front_end/src-tauri/` y `front_end/src/`)

- **Dominio y Aplicación Local (`src-tauri/src/dominio/`, `src-tauri/src/aplicacion/`):**
  - Variante `EventoLibro::GastoDeGrupoAplicado`, atributo `es_gasto_grupo` en `Movimiento` y rechazo estricto en `corregir_movimiento` cuando `es_gasto_grupo == true`.
  - Casos de uso `SincronizarCambios`, `AsignarCuentaGrupo` (con verificación de moneda `INV-18`) y `EncolarGastoOffline` / `DespacharColaOffline`.
- **Persistencia SQLite e IPC (`src-tauri/src/infraestructura/sqlite_repo.rs`, `comandos.rs`):**
  - Tablas locales `local_eventos_servidor_aplicados`, `local_cuentas_por_grupo`, `local_partes_pendientes`, `local_sincronizacion_cursor` y `local_cola_gastos_offline`.
  - Transacción atómica en `aplicar_lote_cambios` que inserta los eventos `GastoDeGrupoAplicado` (o partes pendientes) y actualiza el cursor en un único `COMMIT`.
- **Interfaz de Usuario (`front_end/src/services/syncService.ts`, `VistaComunidad.tsx`, `DetalleCuenta.tsx`, `page.tsx`):**
  - Selector de cuenta personal por grupo filtrado por la moneda del grupo (`INV-18`), bandeja de partes pendientes de aplicar, barra de sincronización con fecha de última sincronización exitosa, botón de reinicio de cursor a cero, cola offline con etiqueta `"Pendiente de envío"`, avisos de rechazos de cola sin bloquear las demás operaciones y visualización de gastos de grupo en `DetalleCuenta` sin botón de corrección.

## 4. Contrato HTTP de la Entrega 7

El contrato formal OpenAPI 3.1 actualizado se encuentra en [`openapi.yaml`](../openapi.yaml).

| Operación | Método y ruta | Autorización | Encabezados | Respuesta exitosa | Errores |
|---|---|---|---|---|---|
| Obtener cambios | `GET /cambios?desde=<cursor>` | Bearer JWT | `Authorization` | `200 OK` (`eventos`, `nuevo_cursor`, `quedan_mas`) | `400` cursor inválido · `401` token ausente, inválido o vencido |
| Registrar gasto con idempotencia | `POST /grupos/{id}/gastos` | Bearer JWT (Miembro) | `Authorization`, `Idempotency-Key` *(opcional)* | `201 Created` (nuevo) · `200 OK` (clave ya procesada con mismos datos) | `400` datos inválidos · `401` sin token · `404` grupo inexistente o no miembro · `409` clave usada con datos distintos |

## 5. Decisiones de diseño

- [ADR-009 — Sincronización unidireccional por cursor, aplicación idempotente y cola offline](adr/ADR-009-sincronizacion-cursor-e-idempotencia.md).

## 6. Estrategia y ejecución de pruebas

1. **Pruebas del servidor (`servidor/tests/`):**
   - `tests/dominio/test_evento_cambio.py` y `tests/dominio/test_idempotencia.py`: validaciones de eventos de cambio, cursores y huellas de idempotencia.
   - `tests/aplicacion/test_obtener_cambios_usuario.py` y `tests/aplicacion/test_registrar_gasto_compartido.py`: paginación de hasta 100 eventos, emisión de eventos solo a participantes e idempotencia (`201`, `200`, `409`).
   - `tests/api/test_sincronizacion_e_idempotencia.py`: integración HTTP de `GET /cambios` y `POST /grupos/{id}/gastos` con `Idempotency-Key`.

2. **Pruebas del núcleo Rust (`front_end/src-tauri/`):**
   - `cargo test --manifest-path src-tauri/Cargo.toml`: verifica `asignar_cuenta_grupo` (`INV-18`), `sincronizar_cambios`, `cola_offline` (despacho FIFO y rechazo no bloqueante), rechazo de corrección sobre gastos de grupo (`RF-COM-10`) y transacción SQLite atómica con reinicio de cursor a cero (`sincronizacion_atomica_idempotencia_y_partes_pendientes`).

3. **Pruebas del frontend (`front_end/tests/`):**
   - `pnpm test`: verifica el flujo de sincronización, idempotencia ante lotes repetidos o reinicio de cursor, partes pendientes y cola offline.
