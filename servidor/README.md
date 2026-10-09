# Servidor Backend - Cuentas Claras

Modulo de servidor backend HTTP para la coordinacion social de la aplicacion Cuentas Claras, desarrollado con Python 3.11+ y FastAPI.

## Principio de Privacidad

Segun la arquitectura del sistema, el servidor backend unicamente coordina identidades, amistades y grupos de gastos compartidos. Nunca almacena ni recibe cuentas bancarias, saldos, categorias personales ni movimientos financieros privados del usuario.

## Requisitos Previos

* Python 3.11 o superior.
* Gestor de paquetes `pip` y soporte para entornos virtuales (`venv`).

## Instalacion y Configuracion

Desde este directorio (`servidor/`):

1. Crear un entorno virtual:
   ```bash
   python -m venv .venv
   ```

2. Activar el entorno virtual:
   * En Windows (PowerShell):
     ```powershell
     .venv\Scripts\Activate.ps1
     ```
   * En Linux / macOS:
     ```bash
     source .venv/bin/activate
     ```

3. Instalar dependencias requeridas:
   ```bash
   pip install -r requirements.txt
   ```

## Ejecucion en Modo Desarrollo

Iniciar el servidor con recarga automatica mediante Uvicorn:

```bash
uvicorn main:app --reload --port 8000
```

* Servidor disponible en: `http://localhost:8000`
* Documentacion Swagger interactiva: `http://localhost:8000/docs`
* Documentacion ReDoc: `http://localhost:8000/redoc`

## Endpoints Disponibles

### GET /ping
Endpoint basico para verificacion de estado y calculo de latencia (The Ping Flow).

* **Respuesta exitosa (200 OK):**
  ```json
  {
    "estado": "ok",
    "mensaje": "Servidor Cuentas Claras activo",
    "version": "0.1.0",
    "timestamp": "2026-10-08T19:30:00Z"
  }
  ```

## Pruebas de Contrato (OpenAPI)
El servidor incluye pruebas automatizadas para comprobar que la respuesta del endpoint '/ping' cumpla estrictamente con la especificación y tipos de datos definidos en 'openapi.yaml'.

### Ejecutar las pruebas
Iniciar el servidor en modo desarrollo en una terminal
```bash
uvicorn main:app --reload --port 8000
```

En otra terminal activar el entorno virtual

   * En Windows (PowerShell):
     ```powershell
     .venv\Scripts\Activate.ps1
     ```
   * En Linux / macOS:
     ```bash
     source .venv/bin/activate
     ```

Correr el test con pytest
```bash
pytest test_correspondencia_openapi.py -v
```

