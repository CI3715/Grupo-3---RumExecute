# Cuentas Claras — Grupo 3 (RumExecute)

Proyecto de desarrollo de software para la asignatura **CI-3715: Ingeniería de Software**, Universidad Simón Bolívar (USB).

---

## Integrantes del Equipo

* **Santiago Armando Bueno Utrera** — Carnet: `20-10168`
* **Victor Manuel Hernández Avilán** — Carnet: `20-10349`
* **Daniel Eduardo Quijada Malavé** — Carnet: `20-10518`
* **David Antonio Paulo Medina** — Carnet: `21-10475`
* **Jean Carlos Sifontes Palacios** — Carnet: `22-10387`

---

## Descripción del Proyecto

**Cuentas Claras** es una plataforma integral para la gestión de finanzas personales multimoneda y la coordinación de gastos compartidos en grupo, construida bajo los principios de **soberanía de datos, funcionamiento sin conexión (*Offline-First*) e inmutabilidad contable**.

El sistema se divide estrictamente en dos ámbitos independientes (`INV-16` / `RES-02`):

1. **Libro Financiero Personal (100% Local en el Dispositivo):**
   * Almacena y procesa en el dispositivo todas las cuentas personales, ingresos, gastos, correcciones inmutables por ajuste, categorías personalizadas, monedas propias, tasas de cambio históricas y el cálculo de patrimonio consolidado.
   * Funciona sin conexión a internet y sin necesidad de crear una cuenta de usuario ni iniciar sesión (`RF-SOC-03`).
   * Los datos personales pertenecen al dispositivo y **jamás** se envían ni sincronizan con el servidor remoto.
2. **Coordinación Social y Gastos Compartidos (Servidor Backend):**
   * Gestiona el registro e inicio de sesión de usuarios (`Argon2id` + `JWT`), la búsqueda exacta y privada por `@nombre_usuario`, las solicitudes de amistad, la creación de grupos colaborativos, el registro de gastos compartidos (`IGUALES`, `PORCENTAJES`, `MONTOS`) con reparto determinista de centavos residuales (`ESC-10`) y el cálculo de balances grupales de suma cero (`INV-08`).
   * Toda la información social se persiste en **PostgreSQL 16** contenerizado con Docker y volumen persistente.

---

## Stack Tecnológico

* **Cliente de Escritorio y Web:** Tauri 2 (Rust) + Next.js (React 19 / TypeScript) + SQLite (`rusqlite`) + Llavero seguro del Sistema Operativo (`keyring`).
* **Servidor Backend:** Python 3.11+ con FastAPI, Uvicorn, SQLAlchemy 2.0, Psycopg 3, Argon2-CFFI y PyJWT.
* **Base de Datos del Servidor:** PostgreSQL 16 (`postgres:16-alpine`) orquestada con Docker Compose y volumen persistente (`postgres_data`).
* **Contrato Formal de API:** Especificación OpenAPI 3.1 (`openapi.yaml`).

---

## Estructura del Repositorio (Monorepo)

```text
Grupo-3---RumExecute/
|-- docs/                                           # Documentación técnica y decisiones de arquitectura
|   |-- adr/                                        # Architecture Decision Records (ADR-001 a ADR-008)
|   |-- entrega4/                                   # Documentación de soporte de la Entrega 4
|   |-- Documento_Tecnico_Explicativo_Entrega_4.md  # Documento técnico de la Entrega 4
|   `-- Documento_Tecnico_Explicativo_Entrega_5_y_6.md # Documento técnico de las Entregas 5 y 6
|-- front_end/                                      # Aplicación cliente (Tauri 2 + Next.js)
|   |-- src/                                        # Interfaz de usuario (componentes, hooks y servicios)
|   |-- src-tauri/                                  # Núcleo en Rust (dominio, aplicación, SQLite, IPC y keyring)
|   |-- tests/                                      # Pruebas automatizadas del frontend
|   `-- README.md                                   # Guía detallada de compilación, uso y pruebas del cliente
|-- servidor/                                       # Servidor backend de coordinación social (FastAPI)
|   |-- dominio/                                    # Entidades, objetos de valor y reglas de negocio puras
|   |-- aplicacion/                                 # Casos de uso y puertos (arquitectura hexagonal)
|   |-- infraestructura/                            # Repositorios PostgreSQL, hashing Argon2id y tokens JWT
|   |-- api/                                        # Routers HTTP FastAPI, esquemas Pydantic y dependencias
|   |-- tests/                                      # Suite de pruebas (dominio, Hypothesis, aplicación e integración)
|   |-- main.py                                     # Punto de entrada de la aplicación FastAPI
|   |-- generar_openapi.py                          # Generador automático del contrato openapi.yaml
|   |-- requirements.txt                            # Dependencias de Python
|   `-- README.md                                   # Guía detallada del servidor, endpoints y pruebas
|-- docker-compose.yml                              # Orquestación de PostgreSQL 16 con volumen persistente
|-- openapi.yaml                                    # Contrato formal OpenAPI 3.1 de todos los endpoints
|-- LICENSE                                         # Licencia Apache-2.0
`-- README.md                                       # Documentación general del proyecto
```

---

## Resumen de Entregas Implementadas

| Entrega | Alcance Principal | Componentes Clave |
| :--- | :--- | :--- |
| **Entrega 1** | Ambiente básico y conexión inicial (*The Full Stack Ping*) | Endpoint `GET /ping`, comando IPC `ping_servidor` en Rust con medición de latencia y diagnóstico de red en UI. |
| **Entrega 2** | Cuentas personales y subtotales por moneda | Dominio y casos de uso en Rust para creación de cuentas, validación de saldos y agrupación por divisa sin mezclar monedas. |
| **Entrega 3** | Movimientos inmutables y corrección mediante ajustes | Registro de `ingreso`, `gasto` y `ajuste` en SQLite local con triggers de inmutabilidad (`BEFORE UPDATE` / `BEFORE DELETE`). |
| **Entrega 4** | Multimoneda, tasas históricas y patrimonio consolidado | Catálogo con 4 monedas de fábrica (`VES`, `USD`, `EUR`, `USDT`) + monedas propias, tasas por fecha, tasa inversa (`1 / valor`) sin triangulación y consolidación patrimonial. |
| **Entrega 5** | Registro, autenticación, sesión y autonomía sin cuenta | Registro e inicio de sesión (`Argon2id` + `JWT`), persistencia en PostgreSQL (Docker), llavero seguro del SO (`keyring`) y uso completo del libro personal sin sesión (`INV-16`). |
| **Entrega 6** | Amigos, grupos, gastos compartidos y balances | Búsqueda exacta por `@usuario`, solicitudes de amistad, grupos colaborativos, división de gastos (`IGUALES`, `PORCENTAJES`, `MONTOS`), residuo determinista (`ESC-10`) y balances (`INV-08`). |

---

## Guía Rápida de Puesta en Marcha Completa

Para ejecutar todo el ecosistema en local se requieren **Docker Desktop**, **Python 3.11+**, **Node.js 20+** (con `pnpm` o `npm`) y **Rust 1.77+** (para modo escritorio Tauri).

### 1. Levantar la Base de Datos PostgreSQL con Docker

Desde la raíz del repositorio:

```powershell
docker compose up -d
```

Esto inicia el contenedor `cuentas_claras_db` (`postgres:16-alpine`) expuesto en el puerto local **`5433`** con el volumen persistente `postgres_data`. Para verificar que está corriendo:

```powershell
docker compose ps
```

### 2. Iniciar el Servidor Backend (FastAPI)

Abrir una terminal en la carpeta `servidor/`:

```powershell
cd servidor

# Crear y activar entorno virtual (si aún no existe)
python -m venv .venv
# En Windows (PowerShell):
.\.venv\Scripts\Activate.ps1
# En Linux / macOS:
# source .venv/bin/activate

# Instalar dependencias
pip install -r requirements.txt

# Iniciar el servidor en el puerto 8000
python -m uvicorn main:app --reload --port 8000
```

* API disponible en: `http://127.0.0.1:8000`
* Documentación interactiva Swagger UI: `http://127.0.0.1:8000/docs`

### 3. Iniciar la Aplicación Cliente

Abrir otra terminal en la carpeta `front_end/`:

```powershell
cd front_end

# Instalar dependencias
pnpm install

# Opción A: Ejecutar aplicación de escritorio nativa (Tauri + Rust + SQLite)
pnpm tauri dev

# Opción B: Ejecutar interfaz en navegador web (http://localhost:3000)
pnpm dev
```

---

## Comandos de Pruebas Automatizadas

### Pruebas del Servidor Backend (Dominio, Propiedades con Hypothesis, Aplicación e Integración PostgreSQL)

```powershell
cd servidor
python -m pytest tests/ -v
```

### Pruebas del Núcleo Rust / Tauri (Dominio, Casos de Uso, Contrato e Inmutabilidad SQLite)

```powershell
cd front_end
cargo test --manifest-path src-tauri/Cargo.toml
```

### Pruebas del Frontend (TypeScript / Node Test Runner)

```powershell
cd front_end
pnpm test
```

---

## Documentación Arquitectónica (ADRs)

* [ADR-001: Selección del Stack Tecnológico](docs/adr/ADR-001-seleccion-stack-tecnologico.md)
* [ADR-002: Arquitectura Hexagonal y Modelado de Dominio](docs/adr/ADR-002-arquitectura-hexagonal-y-modelado-dominio.md)
* [ADR-003: Almacenamiento Local Inmutable en SQLite](docs/adr/ADR-003-almacenamiento-local-inmutable-sqlite.md)
* [ADR-005: Tasas de Cambio, Tasa Inversa y Tasa Faltante](docs/adr/ADR-005-tasas-de-cambio-inversa-y-tasa-faltante.md)
* [ADR-006: PostgreSQL y Persistencia Social](docs/adr/ADR-006-postgresql-y-persistencia-social.md)
* [ADR-007: Autenticación, Hashing Argon2id y Sesiones JWT](docs/adr/ADR-007-autenticacion-hashing-y-sesiones.md)
* [ADR-008: Reparto Determinista de Gastos y Balances de Suma Cero](docs/adr/ADR-008-reparto-determinista-y-balances.md)
* [Documento Técnico Explicativo — Entrega 4](docs/Documento_Tecnico_Explicativo_Entrega_4.md)
* [Documento Técnico Explicativo — Entregas 5 y 6](docs/Documento_Tecnico_Explicativo_Entrega_5_y_6.md)

---

## Licencia

Este proyecto está distribuido bajo la Licencia Apache 2.0. Consulte el archivo [LICENSE](LICENSE) para más información.
