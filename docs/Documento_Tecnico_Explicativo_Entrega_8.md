# Documento técnico explicativo — Entrega 8

**Proyecto:** Cuentas Claras · **Equipo:** Grupo 3 — RumExecute  
**Asignatura:** CI-3715 Ingeniería de Software · Universidad Simón Bolívar

## 1. Propósito y alcance

Este documento describe la arquitectura, el formato abierto de respaldo, el diseño criptográfico, el protocolo de sincronización entre dispositivos y la estrategia de pruebas de la Entrega 8 (*Sincronización y Copia de Seguridad Personal*).

La Entrega 8 permite al usuario **exportar** copias de seguridad de su libro personal (cifradas o en texto legible), **importarlas** en una instalación nueva o existente mediante fusión idempotente sin pérdida de eventos locales, y **sincronizar su libro entre múltiples dispositivos propios** usando una carpeta compartida o un repositorio Git privado (`RF-INT-01` a `RF-INT-07`).

En cumplimiento estricto de `RNF-01` e `INV-16`, **el servidor backend de Cuentas Claras no participa en esta entrega**: no se agregan endpoints ni viaja ningún dato del libro personal, saldo, credencial o frase de paso hacia el servidor.

## 2. Requerimientos e invariantes cubiertos

| Área | Requerimientos y comportamiento |
|---|---|
| Exportación abierta y cifrada | Genera un documento con todos los eventos del libro sin modificarlo (`RF-INT-01`, `RNF-04`). Soporta formato sin cifrar (con advertencia previa) y formato cifrado con frase de paso confirmada de al menos 12 caracteres. |
| Cifrado autenticado y manejo de errores | Deriva claves con **Argon2id** (sal aleatoria nueva por exportación o lote) y cifra con **XChaCha20-Poly1305** autenticando el encabezado (`RF-INT-06`, `ESC-03`). Ante frase incorrecta o manipulación devuelve `"Frase de paso incorrecta o archivo dañado."` (`RNF-18`). |
| Importación y fusión idempotente | Descifra y valida todo el archivo antes de tocar SQLite (`RF-INT-02`, `ESC-04`). Fusiona en una sola transacción como unión de conjuntos por `id` (`INV-13`, `INV-14`): eventos nuevos se agregan, idénticos se ignoran y repetidos con contenido distinto se rechazan con motivo sin dañar el libro. |
| Determinismo entre dispositivos | Identifica cada instalación con un `dispositivo_id` único, genera `id` determinista para partes de gastos de grupo (`gasto_grupo:<id_evento_servidor>`) y proyecta saldos con orden cronológico total `(momento_registro, dispositivo_origen, id)`. |
| Sincronización entre dispositivos | Ejecuta el ciclo de 4 pasos (**Traer → Fusionar → Enviar → Publicar**) sobre adaptadores intercambiables (`RNF-14`, `RNF-15`): Carpeta compartida o Repositorio Git (`RF-INT-03`, `RF-INT-04`, `RF-INT-07`, `RNF-10`). |
| Protección de secretos | La frase de paso y las credenciales de Git se almacenan exclusivamente en el gestor de secretos del sistema operativo (**Keyring**), nunca en SQLite, exportaciones ni URLs (`RF-INT-05`). |

## 3. Arquitectura y responsabilidades

Toda la implementación de la Entrega 8 reside en la aplicación cliente (`front_end/src-tauri/` y `front_end/src/`), siguiendo Arquitectura Hexagonal.

### 3.1. Capa de Dominio (`front_end/src-tauri/src/dominio/`)

- **Identidad de dispositivo (`dispositivo.rs`):** Modela `DispositivoId` para identificar el origen de cada evento generado localmente.
- **Eventos, orden y fusión (`evento.rs`):**
  - `nuevo_gasto_de_grupo_aplicado` e `id_movimiento_gasto_grupo` generan identificadores deterministas a partir de `id_evento_servidor` para evitar duplicados cuando distintos dispositivos sincronizan el mismo gasto de grupo desde el servidor.
  - Orden determinista total mediante instantes UTC (`chrono`) usando la tupla `(momento_registro, dispositivo_origen, id)` para garantizar saldos idénticos independientemente del orden de llegada.
  - Función pura de fusión de eventos como unión por `id`, devolviendo `ResultadoFusion` (`eventos_resultantes`, `eventos_nuevos`, `eventos_existentes`, `eventos_rechazados`) e `InformeSincronizacion` (`enviados`, `recibidos`, `rechazados`, `fecha_ultima_sincronizacion`).
- **Errores y validación de frase (`errores.rs`):** Define `LONGITUD_MINIMA_FRASE_PASO = 12`, rechaza frases vacías o con solo espacios y expone errores de dominio legibles sin trazas técnicas (`RNF-18`).

### 3.2. Capa de Aplicación y Puertos (`front_end/src-tauri/src/aplicacion/`)

- **Puertos de sincronización (`puertos/sincronizacion.rs`):**
  - Define `DocumentoLibro` (`VERSION_FORMATO_LIBRO = 1`), `LoteSincronizacion`, `CredencialesGit` y los traits `SerializadorLibro`, `CifradorLibro`, `AlmacenSecretosSincronizacion`, `TransporteSincronizacion` y `RepositorioLibroSincronizable`.
- **Exportar libro (`exportar_libro.rs`):**
  - Lee todos los eventos del libro sin mutarlo, valida el documento, serializa a texto estructurado y, si se provee frase de paso confirmada (`>= 12` caracteres), cifra el contenido antes de devolver `CopiaExportada`.
- **Importar copia (`importar_copia.rs`):**
  - Detecta si la copia está cifrada, exige y valida la frase de paso si corresponde, descifra, deserializa y valida todo el documento antes de ejecutar la fusión atómica en el repositorio.
- **Sincronizar entre dispositivos (`sincronizar_libro.rs`, `sincronizacion.rs`):**
  - Orquesta los 4 pasos de sincronización:
    1. **Traer:** lee del destino los lotes aún no confirmados por este dispositivo.
    2. **Fusionar:** descifra, valida y aplica atómicamente los eventos recibidos al libro local.
    3. **Enviar:** empaqueta y cifra en un lote nuevo únicamente los eventos locales que el destino todavía no conoce.
    4. **Publicar:** publica los cambios en el destino (si ocurre un conflicto remoto en Git, descarta la publicación pendiente, reintenta desde el paso 1 y confirma lectura de los lotes aplicados).

### 3.3. Capa de Infraestructura e IPC Tauri (`front_end/src-tauri/src/infraestructura/`, `comandos_sincronizacion.rs`, `lib.rs`)

- **Serializador de texto (`serializador_libro.rs`):** `SerializadorTextoLibro` convierte el libro en texto UTF-8 legible (encabezado de 4 líneas + un objeto JSON `EventoLibro` por línea) y valida versión, formato y conteo antes de aplicar cambios.
- **Cifrador autenticado (`cifrador_libro.rs`):** `CifradorArgon2XChaCha20` deriva claves de 256 bits con `Argon2id` (`m = 19456 KiB`, `t = 2`, `p = 1`, sal aleatoria de 16 bytes) y cifra con `XChaCha20-Poly1305` (nonce de 24 bytes), autenticando los 68 bytes del encabezado como AAD y limpiando la clave derivada en memoria con `zeroize`.
- **Almacén de secretos (`almacen_secretos_keyring.rs`):** `AlmacenSecretosKeyring` gestiona la frase de paso y credenciales de Git mediante el crate `keyring` (Administrador de Credenciales de Windows / macOS Keychain / Secret Service en Linux).
- **Adaptadores de transporte:**
  - `transporte_carpeta.rs` (`TransporteCarpeta`): organiza lotes inmutables `.ccbatch` por subcarpeta `<dispositivo_id>/`, escribe atómicamente mediante archivo temporal + `rename` y registra confirmación de lectura en `.leidos/`.
  - `transporte_git.rs` (`TransporteGit`): clona, realiza `fetch`/`pull`, escribe lotes en `<dispositivo_id>/`, ejecuta `commit` y `push` usando credenciales desde el keyring (sin incrustar usuario ni token en la URL) y mapea errores de red, autenticación y conflictos remotos.
  - `transporte_memoria.rs` (`TransporteMemoria`): adaptador en memoria para pruebas de contrato.
- **Persistencia SQLite (`sqlite_repo.rs`):**
  - Agrega las tablas `local_e8_configuracion` (`dispositivo_id`, tipo/ubicación del destino, fecha de última sincronización), `local_e8_eventos_remotos` y `local_e8_lotes_sincronizacion`, además de implementar `RepositorioLibroSincronizable` con fusión atómica en transacción SQLite.
- **Comandos Tauri (`comandos_sincronizacion.rs`, `lib.rs`):**
  - Expone `exportar_copia_seguridad` (con escritura atómica en disco), `importar_copia_seguridad`, `obtener_configuracion_sincronizacion`, `configurar_destino_sincronizacion` (validando carpetas existentes y URLs Git HTTPS sin credenciales incrustadas), `guardar_frase_paso_sincronizacion`, `actualizar_credenciales_git` y `sincronizar_libro_personal`.

### 3.4. Interfaz de Usuario (`front_end/src/`)

- **Servicios (`services/respaldoService.ts`) y Vista (`components/VistaRespaldo.tsx`):**
  - Flujo de copias de seguridad para exportar (cifrado con doble confirmación y advertencia de no recuperación si se olvida la frase, o sin cifrar con advertencia de seguridad) e importar con informe detallado de eventos nuevos, existentes y rechazados.
  - Flujo de sincronización entre dispositivos para configurar repositorio Git o carpeta compartida, actualizar credenciales si fallan, ejecutar sincronización automática al iniciar la app (cuando existe destino configurado), disparar sincronización manual con `"Sincronizar ahora"` y refrescar el libro local tras cada operación.
  - Integrado con `Navbar.tsx` y `page.tsx`.

## 4. Formato de exportación y protocolo de respaldo

La especificación abierta línea por línea del formato sin cifrar (`CUENTAS_CLARAS_LIBRO v1`) y de la estructura binaria de 68 bytes del encabezado cifrado (`CC_CIFRADO_V1`) se encuentra documentada en [`docs/formato-exportacion.md`](formato-exportacion.md).

## 5. Estrategia de pruebas

1. **Pruebas unitarias de infraestructura (`front_end/src-tauri/src/infraestructura/`):**
   - `serializador_libro.rs`: *round-trip* de las 9 variantes de `EventoLibro`, libro vacío, rechazo por versión no soportada, falta de separador, conteo inconsistente o línea corrupta.
   - `cifrador_libro.rs`: *round-trip* cifrado/descifrado, rechazo ante frase incorrecta, detección de alteración byte a byte (tanto en cuerpo como en encabezado AAD), verificación de sal y nonce distintos por operación y ausencia de datos o frase en claro.
   - `almacen_secretos_keyring.rs`: validaciones de rechazo ante frase de paso o credenciales vacías.
   - `transporte_carpeta.rs`, `transporte_git.rs` y `sqlite_repo.rs`: escritura atómica, flujo de lotes por dispositivo, convergencia y persistencia transaccional.
   - `comandos_sincronizacion.rs`: rechazo de credenciales incrustadas en URLs Git.
2. **Pruebas de aplicación y contrato (`front_end/src-tauri/src/aplicacion/pruebas/sincronizacion.rs`):**
   - Ejecución con `cargo test --manifest-path src-tauri/Cargo.toml` verificando exportación, importación idempotente, rechazo sin modificar el libro ante fallos y convergencia entre dispositivos sobre el transporte de sincronización.
