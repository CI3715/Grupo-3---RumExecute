# Servidor Backend — Cuentas Claras

Módulo de servidor backend HTTP para la coordinación social y de gastos compartidos de **Cuentas Claras**, desarrollado con **Python 3.11+**, **FastAPI**, **SQLAlchemy 2.0**, **PostgreSQL 16**, **Argon2-CFFI** y **PyJWT** bajo una **Arquitectura Hexagonal (Puertos y Adaptadores)**.

---

## Principio de Privacidad y Soberanía de Datos (`INV-16` / `RES-02`)

De acuerdo con la arquitectura del sistema, el servidor backend **únicamente** coordina identidades de usuarios, relaciones de amistad, grupos colaborativos, gastos compartidos y balances grupales. **Jamás** recibe, procesa ni almacena cuentas bancarias personales, saldos privados, categorías locales, tasas de cambio ni el patrimonio individual del usuario.

---

## Estructura Interna del Servidor

```text
servidor/
|-- dominio/                  # Reglas de negocio puras, entidades y objetos de valor
|   |-- modelos.py            # Usuario, Amistad, Grupo, GastoCompartido, Cuota, ReglaDivision
|   |-- servicios.py          # Cálculo determinista de cuotas (ESC-10) y balances (INV-08)
|   `-- excepciones.py        # Errores de dominio tipados por campo y código semántico
|-- aplicacion/               # Casos de uso y puertos (interfaces abstractas)
|   |-- puertos.py            # Contratos de repositorios, hashing y proveedor de tokens
|   |-- casos_uso_usuarios.py # Registro, inicio de sesión, perfil y búsqueda exacta
|   |-- casos_uso_amistades.py# Invitaciones, aceptación/rechazo, listado y eliminación
|   `-- casos_uso_grupos.py   # Creación de grupos, miembros, gastos compartidos y balances
|-- infraestructura/          # Adaptadores concretos de persistencia y seguridad
|   |-- base_datos.py         # Configuración de motor SQLAlchemy y sesiones PostgreSQL
|   |-- modelos_db.py         # Tablas relacionales (usuarios, amistades, grupos, gastos, cuotas)
|   |-- repo_usuarios_pg.py   # Repositorio PostgreSQL de usuarios
|   |-- repo_amistades_pg.py  # Repositorio PostgreSQL de amistades
|   |-- repo_grupos_pg.py     # Repositorio PostgreSQL de grupos y miembros
|   |-- repo_gastos_pg.py     # Repositorio PostgreSQL de gastos compartidos y cuotas
|   `-- seguridad.py          # Hashing Argon2id y emisión/verificación de tokens JWT (HS256)
|-- api/                      # Capa de transporte HTTP (FastAPI)
|   |-- esquemas.py           # Modelos Pydantic de entrada/salida y errores estructurados
|   |-- dependencias.py       # Inyección de casos de uso y extracción de Bearer token
|   |-- rutas_auth.py         # Endpoints de usuarios, sesiones, perfil y búsqueda
|   |-- rutas_amistades.py    # Endpoints de solicitudes y lista de amigos
|   `-- rutas_grupos.py       # Endpoints de grupos, invitaciones, gastos y balances
|-- tests/                    # Suite de pruebas automatizadas (Pytest + Hypothesis)
|-- main.py                   # Aplicación FastAPI, CORS, ciclo de vida y /ping, /salud
|-- generar_openapi.py        # Script para exportar el esquema formal a ../openapi.yaml
`-- requirements.txt          # Dependencias del servidor
```

---

## Requisitos Previos

* **Python 3.11 o superior** con `pip` y `venv`.
* **Docker Desktop** (para levantar la base de datos PostgreSQL 16).

---

## 1. Levantar la Base de Datos PostgreSQL con Docker

Desde la **raíz del repositorio** (`Grupo-3---RumExecute/`):

```powershell
docker compose up -d
```

Esto levanta el contenedor `cuentas_claras_db` (`postgres:16-alpine`) mapeado al puerto **`5433`** del host y respaldado por el volumen persistente `postgres_data`.

* **Verificar estado del contenedor:**
  ```powershell
  docker compose ps
  ```
* **Comprobar persistencia tras reinicio:**
  ```powershell
  docker compose restart
  ```
  Los usuarios, amistades, grupos y gastos registrados se mantienen intactos gracias al volumen `postgres_data`.

### Variables de Entorno (Valores por defecto para desarrollo local)

El servidor utiliza automáticamente los valores configurados en `docker-compose.yml` si no se definen variables de entorno adicionales:

| Variable | Valor por defecto | Descripción |
| :--- | :--- | :--- |
| `DB_USER` | `postgres` | Usuario de PostgreSQL |
| `DB_PASSWORD` | `password123` | Contraseña de PostgreSQL |
| `DB_HOST` | `localhost` | Host de la base de datos |
| `DB_PORT` | `5433` | Puerto expuesto en el host |
| `DB_NAME` | `cuentas_claras` | Nombre de la base de datos |
| `DATABASE_URL` | `postgresql+psycopg://postgres:password123@localhost:5433/cuentas_claras` | URI completa de conexión SQLAlchemy |
| `JWT_SECRET_KEY` | Clave local de desarrollo | Secreto para firma HS256 de tokens JWT |

---

## 2. Instalación y Ejecución del Servidor FastAPI

Desde el directorio `servidor/`:

```powershell
# 1. Crear entorno virtual
python -m venv .venv

# 2. Activar entorno virtual
# En Windows (PowerShell):
.\.venv\Scripts\Activate.ps1
# En Linux / macOS:
# source .venv/bin/activate

# 3. Instalar dependencias
pip install -r requirements.txt

# 4. Iniciar servidor en modo desarrollo
python -m uvicorn main:app --reload --port 8000
```

* **URL base del servidor:** `http://127.0.0.1:8000`
* **Swagger UI interactivo:** `http://127.0.0.1:8000/docs`
* **ReDoc:** `http://127.0.0.1:8000/redoc`

---

## Endpoints Disponibles (Entregas 1, 5 y 6)

| Método y Ruta | Autenticación | Descripción |
| :--- | :---: | :--- |
| `GET /ping` | Público | Verificación de estado y latencia (*The Full Stack Ping* — Entrega 1). |
| `GET /salud` | Público | Estado de salud del servicio social. |
| `POST /usuarios` | Público | Registro de nuevo usuario (`nombre_usuario`, `nombre_visible`, `correo`, `contrasena` con hash `Argon2id`). |
| `POST /sesiones` | Público | Inicio de sesión; retorna token temporal JWT (`Bearer`) y datos públicos del usuario. |
| `GET /perfil` y `GET /usuarios/yo` | Bearer JWT | Retorna el perfil del usuario autenticado (`id`, `nombre_usuario`, `nombre_visible`, `correo`). |
| `DELETE /sesiones` | Bearer JWT | Cierre formal de la sesión activa (`204 No Content`). |
| `GET /usuarios?usuario=<username>` | Bearer JWT | Búsqueda exacta por `nombre_usuario`; retorna solo `nombre_usuario` y `nombre_visible` (sin correo). |
| `GET /amistades` | Bearer JWT | Lista amigos activos, solicitudes recibidas pendientes y solicitudes enviadas pendientes. |
| `POST /amistades` | Bearer JWT | Envía una solicitud de amistad indicando el `nombre_usuario` destinatario. |
| `POST /amistades/{id}/respuesta` | Bearer JWT | Acepta (`{"aceptar": true}`) o rechaza (`{"aceptar": false}`) una solicitud de amistad recibida. |
| `DELETE /amistades/{id}` | Bearer JWT | Elimina una relación de amistad existente. |
| `GET /grupos` | Bearer JWT | Lista los grupos a los que pertenece el usuario y las invitaciones a grupos pendientes. |
| `POST /grupos` | Bearer JWT | Crea un nuevo grupo colaborativo (`nombre`, `moneda`, `descripcion`). |
| `POST /grupos/{id}/miembros` | Bearer JWT | Invita a un amigo activo al grupo por su `nombre_usuario`. |
| `POST /grupos/{id}/miembros/yo/respuesta` | Bearer JWT | Acepta o rechaza una invitación pendiente para unirse a un grupo. |
| `POST /grupos/{id}/gastos` | Bearer JWT | Registra un gasto compartido (`IGUALES`, `PORCENTAJES` o `MONTOS`) y calcula cuotas deterministas (`ESC-10`). |
| `GET /grupos/{id}` | Bearer JWT | Obtiene el detalle del grupo, miembros, historial de gastos con cuotas y balances netos (`INV-08`). Retorna `404` si el usuario no es miembro (`INV-17`). |

---

## Ejecución de Pruebas Automatizadas

### 1. Suite Completa de Dominio, Propiedades (Hypothesis), Aplicación e Integración PostgreSQL

Con el contenedor de PostgreSQL activo (`docker compose up -d`), ejecutar desde `servidor/`:

```powershell
python -m pytest tests/ -v
```

Esta suite verifica:
* Validaciones de registro, unicidad de correo y `nombre_usuario`, hashing `Argon2id` y rechazo de tokens inválidos o expirados.
* Pruebas basadas en propiedades con **Hypothesis**: conservación exacta de centavos (`suma(cuotas) == monto_total`), asignación determinista del residuo por orden lexicográfico (`ESC-10`) e invariante de suma cero de balances grupales (`INV-08`).
* Aislamiento de grupos (`ESC-08` / `INV-17`: un usuario no miembro recibe `404` idéntico al de un grupo inexistente).
* Integración completa contra **PostgreSQL** en Docker.

### 2. Prueba de Contrato OpenAPI (`/ping`)

Con el servidor corriendo en `http://127.0.0.1:8000`, en otra terminal ejecutar:

```powershell
python -m pytest test_correspondencia_openapi.py -v
```

### 3. Regenerar el Contrato Formal `openapi.yaml`

Si se modifican rutas o esquemas en `servidor/api/`, se puede actualizar el archivo `openapi.yaml` de la raíz ejecutando:

```powershell
python generar_openapi.py
```
