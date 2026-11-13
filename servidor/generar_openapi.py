"""
Script utilitario para exportar el esquema OpenAPI generado por FastAPI a openapi.yaml.
Garantiza que la documentación y contrato de la API siempre coincidan con las rutas del código.
"""
import sys
import os
import yaml

# Agregar directorio del servidor al path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from main import app


def exportar_openapi():
    spec = app.openapi()
    spec["openapi"] = "3.1.0"
    spec["servers"] = [
        {"url": "http://localhost:8000", "description": "Servidor local de desarrollo"}
    ]

    destino = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "openapi.yaml"))
    with open(destino, "w", encoding="utf-8") as f:
        yaml.dump(spec, f, sort_keys=False, allow_unicode=True)
    print(f"Esquema OpenAPI exportado exitosamente a {destino}")


if __name__ == "__main__":
    exportar_openapi()
