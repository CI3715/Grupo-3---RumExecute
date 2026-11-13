"""
Pruebas de Integración HTTP: The Auth Flow (Entrega 5).
Verifica:
- Registro de usuario (201 Created) y no divulgación de hashes.
- Control de unicidad de nombre de usuario y correo (409 Conflict).
- Validación de contraseñas cortas (400 Bad Request).
- Inicio de sesión (200 OK) con entrega de JWT HS256 y fecha UTC.
- Fallo genérico ante credenciales incorrectas (401 Unauthorized).
- Consulta protegida de perfil propio con Bearer token (200 OK y 401 sin token/alterado).
"""
import pytest


def test_auth_flow_completo(client):
    # 1. Registro exitoso
    datos_registro = {
        "nombre_usuario": "juanito",
        "nombre_visible": "Juan Pérez",
        "correo": "juan@example.com",
        "contrasena": "MiPasswordSeguro123"
    }
    resp = client.post("/usuarios", json=datos_registro)
    assert resp.status_code == 201
    usuario_creado = resp.json()
    assert usuario_creado["nombre_usuario"] == "juanito"
    assert usuario_creado["correo"] == "juan@example.com"
    # Invariante RNF-05: Nunca revelar hash ni password
    assert "password" not in usuario_creado
    assert "password_hash" not in usuario_creado

    # 2. Intentar registrar el mismo nombre de usuario -> 409 Conflict
    resp_duplicado_user = client.post("/usuarios", json={
        "nombre_usuario": "juanito",
        "nombre_visible": "Otro Juan",
        "correo": "otro@example.com",
        "contrasena": "MiPasswordSeguro123"
    })
    assert resp_duplicado_user.status_code == 409
    assert resp_duplicado_user.json()["detail"]["campo"] == "nombre_usuario"

    # 3. Intentar registrar el mismo correo en mayúsculas -> 409 Conflict
    resp_duplicado_correo = client.post("/usuarios", json={
        "nombre_usuario": "juanito2",
        "nombre_visible": "Otro Juan",
        "correo": "JUAN@EXAMPLE.COM",
        "contrasena": "MiPasswordSeguro123"
    })
    assert resp_duplicado_correo.status_code == 409
    assert resp_duplicado_correo.json()["detail"]["campo"] == "correo"

    # 4. Contraseña corta (<10 caracteres) -> 400 Bad Request
    resp_pass_corta = client.post("/usuarios", json={
        "nombre_usuario": "pedrito",
        "nombre_visible": "Pedro",
        "correo": "pedro@example.com",
        "contrasena": "corta123"
    })
    assert resp_pass_corta.status_code == 400
    assert resp_pass_corta.json()["detail"]["campo"] == "contrasena"

    # 5. Login con credenciales incorrectas -> 401 Unauthorized genérico
    resp_login_malo = client.post("/sesiones", json={
        "correo": "juan@example.com",
        "contrasena": "PasswordEquivocada"
    })
    assert resp_login_malo.status_code == 401
    assert resp_login_malo.json()["detail"] == "Correo o contraseña incorrectos."

    # 6. Login exitoso -> 200 OK con Token JWT
    resp_login = client.post("/sesiones", json={
        "correo": "juan@example.com",
        "contrasena": "MiPasswordSeguro123"
    })
    assert resp_login.status_code == 200
    datos_sesion = resp_login.json()
    assert "access_token" in datos_sesion
    assert datos_sesion["token_type"] == "bearer"
    assert datos_sesion["expires_at"].endswith("Z")  # Formato UTC 0 estricto

    token = datos_sesion["access_token"]

    # 7. Consultar perfil con Bearer Token -> 200 OK
    resp_perfil = client.get("/usuarios/yo", headers={"Authorization": f"Bearer {token}"})
    assert resp_perfil.status_code == 200
    perfil = resp_perfil.json()
    assert perfil["nombre_usuario"] == "juanito"
    assert perfil["correo"] == "juan@example.com"

    # 8. Consultar perfil sin token -> 401 Unauthorized
    resp_sin_token = client.get("/usuarios/yo")
    assert resp_sin_token.status_code == 401

    # 9. Consultar perfil con token alterado -> 401 Unauthorized
    resp_token_falso = client.get("/usuarios/yo", headers={"Authorization": "Bearer token_falsificado_123"})
    assert resp_token_falso.status_code == 401

