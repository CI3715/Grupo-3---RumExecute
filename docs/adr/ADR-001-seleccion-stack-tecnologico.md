# ADR-001: Seleccion del Stack Tecnologico Inicial

## Estado
Aceptado

## Fecha
2026-10-06

## Contexto
El proyecto **Cuentas Claras** (CI-3715: Ingenieria de Software) exige el desarrollo de una solucion de finanzas personales y control de gastos compartidos dividida estrictamente en dos piezas desacopladas:

1. **Aplicacion Cliente:** Debe garantizar soberania de datos, almacenamiento local de cuentas y movimientos personales bajo un modelo inmutable de Event Sourcing, sin que estos datos sensibles salgan jamas del dispositivo del usuario.
2. **Servidor Social:** Debe actuar como coordinador de identidades, amistades y grupos de gastos compartidos, exponiendo una API HTTP documentada mediante un contrato formal OpenAPI.

Para la Entrega 1 ("The Full Stack Ping"), la catedra permite optar por aplicaciones de escritorio (Tauri 2 + Next.js) o moviles (Flutter / Kotlin), y servidores en Rust (Axum), TypeScript (Hono) o Python (FastAPI).

## Decisiones Adoptadas

### 1. Cliente: Tauri 2 (Rust) + Next.js (React / TypeScript)
* **Razon de Eleccion:**
  * **Consumo de Recursos y Rendimiento:** A diferencia de Electron, Tauri 2 utiliza el motor web nativo del sistema operativo (WebView2 en Windows, WebKit en macOS/Linux), logrando ejecutables de menor tamano y consumo de memoria RAM reducido.
  * **Seguridad y Capacidad Nativa:** El nucleo de Tauri en Rust proporciona un entorno robusto para alojar el motor de Event Sourcing y la base de datos embebida SQLite (`rusqlite`) en las entregas subsiguientes, garantizando que la logica financiera critica y los tipos decimales exactos no dependan de la capa de presentacion.
  * **Experiencia de Usuario:** Next.js bajo modalidad de exportacion estatica (`output: 'export'`) permite componer interfaces modernas y reactivas en TypeScript manteniendo total compatibilidad con el modelo de empaquetado de Tauri.
  * **Aislamiento Arquitectonico:** La comunicacion entre la interfaz de usuario y las funciones nativas se canaliza exclusivamente mediante comandos IPC tipados (`invoke(...)`), asegurando el patron de Puertos y Adaptadores.
  * **Experiencia Previa y Facilidad de Aprendizaje:** Se eligio Tauri 2 y Next.js debido a la experiencia previa de miembros del equipo en el uso de Tauri, Rust y Next.js, lo que facilita el desarrollo y mantenimiento del cliente.

### 2. Servidor Backend: Python 3.11+ con FastAPI
* **Razon de Eleccion:**
  * **Velocidad de Desarrollo y Mantenibilidad:** FastAPI ofrece una curva de aprendizaje optima y sintaxis declarativa para construir APIs REST de alto rendimiento sobre ASGI (Starlette / Uvicorn).
  * **Validacion Estricta y Tipado:** La integracion nativa con Pydantic permite validar automaticamente los esquemas de entrada y salida, asegurando cumplimiento riguroso con el contrato `openapi.yaml`.
  * **Ecosistema:** Facil integracion futura para calculos analiticos, librerias de soporte y pruebas automatizadas livianas.
  * **Experiencia Previa y Facilidad de Aprendizaje:** Se eligio FastAPI debido a la experiencia previa de miembros del equipo en el uso de Python y FastAPI, lo que facilita el desarrollo y mantenimiento del servidor.

## Consecuencias

### Positivas
* Clara separacion de responsabilidades entre la capa de interfaz grafica (Next.js), el nucleo de dominio de escritorio (Rust) y el servidor de coordinacion (FastAPI).
* Cumplimiento estricto con los requerimientos de privacidad y soberania definidos en la Guia Tecnica.
* Facilidad para realizar pruebas unitarias y de contrato independientes en cada modulo.

### Negativas / Mitigaciones
* **Requisitos del entorno de desarrollo:** Los integrantes del equipo que desarrollen sobre el cliente requieren toolchains de Node.js y Rust (Cargo) configurados en sus equipos. Como mitigacion, la interfaz web cuenta con un mecanismo de ejecucion desacoplado en navegador (`npm run dev`) para desarrollo rapido de maquetacion.
* **Serializacion IPC:** Toda comunicacion cliente-nucleo debe definirse formalmente mediante estructuras serializables con Serde.
