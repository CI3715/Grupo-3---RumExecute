# Documento Técnico Explicativo — Entrega 3 (Sprint 3)
## Sistema de Gestión Financiera Personal "Cuentas Claras"
**Materia:** CI-3715 — Ingeniería de Software (Universidad Simón Bolívar)  
**Grupo:** Grupo 3 — RumExecute  
**Rol Asignado:** Miembro 4 — Interfaz de Usuario y The Ledger Flow  
**Responsable:** Victor Manuel Hernández  
**Fecha:** Octubre 2026  

---

## 1. Resumen Ejecutivo y Alcance de la Entrega

Durante la **Entrega 3 (Sprint 3)**, el objetivo central del proyecto fue la materialización de **The Ledger Flow**: el flujo contable fundamental que permite registrar ingresos, gastos y correcciones no destructivas en cuentas financieras, proyectando saldos exactos en base a un modelo de eventos inmutables (*Event Sourcing*), en estricta conformidad con las invariantes contables del sistema.

Como responsable del **Punto 4 (Interfaz de Usuario y The Ledger Flow)**, el trabajo consistió en:
1. Diseñar, implementar y refactorizar todos los componentes visuales en **Next.js (React / TypeScript)** respetando la fidelidad visual de Figma y la paleta de diseño.
2. Optimizar radicalmente el código fuente de la interfaz, reduciendo la redundancia de casi 4.000 líneas a ~2.000 líneas limpias y mantenibles.
3. Incorporar mejoras de experiencia de usuario (UX) inspiradas en la banca nacional (estilo Banco Mercantil / Banco de Venezuela) para agilizar la entrada de datos.
4. Conectar la interfaz web de manera reactiva y segura con el núcleo de escritorio de **Tauri v2** mediante llamadas IPC (`@tauri-apps/api/core`) hacia la base de datos **SQLite inmutable**.
5. Garantizar la sincronización bidireccional de saldos entre la vista de detalle de cada cuenta y la vista general de cuentas y subtotales por moneda.

---

## 2. Requerimientos e Invariantes Implementadas

| Identificador | Descripción | Implementación en la Interfaz |
| :--- | :--- | :--- |
| **RF-LIB-01** | Creación de Cuentas y saldo inicial | Modal con selector de tipo (Efectivo, Banco, Billetera), selección de moneda base y saldo inicial formateado. |
| **RF-LIB-02** | Consulta de Cuentas y Subtotales | Grid de subtotales separados por moneda (`INV-02`) y lista de tarjetas de cuentas individuales. |
| **RF-LIB-04** | Registro de Gastos con Alerta Reactiva | Pestaña de gasto con indicador en vivo si el saldo resultante entra en sobregiro o saldo negativo. |
| **RF-LIB-05** | Registro de Ingresos | Pestaña de ingreso con actualización inmediata de la composición del balance. |
| **RF-LIB-06** | Consulta del Detalle de Cuenta | Vista cronológica de movimientos con la ecuación contable: `Inicial + Ingresos − Gastos ± Ajustes = Actual`. |
| **RF-LIB-07** | Catálogo Oficial de Categorías | Selector con las 7 categorías oficiales de gasto y 4 de ingreso, con soporte de categorías personalizadas e iconos. |
| **INV-01 / INV-02** | Par Indivisible y Separación Monetaria | Jamás se suman montos de monedas distintas; los subtotales se calculan y agrupan estrictamente por divisa (VES, USD, EUR, USDT). |
| **INV-03** | Aritmética Decimal Exacta | Validación estricta a 2 decimales para evitar desbordamientos o imprecisiones de coma flotante IEEE-754. |
| **INV-04** | Inmutabilidad y Corrección por Ajuste | **Eliminación total de botones de editar o borrar.** Los errores se corrigen emitiendo un movimiento de tipo **Ajuste** compensatorio. |
| **INV-05** | Proyección Exacta de Saldos | El saldo actual siempre es la suma algebraica exacta del saldo inicial y la secuencia de movimientos/ajustes. |
| **INV-13** | Idempotencia y Consistencia Local | La interfaz sincroniza su estado directamente con los eventos persistidos en SQLite local. |

---

## 3. Decisiones de Arquitectura y Optimización de Código

### 3.1. Reducción de Complejidad y Centralización de Estilos
En versiones preliminares, cada componente de React contenía objetos gigantescos de estilos en línea (`const styles = { ... }`) que alcanzaban entre 250 y 350 líneas por archivo, entorpeciendo la legibilidad y dificultando la depuración.

* **Solución aplicada:** Se migró toda la capa de diseño a clases semánticas dentro de `front_end/src/app/globals.css`.
* **Resultado:**
  * Reducción de más del **50% de líneas de código** por componente.
  * Eliminación de bugs de consola en React/Turbopack causados por el solapamiento de propiedades CSS (`border` vs `borderColor`).
  * Mantenimiento de la estética corporativa y moderna de Figma al 100%.

---

## 4. Mejoras de Usabilidad y Experiencia de Usuario (UX)

### 4.1. Entrada de Montos Inteligente (Estilo Banco Mercantil / BDV)
* **El problema:** Prellenar los campos con `"0,00"` obligaba al usuario a presionar la tecla de retroceso (`Backspace`) para borrar los ceros antes de escribir su saldo o monto.
* **La solución:**
  1. Los inputs inician vacíos con `placeholder="0,00"`.
  2. Al enfocar el input (`onFocus`), se auto-selecciona el texto para permitir sobreescritura inmediata con un solo toque.
  3. Al escribir, la función `normalizarMontoInput` admite comas y puntos de forma tolerante.
  4. Al salir del campo (`onBlur`), la función `completarDecimalesOnBlur` añade automáticamente los decimales correspondientes (ej. si el usuario escribe `50`, se formatea a `50,00`).

### 4.2. Corrección del Bug de Selección de Texto en Modales
* **El problema:** Cuando el usuario hacía clic sostenido para subrayar un texto dentro de un modal y soltaba el cursor fuera del cuadro, el evento `onClick` del overlay cerraba la ventana inesperadamente, perdiendo los datos escritos.
* **La solución:** Se reemplazó el listener `onClick` en el overlay por:
  ```tsx
  onMouseDown={(e) => e.target === e.currentTarget && onClose()}
  ```
  Esto garantiza que solo un clic originado y finalizado en el fondo gris cierre la ventana.

### 4.3. Eliminación de Jerga Técnica para el Cliente
* Se eliminaron etiquetas internas de desarrollo como *"Invariante INV-04"*, *"Event Sourcing"* o *"The Ledger Flow"*.
* En su lugar, el cliente observa términos familiares y profesionales:
  * **"Movimientos"** en lugar de "Libro de Eventos".
  * **"Ajuste Contable"** en lugar de "Evento Compensatorio".
  * **"🔒 100% Local y Privado"** en lugar de "Aislamiento de Persistencia".

---

## 5. El Flujo de Corrección No Destructiva (`INV-04`)

El sistema implementa el principio contable de inmutabilidad: **los datos históricos nunca se borran ni se editan**.

En la interfaz:
1. En la tabla de movimientos, cada fila cuenta con un botón **"Corregir"** (únicamente disponible para movimientos vigentes, nunca para ajustes ni movimientos ya corregidos).
2. El modal `ModalCorregirMovimiento` calcula en tiempo real la diferencia algebraica según el tipo de movimiento:
   * **Gasto anotado de más:** genera un ajuste positivo (`+`).
   * **Gasto anotado de menos:** genera un ajuste negativo (`−`).
   * **Ingreso anotado de más:** genera un ajuste negativo (`−`).
   * **Ingreso anotado de menos:** genera un ajuste positivo (`+`).
3. El usuario puede ver la previsualización del ajuste resultante antes de confirmar.
4. Al confirmar, el movimiento original se tacha visualmente con el badge `⚪ Corregido`, conservando su registro en la base de datos, y se añade una nueva fila con el badge `🔵 Ajuste` referenciando el identificador del movimiento original.

---

## 6. Conexión IPC con Tauri v2 y Sincronización de Estado

### 6.1. Contrato de Invocaciones IPC
La capa web interactúa con el núcleo de Rust a través de los siguientes comandos registrados:

| Comando Tauri | Parámetros Enviados desde JS | Retorno de Rust | Propósito |
| :--- | :--- | :--- | :--- |
| `listar_cuentas` | Ninguno | `ResumenCuentas` | Obtiene la lista base de cuentas registradas. |
| `crear_cuenta` | `{ entrada: { nombre, tipo, moneda, saldo_inicial } }` | `Cuenta` | Emite el evento `CuentaCreada`. |
| `obtener_detalle_cuenta` | `{ idCuenta: string }` | `DetalleCuenta` | Devuelve saldo actual, acumulados y lista de movimientos de SQLite. |
| `registrar_movimiento` | `{ entrada: { id_cuenta, tipo, monto, categoria, fecha, descripcion } }` | `Movimiento` | Emite el evento `MovimientoRegistrado`. |
| `corregir_movimiento` | `{ entrada: { id_movimiento_original, nuevo_monto, fecha, descripcion } }` | `Movimiento` | Emite el evento `MovimientoCorregido` (Ajuste). |
| `ping_servidor` | Ninguno | `PingResult` | The Ping Flow: mide latencia contra el servidor FastAPI. |

### 6.2. Sincronización Global de Saldos (Vista Interna vs Vista Externa)
Uno de los retos técnicos resueltos fue garantizar que, cuando un usuario registre o corrija un movimiento dentro de una cuenta, las tarjetas de la pantalla principal ("Mis Cuentas") y los subtotales superiores se actualicen en tiempo real sin requerir recargar la aplicación:

* **Mecanismo:** La función `cargarCuentasTauri()` en `page.tsx` ejecuta en paralelo (`Promise.all`) llamadas a `obtener_detalle_cuenta` para cada cuenta registrada, extrayendo el `saldo_actual` proyectado por SQLite.
* **Garantía:** Tanto al volver de la vista de detalle como al guardar un movimiento, se dispara la sincronización, asegurando que el saldo mostrado en las tarjetas externas coincida exactamente con el saldo de la ecuación interna.

---

## 7. Incidencias Técnicas Resueltas durante la Integración

### 7.1. Deserialización del Struct `Categoria` en React
* **Síntoma:** Error de renderizado en runtime:
  `Objects are not valid as a React child (found: object with keys {id, nombre, tipo}).`
* **Causa:** En Rust, `m.categoria` se definió como una entidad completa `Categoria { id, nombre, tipo }` en lugar de una cadena simple. Al devolver el JSON por IPC, React intentaba pintar el objeto directamente dentro del árbol de componentes.
* **Solución:** Se implementó una capa de sanitización estricta tanto en `page.tsx` como en `DetalleCuenta.tsx`, extrayendo siempre el nombre textual (`m.categoria.nombre || m.categoria.id || "Varios"`) y asegurando la conversión a tipos primitivos con `String(...)`.

### 7.2. Limpieza de Caché y Memoria de Turbopack
* **Síntoma:** Error de variables duplicadas `the name cuentaActivaDetalle is defined multiple times`.
* **Causa:** Durante una fusión en GitHub, se había pegado un bloque duplicado al final del archivo `page.tsx`. Asimismo, Turbopack en modo de recarga en caliente (`stale`) retenía la memoria del estado anterior.
* **Solución:** Se eliminó quirúrgicamente el bloque redundante en `page.tsx`, se purgó la carpeta temporal `front_end/.next` y la base de datos previa en `%APPDATA%`, logrando un arranque 100% limpio desde cero.

---

## 8. Verificación y Resultados de Pruebas

Toda la implementación fue verificada mediante un conjunto riguroso de pruebas automáticas y manuales:

1. **Compilación de TypeScript:**
   ```bash
   npx tsc --noEmit
   # Resultado: 0 errores de tipado en todo el proyecto.
   ```
2. **Empaquetado de Producción:**
   ```bash
   pnpm build
   # Resultado: ✓ Compiled successfully en 6.3s (3 páginas estáticas generadas sin advertencias).
   ```
3. **Suite de Pruebas de Rust y Persistencia:**
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   # Resultado: test result: ok. 89 passed; 0 failed; 0 ignored; finished in 0.19s
   ```
4. **Verificación de Inmutabilidad SQLite:**
   * Pruebas automáticas ejecutan sentencias `UPDATE` y `DELETE` directas sobre la tabla `eventos_libro`.
   * Los triggers `bloquear_modificacion_eventos_libro` y `bloquear_eliminacion_eventos_libro` abortan la transacción (`RAISE(ABORT)`), demostrando matemáticamente la inmutabilidad física del libro contable.
5. **Autonomía Offline:**
   * La aplicación fue probada sin conexión de red y con el servidor apagado: la creación de cuentas, registro de movimientos y consulta de saldos operan con 100% de autonomía en el dispositivo local.

---

## 9. Conclusión

El **Punto 4 de la Entrega 3** se encuentra completado, probado y totalmente integrado en la rama `main` del repositorio oficial. La solución entregada combina una arquitectura limpia y desacoplada, alta fidelidad de diseño, respeto inquebrantable a las invariantes contables y una experiencia de usuario fluida y confiable para la gestión de finanzas personales.
