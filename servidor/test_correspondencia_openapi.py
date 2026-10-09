from datetime import datetime
import httpx
from jsonschema import validate, ValidationError
import yaml

with open("../openapi.yaml", "r", encoding="utf-8") as f:
    contract = yaml.safe_load(f)

schema_ping = contract["components"]["schemas"]["PingResponse"]


def test_ping_openapi():
    # Petición al backend
    with httpx.Client(base_url="http://localhost:8000") as client:
        response = client.get("/ping")

    assert response.status_code == 200

    data = response.json()

    # Validar campos, tipos y propiedades requeridas
    try:
        validate(instance=data, schema=schema_ping)
    except ValidationError as e:
        assert(
            False
            ), f"La respuesta del backend no cumple con openapi.yaml: {e.message}"

    # Validar formato ISO-8601 del timestamp
    datetime.fromisoformat(data["timestamp"].replace("Z", "+00:00"))
