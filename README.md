# Cuentas Claras — Grupo 3 (RumExecute)

Proyecto desarrollado para la asignatura **CI-3715: Ingeniería de Software** en la **Universidad Simón Bolívar (USB)**.

---

## 👥 Integrantes del Equipo

* **Santiago Armando Bueno Utrera** — Carnet: 20-10168
* **Víctor Manuel Hernández Avilán** — Carnet: 20-10349
* **Daniel Eduardo Quijada Malavé** — Carnet: 20-10518
* **David Antonio Paulo Medina** — Carnet: 21-10475
* **Jean Carlos Sifontes Palacios** — Carnet: 22-10387

---

## 📌 Descripción del Producto

**Cuentas Claras** es una aplicación de finanzas personales y gestión de gastos compartidos con soporte multimoneda (USD / Bs).

* **Aplicación Cliente:** Almacena de manera privada y local los movimientos y cuentas financieras del usuario en una base de datos SQLite embebida.
* **Servidor Social:** Coordina identidades, amistades y grupos con gastos compartidos sin recibir datos financieros personales.

---

## 🛠️ Stack Tecnológico

* **Cliente:** Tauri 2 (Rust) + Next.js (Exportación estática).
* **Servidor Backend:** Python 3.11+ con FastAPI y Uvicorn.
* **Contrato API:** OpenAPI 3.1 (`openapi.yaml`).

---

## 📁 Estructura del Proyecto (Monorepo)

```text
Grupo-3---RumExecute/
├── app/            # Aplicación Cliente (Tauri 2 + Next.js)
├── servidor/       # Servidor Backend HTTP (Python / FastAPI)
├── openapi.yaml    # Contrato formal de la API (Endpoint /ping)
├── LICENSE         # Licencia Apache-2.0
├── .gitignore      # Exclusiones de Git
└── README.md       # Este documento
```

---

## 🚀 Instrucciones Básicas de Ejecución (Modo Desarrollo)

*(Estas instrucciones se irán acomodando y detallando a medida que se avance en la implementación)*.

### 1. Servidor Backend (`servidor/`)
```bash
cd servidor
python -m venv .venv

# Activar entorno virtual:
# En Windows (PowerShell): .venv\Scripts\Activate.ps1
# En Linux/macOS: source .venv/bin/activate

pip install -r requirements.txt
uvicorn main:app --reload --port 8000
```
* Servidor accesible en: `http://localhost:8000`
* Endpoint de prueba: `GET http://localhost:8000/ping`

### 2. Aplicación Cliente (`app/`)
```bash
cd app
npm install
npm run tauri dev
```
* Abre la ventana de escritorio nativa y permite verificar la conectividad con el servidor mediante el botón **"Verificar Conexión con el Servidor"**.

---

## 📄 Licencia

Este proyecto está bajo la Licencia **Apache 2.0**. Consulte el archivo [LICENSE](LICENSE) para más detalles.
