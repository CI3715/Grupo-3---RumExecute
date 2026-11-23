# Documento técnico explicativo — Entrega 1

**Proyecto:** Cuentas Claras · **Equipo:** Grupo 3 — RumExecute
**Asignatura:** CI-3715 Ingeniería de Software · Universidad Simón Bolívar

## 1. Propósito y alcance

El objetivo central de la Entrega 1 consiste en construir y validar los cimientos arquitectónicos del proyecto **Cuentas Claras**, logrando la comunicación de extremo a extremo (*Full-Stack*) entre la aplicación cliente de escritorio y el servidor backend a través del flujo de diagnóstico denominado **The Ping Flow**.

Desde esta primera entrega se establecen los principios arquitectónicos del sistema:

- **Soberanía y privacidad estricta de datos (`INV-16` / `RES-02`):** El servidor nunca recibe ni almacena cuentas bancarias, balances, categorías personales ni movimientos financieros individuales. Toda esa información reside y se calcula exclusivamente en el dispositivo del usuario.
- **Arquitectura desacoplada en dos piezas:**
  1. **Cliente de Escritorio (Tauri 2 + Next.js):** Dueño del libro de cuentas personal y la base de datos SQLite local embebida.
  2. **Servidor Social (Python + FastAPI):** Encargado exclusivamente de coordinar identidades, amistades y grupos de gastos compartidos.
- **Aritmética decimal exacta (`RNF-08`):** Prohibición de coma flotante binaria (`float` / `double`) para cálculos monetarios (`rust_decimal` en Rust y `Decimal` en Python).
- **Arquitectura Hexagonal (Puertos y Adaptadores):** El núcleo de dominio permanece libre de dependencias de frameworks o librerías de infraestructura.

## 2. Selección del stack tecnológico (ADR-001)

| Pieza del sistema | Tecnología principal | Rol arquitectónico | Justificación técnica |
|---|---|---|---|
| Cliente de Escritorio (UI) | Next.js (React 19 + TypeScript) | Interfaz gráfica y captura de eventos | Modularidad por componentes, tipado estático estricto y exportación estática (`output: 'export'`). |
| Cliente de Escritorio (Core) | Tauri 2 (Rust) | Núcleo nativo, canal IPC y motor SQLite | Huella de memoria mínima (< 40 MB), binario ligero y aislamiento seguro respecto al sistema operativo. |
| Servidor Social (Backend) | Python 3.11+ / FastAPI | API HTTP de coordinación social | Validación declarativa con Pydantic, rapidez de desarrollo y generación del contrato OpenAPI 3.1. |
| Servidor Web ASGI | Uvicorn | Servidor asíncrono HTTP | Despacho concurrente y compatibilidad multiplataforma (Windows, Linux y macOS). |

La justificación detallada frente a alternativas como Electron o Flutter se documenta en [ADR-001 — Selección del Stack Tecnológico](adr/ADR-001-seleccion-stack-tecnologico.md).

## 3. Flujo extremo a extremo (The Ping Flow)

1. El usuario abre la ventana de diagnóstico de red en la interfaz Next.js y pulsa **"Verificar Conexión con el Servidor"**.
2. La interfaz invoca el comando nativo `invoke("ping_servidor")` a través del puente IPC de Tauri (con respaldo HTTP directo cuando se ejecuta en modo navegador web durante el desarrollo).
3. En `front_end/src-tauri/src/lib.rs`, el comando `ping_servidor` captura el instante inicial con un reloj monotónico (`std::time::Instant::now()`) y envía una petición `GET http://127.0.0.1:8000/ping` mediante `reqwest`.
4. El servidor FastAPI (`servidor/main.py`) procesa la solicitud, genera la marca de tiempo actual en formato ISO-8601 UTC (`Z`) y responde `200 OK` con el esquema validado por Pydantic:
   ```json
   {
     "estado": "ok",
     "mensaje": "Servidor Cuentas Claras activo",
     "version": "0.1.0",
     "timestamp": "2026-10-08T19:30:00Z"
   }
   ```
5. El núcleo en Rust deserializa la respuesta JSON, calcula la latencia de ida y vuelta en milisegundos (`inicio.elapsed().as_millis()`) y retorna el resultado tipado `PingResult` a la interfaz.
6. La interfaz actualiza el estado visual (*Desconectado*, *Verificando...*, *Conectado* o *Error de Conexión*) y muestra el estado, versión, timestamp UTC y latencia en milisegundos.

## 4. Contrato OpenAPI y pruebas de correspondencia

El contrato formal se define en [`openapi.yaml`](../openapi.yaml) bajo la especificación OpenAPI 3.1.0. La prueba automatizada `servidor/test_correspondencia_openapi.py` verifica que el endpoint `GET /ping` cumpla estrictamente con los campos, tipos y formato de fecha definidos en el contrato.
