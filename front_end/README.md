# Aplicacion Cliente - Cuentas Claras

Modulo cliente para la aplicacion de finanzas personales y gestion de gastos compartidos Cuentas Claras, desarrollado con Tauri 2 (Rust) y Next.js (React / TypeScript).

## Arquitectura del Cliente

* **Interfaz de Usuario (Frontend):** Construida con Next.js utilizando exportacion estatica (`output: 'export'`).
* **Nucleo Nativo (Desktop Core):** Empaquetado de escritorio mediante Tauri 2 en Rust (`src-tauri/`).
* **Comunicacion IPC:** La interfaz web se comunica con el backend a traves de comandos invocados con Tauri (`invoke('ping_servidor')`), garantizando aislamiento y ejecucion nativa.

## Requisitos Previos

* Node.js (version 20 o superior) y npm / pnpm.
* Rust y Cargo (version 1.77 o superior) instalados mediante rustup.
* Dependencias del sistema requeridas para Tauri segun el sistema operativo (en Windows: Microsoft Visual Studio C++ Build Tools y WebView2).

## Instalacion de Dependencias

Ejecutar en este directorio (`front_end/`):

```bash
npm install
```

## Ejecucion en Modo Desarrollo

Para iniciar la aplicacion de escritorio completa con recarga rapida (hot-reload):

```bash
npm run tauri dev
```

Este comando inicia el servidor de desarrollo de Next.js en el puerto 3000 y abre la ventana nativa de Tauri.

### Ejecucion Solo Frontend (Navegador)

Para trabajar unicamente en la interfaz web sin compilar el binario de Rust:

```bash
npm run dev
```

La interfaz estara accesible en `http://localhost:3000`. Nota: Las llamadas que requieren IPC nativo de Tauri cuentan con un mecanismo de respaldo por HTTP directo para facilitar pruebas visuales en navegador.

## Compilacion para Produccion

Para generar el export estatico del frontend web:

```bash
npm run build
```

Los artefactos estaticos se generaran en el directorio `out/`.

Para compilar el binario instalable de escritorio:

```bash
npm run tauri build
```
