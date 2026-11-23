# Aplicación Cliente — Cuentas Claras

Módulo cliente para la aplicación de finanzas personales y gestión de gastos compartidos **Cuentas Claras**, desarrollado con **Tauri 2 (Rust)** y **Next.js (React 19 / TypeScript)**.

---

## Arquitectura del Cliente

* **Interfaz de Usuario (Frontend):** Construida con Next.js utilizando exportación estática (`output: 'export'`).
* **Núcleo Nativo (Desktop Core):** Empaquetado de escritorio mediante Tauri 2 en Rust (`src-tauri/`).
* **Comunicación IPC y HTTP:** La interfaz delega todas las operaciones del libro personal al núcleo local en Rust mediante comandos IPC (`invoke(...)`) sobre SQLite local, y coordina las funciones sociales y la sincronización de gastos compartidos (Entregas 5, 6 y 7) con el servidor FastAPI, manteniendo un desacoplamiento total entre las finanzas personales del dispositivo y la cuenta social del usuario (`INV-16` / `RES-02`).

### Comandos locales de cuentas y movimientos (Entregas 2 y 3)

El núcleo Tauri expone los comandos `crear_cuenta`, `listar_cuentas`, `crear_categoria`, `listar_categorias`, `registrar_movimiento`, `corregir_movimiento` y `obtener_detalle_cuenta`. Todos operan sobre el libro SQLite local (`cuentas_claras_libro.db`): los movimientos se registran como eventos inmutables y las correcciones se agregan como movimientos de tipo `ajuste` que conservan la referencia al movimiento original, sin borrar ni sobrescribir filas previas.

### Comandos locales de monedas, tasas y patrimonio (Entrega 4)

El núcleo Tauri expone los comandos `listar_monedas`, `crear_moneda`, `obtener_moneda_base`, `cambiar_moneda_base`, `registrar_tasa`, `eliminar_tasa`, `listar_historial_tasas` y `consultar_patrimonio`. Las monedas propias, el cambio de moneda base y las tasas se guardan como eventos inmutables (`MonedaCreada`, `MonedaBaseCambiada`, `TasaRegistrada`, `TasaEliminada`). La tasa inversa (`1 / valor`) se calcula al consultar sin triangulación entre monedas.

* Decisiones de diseño: `docs/adr/ADR-005-tasas-de-cambio-inversa-y-tasa-faltante.md`.
* Detalle técnico: `docs/Documento_Tecnico_Explicativo_Entrega_4.md`.

### Autenticación, sesión y finanzas colaborativas (Entregas 5 y 6)

* **Núcleo de Autenticación (`src-tauri/src/autenticacion.rs` y `src/services/authService.ts`):**
  * Expone los comandos `registrar_usuario`, `iniciar_sesion`, `obtener_perfil`, `cerrar_sesion` y `verificar_estado_servidor`.
  * En modo escritorio Tauri, el token de sesión se resguarda en el almacén seguro del sistema operativo (`keyring`: Windows Credential Manager / macOS Keychain / Secret Service) y se limpia al cerrar sesión o detectar vencimiento (`401`).
  * **Soberanía del dispositivo (`INV-16`):** Las cuentas personales, movimientos, categorías, monedas propias y tasas de cambio residen exclusivamente en el dispositivo local y son independientes de la sesión de usuario. Pueden crearse y consultarse sin iniciar sesión y sin conexión al servidor; al iniciar sesión con cualquier cuenta social, los datos locales del dispositivo permanecen intactos.
* **Módulo Social y Gastos Compartidos (`src/services/socialService.ts` y `src/components/VistaComunidad.tsx`):**
  * **Amigos y Solicitudes:** Búsqueda exacta por `@nombre_usuario` (`RF-SOC-04`), tarjeta y pestaña dedicada de **Solicitudes** (recibidas con `Aceptar` / `Rechazar` y enviadas pendientes) y eliminación de amistades con confirmación integrada en página.
  * **Grupos Colaborativos:** Creación de grupos seleccionando amigos de la lista propia mediante **búsqueda parcial en tiempo real** por nombre o `@usuario`, e invitación/aceptación de miembros.
  * **Gastos Compartidos y Balances:** Registro de gastos con división en **Partes iguales (`IGUALES`)**, **Porcentajes (`PORCENTAJES`)** o **Montos exactos (`MONTOS`)**, visualización del desglose por participante y consulta de balances ordenados alfabéticamente (`A favor`, `Por pagar`, `Al día`).

* Decisiones de diseño: `docs/adr/ADR-006-postgresql-y-persistencia-social.md`, `ADR-007-autenticacion-hashing-y-sesiones.md` y `ADR-008-reparto-determinista-y-balances.md`.
* Detalle técnico: `docs/Documento_Tecnico_Explicativo_Entrega_5_y_6.md`.

### Sincronización de gastos de grupo y cola sin conexión (Entrega 7)

* **Servicio y Núcleo de Sincronización (`src/services/syncService.ts` y `src-tauri/src/`):**
  * **Cuenta asociada por grupo (`INV-18`):** En la barra de pestañas del grupo, el selector **"Cuenta asociada (`<Moneda>`)"** permite vincular una cuenta personal activa en la misma moneda del grupo. La elección se guarda únicamente en el dispositivo (`INV-16`).
  * **Sincronización e idempotencia (`RF-COM-10`, `INV-13`, `INV-15`):** Al abrir **Grupos y Amigos** o pulsar **"Sincronizar"** (o el botón **`↺`** para volver a verificar todo el historial desde el inicio), la aplicación consulta `GET /cambios?desde=<cursor>` y aplica cada parte como un único movimiento de gasto (`"<Grupo> · <Concepto>"`, marcado como **`Gasto de grupo`** y **`Se corrige en el grupo`**). Si el grupo aún no tiene cuenta asociada, la parte queda en **"Gastos de grupo pendientes de asignar"** y se aplica en cuanto se elige la cuenta.
  * **Cola automática sin conexión (`RNF-17`, `ESC-09`):** Si se registra un gasto de grupo cuando no hay conexión con el servidor, la operación se guarda automáticamente en la cola local con estado **`Pendiente de envío`** y su `Idempotency-Key` única, y se despacha en orden al restablecerse la conexión.

* Decisiones de diseño: `docs/adr/ADR-009-sincronizacion-cursor-e-idempotencia.md`.
* Detalle técnico: `docs/Documento_Tecnico_Explicativo_Entrega_7.md`.

---

## Requisitos Previos

* **Node.js** (versión 20 o superior) y `pnpm` / `npm`.
* **Rust y Cargo** (versión 1.77 o superior) instalados mediante `rustup`.
* Dependencias del sistema requeridas para Tauri según el sistema operativo (en Windows: Microsoft Visual Studio C++ Build Tools y WebView2).

---

## Instalación y Ejecución

Ejecutar desde este directorio (`front_end/`):

```powershell
# 1. Instalar dependencias
pnpm install

# 2. Iniciar aplicación nativa de escritorio con Tauri (puerto 3000 + ventana nativa)
pnpm tauri dev

# Alternativa: Iniciar únicamente la interfaz web en el navegador (http://localhost:3000)
pnpm dev
```

### Compilación para Producción

```powershell
# Generar export estático del frontend web (en out/)
pnpm build

# Compilar binario instalable de escritorio Tauri
pnpm tauri build
```

---

## Almacenamiento Local (SQLite)

Los datos financieros personales residen exclusivamente en el dispositivo dentro del archivo SQLite embebido `cuentas_claras_libro.db`:

* **Windows:** `%APPDATA%\com.davidapm.cuentas-claras\cuentas_claras_libro.db`  
  *(Ruta completa: `C:\Users\<Usuario>\AppData\Roaming\com.davidapm.cuentas-claras\cuentas_claras_libro.db`)*
* **Linux:** `~/.local/share/com.davidapm.cuentas-claras/cuentas_claras_libro.db`

---

## Ejecución de Pruebas Automatizadas

```powershell
# Ejecutar todas las pruebas en Rust (dominio, casos de uso, contrato e inmutabilidad SQLite)
cargo test --manifest-path src-tauri/Cargo.toml

# Ejecutar solo las pruebas de la Entrega 4 en Rust
cargo test --manifest-path src-tauri/Cargo.toml pruebas_entrega4

# Ejecutar las pruebas automatizadas del frontend (TypeScript)
pnpm test

# Verificar tipado estático de TypeScript
npx tsc --noEmit
```

---

## Reproducir el Ejemplo de la Sección 2.2 (Corrección Inmutable de un Movimiento)

1. Ejecutar `pnpm tauri dev` y crear una cuenta llamada `"Billetera dólares"` en `USD` con saldo inicial de `1.000,00`.
2. Registrar un **ingreso** de `800,00` en la categoría `"Salario"` → El saldo queda en **`1.800,00`**.
3. Registrar un **gasto** de `500,00` en `"Alimentación"` (simulando un error de digitación: el monto real era `50,00`) → El saldo queda en **`1.300,00`**.
4. Sobre ese movimiento, pulsar **"Corregir"** e indicar el monto correcto: `50,00` → Se genera un ajuste de `+450,00` y el saldo final queda en **`1.750,00`**.
5. Verificar en el historial que el movimiento original (`500,00`) sigue visible como `"corregido"` y existe el nuevo movimiento de tipo `"ajuste"`.

---

## Reproducir el Escenario de la Sección 2.3 (Tasas de Cambio y Patrimonio Consolidado)

1. En **Patrimonio**, confirmar que la moneda base es `USD`.
2. Crear una cuenta `"Banesco"` en `VES` con saldo `1.000,00` y una cuenta `"Efectivo"` en `USD` con saldo `50,00`.
3. Consultar el patrimonio sin tasas registradas → Se muestran los subtotales por moneda pero **no** el total consolidado, indicando que falta la tasa `VES -> USD`.
4. Registrar la tasa `VES -> USD = 0,025` (`t1`) → El total consolidado pasa a **`75,00 USD`**.
5. Registrar una tasa más reciente `VES -> USD = 0,020` (`t2`) → El total consolidado se actualiza a **`70,00 USD`**.
6. Eliminar la tasa `0,020` en **Tasas de Cambio** → Vuelve a aplicarse `0,025` (**`75,00 USD`**).
7. Cambiar la moneda base a `VES` → Se aplica automáticamente la tasa inversa (`1 / 0,025 = 40`) dando **`3.000,00 VES`**.

---

## Reproducir el Escenario de las Entregas 5 y 6 (Autenticación, Amigos, Grupos y Gastos Compartidos)

### Parte 1: Autonomía sin sesión y datos locales del dispositivo (`INV-16`)
1. Sin iniciar sesión (incluso con el servidor apagado), crear cuentas personales, registrar movimientos y agregar tasas de cambio en **Mis Cuentas** y **Tasas de Cambio**.
2. Encender PostgreSQL (`docker compose up -d`) y el servidor FastAPI (`python -m uvicorn main:app --reload --port 8000`).
3. Verificar que al iniciar sesión con distintos usuarios o al cerrar sesión, el libro personal (**Mis Cuentas**, **Patrimonio** y **Tasas de Cambio**) conserva siempre los datos locales del dispositivo.

### Parte 2: Registro, sesión y persistencia en Docker (Entrega 5)
1. Abrir el modal de **Iniciar sesión / Registrarse** (con el botón de ícono de ojo para mostrar/ocultar la contraseña mientras se escribe) y registrar tres usuarios:
   * `@ana` (`Ana Pérez`, `ana@correo.com`)
   * `@beto` (`Beto Gómez`, `beto@correo.com`)
   * `@carla` (`Carla Díaz`, `carla@correo.com`)
2. Verificar que tras recargar con `F5` la sesión permanece abierta y que tras ejecutar `docker compose restart` los usuarios siguen persistidos en PostgreSQL.

### Parte 3: Solicitudes de amistad, grupos y división de gastos (Entrega 6)
1. Con `@ana`, entrar a **Grupos y Amigos → Amigos**, buscar exactamente a `beto` y `carla` y enviarles invitación de amistad.
2. Iniciar sesión con `@beto` y `@carla`, ir a la tarjeta/pestaña **Solicitudes** y **Aceptar** la solicitud de Ana.
3. Con `@ana`, ir a **Mis Grupos → + Crear Grupo**:
   * Nombrar el grupo `"Viaje a Mérida"` en moneda `USD`.
   * Usar la **barra de búsqueda parcial de amigos** (ej. escribir `"be"` o `"ca"`) para seleccionar a `@beto` y `@carla` de la lista de amigos e invitarlos automáticamente al guardar el grupo.
4. Con `@beto` y `@carla`, aceptar la invitación al grupo en **Solicitudes** o **Mis Grupos**.
5. Registrar un gasto compartido de **`100,00 USD`** pagado por **Ana** en **Partes iguales** entre Ana, Beto y Carla:
   * Verificar que el desglose asigna **`$ 33,34`** a `@ana` (primera en orden alfabético por `ESC-10`), **`$ 33,33`** a `@beto` y **`$ 33,33`** a `@carla` (`suma = 100,00 USD`).
   * En la pestaña **Balances**, comprobar que aparecen ordenados alfabéticamente (`@ana`: `+$ 66,66` A favor, `@beto`: `-$ 33,33` Por pagar, `@carla`: `-$ 33,33` Por pagar; suma algebraica = `0,00`).
6. Verificar finalmente que en **Mis Cuentas** los saldos personales del dispositivo no fueron alterados por los gastos compartidos del grupo (`INV-16`).

---

## Reproducir el Escenario de la Entrega 7 (Traer Cuentas del Grupo al Registro Personal — The Sync Flow)

Partiendo de los usuarios `@ana`, `@beto` y `@carla` y el grupo **"Viaje a Mérida"** (`USD`) de la Entrega 6, con una cuenta personal **"Efectivo USD"** con saldo inicial de **`200,00 USD`**:

1. **Cuenta del Grupo (`INV-18`):**
   * Iniciar sesión con `@beto`, abrir el grupo **"Viaje a Mérida"** (`USD`) y en el selector **"Cuenta asociada (USD):"** (ubicado junto a las pestañas *Gastos / Balances / Miembros*) elegir la cuenta **`Efectivo USD`** (solo se ofrecen cuentas activas en `USD`).
   * Con `@carla`, dejar el selector en **`Sin cuenta asignada`**.
2. **Bajada de Partes y Pendientes de Asignar (`RF-COM-10`, `2.2`):**
   * Con `@ana`, registrar el gasto **"Hotel"** por **`100,00 USD`** en partes iguales (`33,34` Ana / `33,33` Beto / `33,33` Carla).
   * Con `@beto`, pulsar **"Sincronizar"**: en su cuenta **`Efectivo USD`** aparece un gasto de **`33,33 USD`** con concepto `"Viaje a Mérida · Hotel"`, marcado como **`Gasto de grupo`** y con la indicación **`Se corrige en el grupo`** (sin opción de corregir en el libro personal). El saldo de **`Efectivo USD`** pasa a **`166,67 USD`**.
   * Con `@carla`, pulsar **"Sincronizar"**: como aún no eligió cuenta para el grupo, su parte de **`33,33 USD`** aparece en el aviso **"Gastos de grupo pendientes de asignar"** y se aplica automáticamente en cuanto selecciona su cuenta en `USD`.
3. **Sin Duplicar y Reverificación desde el Inicio (`INV-13`, `INV-15`):**
   * Con `@beto`, pulsar **"Sincronizar"** tres veces seguidas y luego pulsar el botón **`↺`** (*Volver a verificar todo el historial de gastos desde el inicio*, que reinicia el cursor local a `0`): vuelven a descargarse los eventos desde `GET /cambios?desde=0`, pero el núcleo comprueba que ya fueron aplicados (`GastoDeGrupoAplicado`), sigue habiendo **un solo movimiento** y el saldo se mantiene en **`166,67 USD`**.
4. **Cola de Gastos de Grupo sin Conexión (`RNF-17`, `ESC-09`):**
   * Con `@beto`, desconectar la red (por ejemplo, activando `Offline` en la pestaña *Network* de las herramientas de desarrollador o deteniendo temporalmente el servidor) y registrar el gasto **"Cena"** por **`60,00 USD`** por porcentajes (`50%` Ana / `25%` Beto / `25%` Carla).
   * La aplicación detecta automáticamente la falta de conexión y guarda el gasto en la cola local, mostrándolo en el grupo como **`Pendiente de envío`** mientras el saldo de **`Efectivo USD`** permanece en **`166,67 USD`** y las funciones personales siguen disponibles.
   * Al restablecer la conexión (o pulsar **"Enviar ahora"** / **"Sincronizar"**), la cola envía la cena **una sola vez** con su `Idempotency-Key`, aparece en el grupo y baja la parte de `@beto` (**`15,00 USD`**), actualizando el saldo de **`Efectivo USD`** a **`151,67 USD`**.
   * Si se pulsa nuevamente el botón **`↺`**, bajan otra vez ambos eventos (`Hotel` y `Cena`), se ignoran por estar ya aplicados y el saldo permanece en **`151,67 USD`**.
5. **Privacidad (`INV-16`):**
   * En el registro de peticiones del servidor se comprueba que ninguna petición contiene cuentas personales, movimientos privados, saldos ni tasas del libro local.
