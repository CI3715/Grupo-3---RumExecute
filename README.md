# Cuentas Claras - Grupo 3 (RumExecute)

Proyecto de desarrollo de software para la asignatura **CI-3715: Ingenieria de Software**, Universidad Simon Bolivar (USB).

---

## Integrantes del Equipo

* **Santiago Armando Bueno Utrera** - Carnet: 20-10168
* **Victor Manuel Hernandez Avilan** - Carnet: 20-10349
* **Daniel Eduardo Quijada Malave** - Carnet: 20-10518
* **David Antonio Paulo Medina** - Carnet: 21-10475
* **Jean Carlos Sifontes Palacios** - Carnet: 22-10387

---

## Descripcion del Proyecto

**Cuentas Claras** es una solucion integral para la gestion de finanzas personales y control de gastos compartidos con soporte multimoneda (USD y Bolivares con referencia a tasas oficiales BCV).

El sistema sigue una arquitectura desacoplada basada en el principio de soberania y privacidad de datos:

1. **Aplicacion Cliente (Escritorio):** Almacena y procesa de forma completamente local y privada los movimientos financieros, cuentas, saldos y presupuestos del usuario mediante una base de datos SQLite embebida bajo un esquema inmutable de Event Sourcing.
2. **Servidor Social (Backend):** Plataforma orientada a la coordinacion social entre usuarios (amistades, grupos de gastos compartidos y solicitudes de cobro), sin recibir ni almacenar jamas transacciones personales ni informacion bancaria sensible.

---

## Stack Tecnologico

* **Cliente de Escritorio:** Tauri 2 (Rust) + Next.js (React 19 / TypeScript) con exportacion estatica.
* **Servidor Backend:** Python 3.11+ con FastAPI y Uvicorn.
* **Contrato de Interfaz:** Especificacion formal OpenAPI 3.1 (`openapi.yaml`).

---

## Estructura del Repositorio (Monorepo)

```text
Grupo-3---RumExecute/
|-- docs/                # Documentacion de arquitectura y diseno
|   `-- adr/             # Architecture Decision Records (ADRs)
|       `-- ADR-001-seleccion-stack-tecnologico.md
|-- front_end/           # Aplicacion cliente de escritorio (Tauri 2 + Next.js)
|   |-- src/             # Interfaz web y componentes de usuario (Next.js)
|   |-- src-tauri/       # Nucleo nativo de escritorio y comandos IPC en Rust
|   |-- package.json     # Dependencias y scripts de Node.js
|   `-- README.md        # Documentacion especifica del cliente
|-- servidor/            # Servidor backend de coordinacion social (Python / FastAPI)
|   |-- main.py          # Aplicacion FastAPI y endpoints HTTP (/ping)
|   |-- requirements.txt # Dependencias del servidor backend
|   `-- README.md        # Documentacion especifica del servidor
|-- openapi.yaml         # Contrato formal de comunicacion API
|-- LICENSE              # Licencia Apache-2.0
|-- .gitignore           # Reglas de exclusion de Git para monorepo
`-- README.md            # Documentacion general del repositorio
```

---

## Entrega 1: Ambiente Basico y Conexion Inicial (The Full Stack Ping)

En esta primera fase del proyecto se implemento y valido el flujo completo de comunicacion entre la aplicacion cliente y el servidor backend:

1. **Endpoint Backend (`GET /ping`):** Expone un servicio HTTP de solo lectura que responde con codigo 200 OK y la carga estandarizada:
   ```json
   {
     "estado": "ok",
     "mensaje": "Servidor Cuentas Claras activo",
     "version": "0.1.0",
     "timestamp": "2026-10-08T19:30:00Z"
   }
   ```
2. **Nucleo Nativo Tauri en Rust:** El comando `ping_servidor` en `front_end/src-tauri/src/lib.rs` efectua la peticion HTTP al backend mediante `reqwest`, deserializa la respuesta, calcula la latencia de ida y vuelta en milisegundos (`Instant::now().elapsed().as_millis()`) y retorna la informacion estructurada a la interfaz.
3. **Interfaz de Usuario (Next.js):** Ofrece un boton claramente identificado (*"Verificar Conexion con el Servidor"*), gestiona estados visuales (*Desconectado*, *Verificando...*, *Conectado*, *Error de Conexion*) y despliega las metricas obtenidas incluyendo estado, version, mensaje y latencia exacta en milisegundos.

---

## Instrucciones de Instalacion y Ejecucion Local

Para probar el proyecto completo se deben iniciar ambos servicios en terminales separadas.

### 1. Iniciar el Servidor Backend

Abrir una terminal en la raiz del repositorio:

```bash
cd servidor

# Crear entorno virtual
python -m venv .venv

# Activar entorno virtual
# En Windows (PowerShell):
.venv\Scripts\Activate.ps1
# En Linux / macOS:
# source .venv/bin/activate

# Instalar dependencias
pip install -r requirements.txt

# Iniciar servidor FastAPI
uvicorn main:app --reload --port 8000
```

El servidor quedara escuchando en `http://localhost:8000`.

### 2. Iniciar la Aplicacion Cliente

Abrir una segunda terminal en la raiz del repositorio:

```bash
cd front_end

# Instalar dependencias de Node.js
npm install

# Iniciar aplicacion nativa de escritorio con Tauri
npm run tauri dev
```

*Nota para desarrollo rapido en navegador:* Tambien es posible ejecutar unicamente la interfaz web con `npm run dev` (disponible en `http://localhost:3000`), la cual incorpora un mecanismo de comunicacion compatible con el servidor para pruebas de maquetado.

---

## Licencia

Este proyecto esta bajo la Licencia Apache 2.0. Consulte el archivo [LICENSE](LICENSE) para mas informacion.
