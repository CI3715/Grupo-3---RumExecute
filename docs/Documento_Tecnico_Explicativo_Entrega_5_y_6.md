# Documento técnico explicativo — Entregas 5 y 6

**Proyecto:** Cuentas Claras · **Equipo:** Grupo 3 — RumExecute
**Asignatura:** CI-3715 Ingeniería de Software · Universidad Simón Bolívar

## 1. Propósito y alcance

Este documento consolida la justificación arquitectónica, el contrato HTTP y la estrategia de pruebas de las Entregas 5 y 6. El trabajo se organiza en cinco módulos: dominio, aplicación, infraestructura, interfaz de usuario, y documentación y pruebas. Este último corresponde al Módulo 5.

La Entrega 5 incorpora registro de usuarios, autenticación y consulta del perfil propio. La Entrega 6 agrega amistades, grupos, gastos compartidos y balances. En ambas entregas rige el principio **lo personal permanece en el cliente; lo compartido se coordina en el servidor**.

El servidor no almacena cuentas bancarias, movimientos personales, categorías privadas, tasas de cambio ni patrimonio del usuario. Registrarse es opcional: las funciones financieras locales deben permanecer disponibles sin conexión y sin iniciar sesión; la autenticación habilita las funciones sociales.

## 2. Requerimientos e invariantes cubiertos

| Área | Requerimientos y comportamiento |
|---|---|
| Autenticación | Registro (RF-SOC-01), inicio de sesión con token temporal (RF-SOC-02) y perfil propio protegido. |
| Autonomía y privacidad | El registro es opcional (RF-SOC-03); la información financiera personal no se envía al servidor (INV-16 / RES-02). |
| Seguridad | Hash de contraseña con Argon2id (RNF-05); los tokens expirados, alterados o ausentes no autorizan endpoints protegidos. |
| Amistades | Búsqueda exacta y privada (RF-SOC-04), invitaciones, aceptación/rechazo, listado y eliminación (RF-SOC-05/06, INV-10). |
| Grupos y gastos | Crear grupos, invitar amistades, responder invitaciones, registrar gastos en partes iguales, porcentajes o montos, y consultar balances (RF-COM-01/02/04/05/08). |
| Importes | Aritmética decimal, conservación de centavos y asignación determinista de residuos (RNF-08, INV-07 / ESC-10). |
| Balances y aislamiento | La suma de balances de un grupo es cero (INV-08). Un usuario no miembro recibe el mismo 404 que para un grupo inexistente (INV-17 / ESC-08). |

## 3. Arquitectura y responsabilidades

La API implementa arquitectura hexagonal. La capa de dominio contiene las reglas de usuarios, amistades, grupos, reparto y balances. Los casos de uso de `servidor/aplicacion/` coordinan esas reglas mediante puertos; los adaptadores de `servidor/infraestructura/` implementan persistencia PostgreSQL, hashing y tokens. Los routers de `servidor/api/` traducen HTTP a casos de uso y resultados a respuestas HTTP.

En el cliente, los comandos Tauri median entre la interfaz y los casos de uso Rust. La capa de autenticación del cliente usa `reqwest` para comunicarse con el backend y `keyring` para guardar el token en el almacén seguro del sistema operativo, no en `localStorage`.

La persistencia del servidor se limita a identidades, amistades, grupos, miembros, gastos compartidos y cuotas. Los gastos y sus cuotas son hechos persistidos; los balances se calculan al consultar el grupo en vez de mantener un saldo mutable, reduciendo el riesgo de descuadres y carreras.

## 4. API y contrato OpenAPI

El contrato formal OpenAPI 3.1 está en [`openapi.yaml`](../openapi.yaml). La aplicación también publica la documentación interactiva en `/docs` y `/redoc`.

| Método y ruta | Acceso | Propósito |
|---|---|---|
| `POST /usuarios` | Público | Registrar usuario. |
| `POST /sesiones` | Público | Validar credenciales y emitir token. |
| `GET /usuarios/yo` | Bearer | Obtener perfil del usuario autenticado. |
| `GET /usuarios?usuario=...` | Bearer | Buscar por nombre de usuario exacto; no expone el correo. |
| `GET /amistades` | Bearer | Listar amistades e invitaciones. |
| `POST /amistades` | Bearer | Enviar invitación de amistad. |
| `POST /amistades/{id}/respuesta` | Bearer | Aceptar o rechazar invitación. |
| `DELETE /amistades/{id}` | Bearer | Eliminar amistad activa. |
| `GET /grupos` | Bearer | Listar grupos e invitaciones recibidas. |
| `POST /grupos` | Bearer | Crear grupo. |
| `POST /grupos/{id}/miembros` | Bearer | Invitar amistad al grupo. |
| `POST /grupos/{id}/miembros/yo/respuesta` | Bearer | Responder invitación de grupo. |
| `POST /grupos/{id}/gastos` | Bearer | Registrar gasto y cuotas. |
| `GET /grupos/{id}` | Bearer | Consultar detalle, gastos y balances. |
| `GET /ping` | Público | Comprobar disponibilidad del servidor. |

Los errores de entrada y dominio se traducen a respuestas HTTP; entre los códigos usados están 400, 401, 403, 404 y 409. Las credenciales de inicio de sesión incorrectas comparten un mensaje genérico. Las respuestas públicas de usuario no contienen contraseñas ni hashes.

Para regenerar el contrato desde los routers de FastAPI, con las dependencias del servidor instaladas:

```powershell
cd servidor
python generar_openapi.py
```

El script escribe el resultado en `openapi.yaml` en la raíz del repositorio.

## 5. Decisiones de diseño

Las decisiones y sus alternativas se registran en:

- [ADR-006 — PostgreSQL y persistencia social](adr/ADR-006-postgresql-y-persistencia-social.md): PostgreSQL 16, UUID v4 representados como `String(36)`, repositorios por responsabilidad y pruebas de integración contra PostgreSQL.
- [ADR-007 — Autenticación, hashing y sesiones](adr/ADR-007-autenticacion-hashing-y-sesiones.md): Argon2id, JWT HS256 de 60 minutos, UTC y almacenamiento seguro de la sesión en el cliente.
- [ADR-008 — Reparto determinista y balances](adr/ADR-008-reparto-determinista-y-balances.md): importes decimales, conservación del total, reparto reproducible del residuo y cálculo de balances.

## 6. Estrategia de pruebas

La suite está en `servidor/tests/`:

- **Dominio:** validaciones de usuarios, amistades y grupos; repartos y balances.
- **Propiedades:** Hypothesis genera importes y participantes para comprobar que las cuotas conservan el total, el residuo es determinista y los balances suman cero.
- **Aplicación:** casos de uso comprobados con dependencias simuladas.
- **Integración HTTP:** flujos de autenticación y de gastos compartidos, además de casos límite de amistades y grupos.
- **Persistencia:** las pruebas de API usan PostgreSQL real, no SQLite en memoria.
- **Contrato:** `servidor/test_correspondencia_openapi.py` comprueba `/ping` contra el esquema OpenAPI y requiere que la API esté ejecutándose en `localhost:8000`.

### Ejecutar las pruebas

Primero inicia PostgreSQL con Docker y prepara un entorno Python como se describe en el README. Las pruebas de integración vacían las tablas sociales al preparar y finalizar cada prueba; ejecútalas únicamente contra una base de datos descartable, nunca contra una base con datos que se quieran conservar.

En PowerShell, desde la raíz del repositorio, crea una base separada para pruebas (con los valores de desarrollo de `.env.example`):

```powershell
docker compose exec postgres createdb -U postgres cuentas_claras_test
```

Desde `servidor/`, activa el entorno virtual y configura la conexión a esa base:

```powershell
$env:DATABASE_URL = "postgresql+psycopg://postgres:cuentas_claras_local@localhost:5433/cuentas_claras_test"
python -m pytest tests -v
```

Para ejecutar además la prueba de contrato del endpoint `/ping`, deja levantado el backend en `localhost:8000` en otra terminal y ejecuta:

```powershell
python -m pytest test_correspondencia_openapi.py -v
```

## 7. Puesta en marcha y límites del despliegue local

`docker compose up --build -d` levanta la API FastAPI y PostgreSQL. El cliente de escritorio Tauri no corre dentro de ese Compose: se inicia en el host con `pnpm tauri dev`, desde `front_end/`. La base de datos conserva sus datos en el volumen `postgres_data` incluso al detener y recrear los contenedores.

La configuración incluida es para desarrollo local. Antes de exponer el servicio en una red o desplegarlo, configura secretos propios, habilita HTTPS/TLS y restringe los orígenes CORS.
