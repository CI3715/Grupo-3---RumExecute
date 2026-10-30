# Aplicacion Cliente - Cuentas Claras

Modulo cliente para la aplicacion de finanzas personales y gestion de gastos compartidos Cuentas Claras, desarrollado con Tauri 2 (Rust) y Next.js (React / TypeScript).

## Arquitectura del Cliente

* **Interfaz de Usuario (Frontend):** Construida con Next.js utilizando exportacion estatica (`output: 'export'`).
* **Nucleo Nativo (Desktop Core):** Empaquetado de escritorio mediante Tauri 2 en Rust (`src-tauri/`).
* **Comunicacion IPC:** La interfaz web se comunica con el backend a traves de comandos invocados con Tauri (`invoke('ping_servidor')`), garantizando aislamiento y ejecucion nativa.

### Comandos locales de cuentas y movimientos

El nucleo Tauri expone los comandos `registrar_movimiento`, `corregir_movimiento` y `obtener_detalle_cuenta`. Los dos primeros reciben el objeto `entrada` definido por sus casos de uso; el ultimo recibe `idCuenta`. Los tres operan sobre el libro SQLite local: los movimientos se registran como eventos inmutables y las correcciones se agregan como ajustes que conservan la referencia al movimiento original.

### Comandos locales de monedas, tasas y patrimonio

El nucleo Tauri expone los comandos `listar_monedas`, `crear_moneda`, `cambiar_moneda_base`, `registrar_tasa`, `eliminar_tasa`, `listar_historial_tasas` y `consultar_patrimonio`. Todos operan sobre el libro SQLite local: las monedas propias, el cambio de moneda base y las tasas se guardan como eventos inmutables (`MonedaCreada`, `MonedaBaseCambiada`, `TasaRegistrada`, `TasaEliminada`). Eliminar una tasa agrega un evento `TasaEliminada`; nunca se borra ni se edita una fila existente. La tasa inversa (`1 / valor`) se calcula al consultar, sin triangulacion entre monedas.

Decisiones de diseno: `docs/adr/ADR-005-tasas-de-cambio-inversa-y-tasa-faltante.md`. Detalle tecnico: `docs/Documento_Tecnico_Explicativo_Entrega_4.md`.

## Requisitos Previos

* Node.js (version 20 o superior) y npm / pnpm.
* Rust y Cargo (version 1.77 o superior) instalados mediante rustup.
* Dependencias del sistema requeridas para Tauri segun el sistema operativo (en Windows: Microsoft Visual Studio C++ Build Tools y WebView2).

## Instalacion de Dependencias

Ejecutar en este directorio (`front_end/`):

```bash
pnpm install
```

## Ejecucion en Modo Desarrollo

Para iniciar la aplicacion de escritorio completa con recarga rapida (hot-reload):

```bash
pnpm tauri dev
```

Este comando inicia el servidor de desarrollo de Next.js en el puerto 3000 y abre la ventana nativa de Tauri.

### Ejecucion Solo Frontend (Navegador)

Para trabajar unicamente en la interfaz web sin compilar el binario de Rust:

```bash
pnpm dev
```

La interfaz estara accesible en `http://localhost:3000`. Nota: Las llamadas que requieren IPC nativo de Tauri cuentan con un mecanismo de respaldo por HTTP directo para facilitar pruebas visuales en navegador.

## Compilacion para Produccion

Para generar el export estatico del frontend web:

```bash
pnpm build
```

Los artefactos estaticos se generaran en el directorio `out/`.

Para compilar el binario instalable de escritorio:

```bash
pnpm tauri build
```

## Almacenamiento Local (SQLite)

Los datos financieros residen exclusivamente en el dispositivo dentro de un archivo SQLite embebido denominado `cuentas_claras_libro.db`.

### Ubicación del archivo por Sistema Operativo

* **Windows:** `%APPDATA%\com.davidapm.cuentas-claras\cuentas_claras_libro.db`  
  *(Ruta absoluta: `C:\Users\<Usuario>\AppData\Roaming\com.davidapm.cuentas-claras\cuentas_claras_libro.db`)*
* **Linux:** `~/.local/share/com.davidapm.cuentas-claras/cuentas_claras_libro.db`

---

## Ejecución de Pruebas Automatizadas

Para ejecutar la suite completa de pruebas unitarias de dominio, casos de uso con repositorio en memoria, fitness functions y pruebas de contrato:

```bash
# Ejecutar todas las pruebas en Rust
cargo test --manifest-path src-tauri/Cargo.toml

# Ejecutar solo las pruebas de la Entrega 4 (aplicacion y SQLite)
cargo test --manifest-path src-tauri/Cargo.toml pruebas_entrega4

# Ejecutar las pruebas del frontend (tasas y patrimonio)
pnpm test
```

## Reproducir el Ejemplo de la Sección 2.2 (Corrección de un Movimiento)

Este ejemplo demuestra el flujo de inmutabilidad: un gasto anotado incorrectamente
se corrige mediante un ajuste, sin borrar ni editar el original.

1. Ejecutar `pnpm tauri dev` y crear (o abrir) una cuenta llamada "Billetera dólares"
   en USD con saldo inicial de 1.000,00.
2. Registrar un **ingreso** de 800,00 en la categoría "Salario".
   → El saldo debe quedar en 1.800,00.
3. Registrar un **gasto** de 500,00 en la categoría "Alimentación"
   (simulando el error: el monto real era 50,00).
   → El saldo debe quedar en 1.300,00.
4. Sobre ese movimiento, seleccionar "Corregir" e indicar el monto correcto: 50,00.
   → Se genera un ajuste de +450,00. El saldo final debe quedar en **1.750,00**.
5. Verificar en el historial de la cuenta que:
   - El movimiento original (500,00) sigue visible, marcado como "corregido".
   - Existe un ajuste nuevo que referencia al movimiento original.

### Verificación de Autonomía (sin conexión)

1. Apagar el servidor backend del proyecto (si está corriendo).
2. Desactivar la conexión de red del equipo (WiFi/Ethernet).
3. Cerrar la aplicación por completo y volver a abrirla (`pnpm tauri dev` o el binario compilado).
4. Confirmar que:
   - La cuenta, sus movimientos y el saldo de 1.750,00 son exactamente los mismos.
   - Ninguna operación de registro, corrección o consulta de saldo requiere red
     ni el servidor backend.

## Reproducir el Escenario de la Sección 2.3 (Tasas de Cambio y Patrimonio Consolidado)

Este escenario demuestra la deteccion de tasa faltante, la seleccion de tasa por fecha y la tasa inversa, sin inventar nunca una tasa.

1. Ejecutar `pnpm tauri dev`. En la pantalla de Patrimonio, confirmar que la moneda base es USD.
2. Crear una cuenta "Banesco" en VES con saldo inicial de 1.000,00 y una cuenta "Efectivo" en USD con saldo inicial de 50,00.
3. Consultar el patrimonio a una fecha t1 (por ejemplo, 2026-10-20) sin ninguna tasa registrada.
   → Se muestran las cuentas y los subtotales por moneda, pero **no** hay total consolidado. Aparece una alerta: falta la tasa VES -> USD al 2026-10-20.
4. Pulsar "Registrar Tasa Ahora" y registrar VES -> USD = 0,025 con fecha t1 (fuente: "BCV Oficial").
   → El total consolidado a t1 es **75,00 USD**.
5. Registrar una nueva tasa VES -> USD = 0,020 con una fecha t2 posterior (por ejemplo, 2026-11-10).
   → A t1 se sigue aplicando 0,025 (75,00 USD). A t2 se aplica 0,020 (70,00 USD).
6. Con solo la tasa VES -> USD registrada, cambiar la moneda base a VES.
   → Se aplica la tasa inversa (1 / 0,025 = 40) sin registrar USD -> VES. El total es 3.000,00 VES y los saldos originales de las cuentas no cambian.
7. Registrar tambien EUR -> USD, agregar una cuenta en EUR y cambiar la moneda base a EUR.
   → No se inventa la tasa VES -> EUR (no hay conversion triangular): la alerta exige registrarla.
8. Intentar registrar una tasa con fecha futura, con valor menor o igual a 0 o con monedas identicas.
   → Cada intento se rechaza con un mensaje en espanol que indica el campo que falla.
9. En el historial de tasas del par, eliminar una tasa.
   → Deja de aplicarse en los calculos, pero el evento `TasaEliminada` permanece en la base de datos.
10. Crear una moneda propia (por ejemplo, `CAD`, 2 decimales). Confirmar que VES, USD, EUR y USDT no se pueden editar ni eliminar.