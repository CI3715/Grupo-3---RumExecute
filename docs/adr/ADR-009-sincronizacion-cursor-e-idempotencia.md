# ADR-009: Sincronización unidireccional por cursor, aplicación idempotente y cola offline

- **Estado:** Aceptado
- **Contexto:** Entrega 7
- **Fecha:** 2026-11-23

## Contexto

La Entrega 7 requiere que la parte de cada gasto compartido que le corresponde a un participante baje automáticamente del servidor a su libro personal como exactamente un movimiento de gasto en la cuenta personal que eligió para ese grupo (`RF-COM-10`, `INV-15`), sin enviar nunca información del libro personal al servidor (`INV-16` / `RES-02`) y permitiendo registrar gastos de grupo sin conexión mediante una cola local con reintentos seguros (`RNF-17`, `ESC-09`).

## Decisión

1. **Flujo unidireccional por cursor (`GET /cambios?desde=<cursor>`):**
   - Al registrarse un gasto de grupo, el servidor anexa en una tabla de solo crecimiento (`eventos_cambio_usuario`) un evento inmutable únicamente para cada participante incluido en la división (`INV-17`).
   - Cada evento posee un identificador único inmutable (`id` UUID v4) y una posición entera creciente (`posicion`), que actúa como cursor.
   - El endpoint `GET /cambios?desde=<cursor>` retorna en orden ascendente hasta 100 eventos del usuario autenticado posteriores al cursor indicado, junto con `nuevo_cursor` y `quedan_mas`.

2. **Elección local de cuenta del grupo y partes pendientes (`INV-16`, `INV-18`):**
   - La asociación entre un grupo y la cuenta personal donde se aplican las partes se guarda exclusivamente en el dispositivo (`local_cuentas_por_grupo`) y solo admite cuentas activas en la misma moneda del grupo (`INV-18`).
   - Si llega un evento de un grupo sin cuenta elegida, se conserva localmente en `local_partes_pendientes` y se aplica de forma automática en cuanto el usuario selecciona la cuenta.

3. **Aplicación idempotente y transacción atómica en el cliente (`INV-13`, `INV-14`, `INV-15`, `RF-COM-10`):**
   - Cada parte aplicada genera un evento local inmutable `GastoDeGrupoAplicado` que registra el `id_evento_servidor` en el índice único `local_eventos_servidor_aplicados`.
   - Antes de aplicar cualquier evento, el núcleo comprueba si `id_evento_servidor` ya fue procesado. Si ya existe, se ignora sin alterar saldos ni duplicar movimientos, incluso si el cursor local se reinicia a cero o la aplicación se interrumpe a mitad de camino.
   - Los movimientos del lote y la actualización del cursor (`local_sincronizacion_cursor`) se persisten dentro de una misma transacción atómica SQLite.
   - Los movimientos originados por gastos de grupo se marcan con `es_gasto_grupo = true` y concepto `"<Grupo> · <Concepto>"`, rechazando en el núcleo y en la interfaz cualquier intento de corrección desde el libro personal (`RF-COM-10`).

4. **Cola local sin conexión y clave de idempotencia (`RNF-17`, `ESC-09`):**
   - Cuando no hay conexión, el registro de un gasto de grupo se almacena en la cola local (`local_cola_gastos_offline`) con estado `"PENDIENTE"` y un identificador único `Idempotency-Key` generado una sola vez.
   - Al restablecerse la conexión, la cola se despacha en orden FIFO enviando el encabezado `Idempotency-Key` a `POST /grupos/{id}/gastos`.
   - El servidor almacena la huella canónica (`SHA-256`) del payload junto con la clave en `claves_idempotencia_gastos`: devuelve `201 Created` en el primer registro, `200 OK` con el gasto ya creado si se reintenta la misma clave con los mismos datos, y `409 Conflict` si se reutiliza la clave con datos distintos. Si el servidor rechaza una operación encolada, esta sale de la cola informando el motivo al usuario sin bloquear las demás.

## Alternativas consideradas

1. **Enviar al servidor el ID de la cuenta personal elegida:** se descarta porque violaría el principio de privacidad `INV-16` / `RES-02`, según el cual el servidor nunca conoce las cuentas ni saldos del libro personal.
2. **Confiar únicamente en el cursor sin guardar los `id_evento_servidor` aplicados:** se descarta porque un reinicio del cursor o un cierre inesperado antes de actualizar el cursor duplicaría movimientos en el libro personal (`INV-13`, `INV-15`).
3. **Bloquear toda la cola offline cuando falla una operación:** se descarta porque la especificación (`RNF-17`, Sección 2.5) exige que una operación rechazada por validación salga de la cola mostrando el motivo sin impedir el envío de las operaciones siguientes.

## Consecuencias

- El libro personal se mantiene estrictamente local y privado mientras incorpora de forma automática y sin duplicados las cuotas de gastos compartidos.
- La sincronización soporta reconexiones, reintentos, interrupciones a mitad de lote y reinicios de cursor manteniendo intacta la consistencia contable.
