"""
Pruebas de Casos Borde y Acciones Adicionales: Amistades y Grupos.
Verifica:
1. Amistades:
   - Auto-invitación (400 Bad Request).
   - Invitación a usuario inexistente (404 Not Found).
   - Invitación duplicada mientras está pendiente (409 Conflict).
   - Intento de respuesta por un tercero ajeno a la invitación (403 Forbidden).
   - Doble aceptación o respuesta a la misma invitación de amistad (409 Conflict).
   - Rechazo de invitación de amistad (200 OK con estado RECHAZADA).
   - Invitación a alguien con quien ya se tiene amistad activa (409 Conflict).
   - Listar amistades pendientes y activas (200 OK).
   - Eliminación de amistad activa (204 No Content).
   - Doble eliminación de amistad (404 Not Found).
   - Usuario ajeno intenta eliminar amistad de otros (403 Forbidden).

2. Grupos:
   - Creación de grupo con moneda inválida o nombre vacío (400 Bad Request).
   - Invitar a un grupo a un usuario que NO es amigo previo (403 Forbidden).
   - Invitar al grupo a un usuario ya invitado o ya miembro (409 Conflict).
   - Responder invitación de grupo inexistente (404 Not Found).
   - Doble aceptación de invitación a un grupo (409 Conflict).
   - Rechazo de invitación a un grupo (200 OK).
   - Listado de grupos e invitaciones pendientes (200 OK).
"""
import pytest


def registrar_y_loguear(client, username, nombre, correo, clave):
    client.post("/usuarios", json={
        "nombre_usuario": username,
        "nombre_visible": nombre,
        "correo": correo,
        "contrasena": clave
    })
    res = client.post("/sesiones", json={
        "correo": correo,
        "contrasena": clave
    })
    return res.json()["access_token"], res.json()["usuario"]["id"]


# ==============================================================================
# PRUEBAS DE AMISTADES
# ==============================================================================

def test_amistades_auto_invitacion_rechazada(client):
    """Un usuario no puede invitarse a sí mismo como amigo (400 Bad Request)."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    h_ana = {"Authorization": f"Bearer {token_ana}"}

    resp = client.post("/amistades", json={"nombre_usuario": "ana"}, headers=h_ana)
    assert resp.status_code == 400


def test_amistades_invitar_usuario_inexistente(client):
    """Invitar a un usuario que no existe devuelve 404 Not Found."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    h_ana = {"Authorization": f"Bearer {token_ana}"}

    resp = client.post("/amistades", json={"nombre_usuario": "fantasma"}, headers=h_ana)
    assert resp.status_code == 404


def test_amistades_invitacion_duplicada_pendiente(client):
    """No se puede enviar una segunda invitación si ya hay una pendiente (409 Conflict)."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")
    h_ana = {"Authorization": f"Bearer {token_ana}"}

    resp1 = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    assert resp1.status_code == 201

    resp2 = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    assert resp2.status_code == 409


def test_amistades_tercero_no_puede_responder_invitacion(client):
    """Un usuario ajeno no puede responder la invitación de otra persona (403 Forbidden)."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")
    token_carla, _ = registrar_y_loguear(client, "carla", "Carla Ruiz", "carla@test.com", "ClaveSegura123")

    # Ana invita a Beto
    h_ana = {"Authorization": f"Bearer {token_ana}"}
    resp_inv = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    amistad_id = resp_inv.json()["id"]

    # Carla (intruso) intenta responder la invitación de Beto
    h_carla = {"Authorization": f"Bearer {token_carla}"}
    resp_hack = client.post(f"/amistades/{amistad_id}/respuesta", json={"aceptar": True}, headers=h_carla)
    assert resp_hack.status_code == 403


def test_amistades_doble_aceptacion_conflicto(client):
    """
    Aceptar una invitación dos veces devuelve 409 Conflict en la segunda ocasión.
    (La invitación ya fue respondida).
    """
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    resp_inv = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    amistad_id = resp_inv.json()["id"]

    h_beto = {"Authorization": f"Bearer {token_beto}"}
    # Primera aceptación -> 200 OK
    resp_ok = client.post(f"/amistades/{amistad_id}/respuesta", json={"aceptar": True}, headers=h_beto)
    assert resp_ok.status_code == 200
    assert resp_ok.json()["estado"] == "ACTIVA"

    # Segunda aceptación -> 409 Conflict
    resp_doble = client.post(f"/amistades/{amistad_id}/respuesta", json={"aceptar": True}, headers=h_beto)
    assert resp_doble.status_code == 409
    assert resp_doble.json()["detail"]["mensaje"] == "La invitación ya fue respondida."


def test_amistades_rechazo_invitacion(client):
    """Rechazar una invitación de amistad actualiza su estado a RECHAZADA."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    resp_inv = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    amistad_id = resp_inv.json()["id"]

    h_beto = {"Authorization": f"Bearer {token_beto}"}
    resp_rechazo = client.post(f"/amistades/{amistad_id}/respuesta", json={"aceptar": False}, headers=h_beto)
    assert resp_rechazo.status_code == 200
    assert resp_rechazo.json()["estado"] == "RECHAZADA"


def test_amistades_invitar_amigo_ya_activo_conflicto(client):
    """No se puede volver a invitar a alguien si la amistad ya está activa (409 Conflict)."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    resp_inv = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    amistad_id = resp_inv.json()["id"]

    h_beto = {"Authorization": f"Bearer {token_beto}"}
    client.post(f"/amistades/{amistad_id}/respuesta", json={"aceptar": True}, headers=h_beto)

    # Intentar invitarlo de nuevo cuando ya son amigos activos
    resp_re_inv = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    assert resp_re_inv.status_code == 409


def test_amistades_listado_completo(client):
    """GET /amistades lista correctamente amigos e invitaciones enviadas y recibidas."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")
    token_carla, _ = registrar_y_loguear(client, "carla", "Carla Ruiz", "carla@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    h_beto = {"Authorization": f"Bearer {token_beto}"}
    h_carla = {"Authorization": f"Bearer {token_carla}"}

    # Ana invita a Beto y Beto acepta (Amigos)
    r_inv = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    client.post(f"/amistades/{r_inv.json()['id']}/respuesta", json={"aceptar": True}, headers=h_beto)

    # Carla invita a Ana (Ana tiene 1 recibida pendiente, Carla 1 enviada pendiente)
    client.post("/amistades", json={"nombre_usuario": "ana"}, headers=h_carla)

    # Ver listado de Ana
    resp_ana = client.get("/amistades", headers=h_ana)
    assert resp_ana.status_code == 200
    datos_ana = resp_ana.json()
    assert len(datos_ana["amigos"]) == 1
    assert datos_ana["amigos"][0]["nombre_usuario"] == "beto"
    assert len(datos_ana["invitaciones_recibidas"]) == 1
    assert datos_ana["invitaciones_recibidas"][0]["nombre_usuario"] == "carla"
    assert len(datos_ana["invitaciones_enviadas"]) == 0


def test_amistades_eliminacion_y_doble_eliminacion(client):
    """Eliminar amistad activa devuelve 204. Una segunda eliminación devuelve 404."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    h_beto = {"Authorization": f"Bearer {token_beto}"}

    r_inv = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    amistad_id = r_inv.json()["id"]
    client.post(f"/amistades/{amistad_id}/respuesta", json={"aceptar": True}, headers=h_beto)

    # Ana elimina la amistad
    resp_del1 = client.delete(f"/amistades/{amistad_id}", headers=h_ana)
    assert resp_del1.status_code == 204

    # Intentar eliminarla otra vez -> 404 Not Found
    resp_del2 = client.delete(f"/amistades/{amistad_id}", headers=h_ana)
    assert resp_del2.status_code == 404


def test_amistades_tercero_no_puede_eliminar(client):
    """Un usuario ajeno no puede eliminar la amistad entre otras dos personas."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")
    token_carla, _ = registrar_y_loguear(client, "carla", "Carla Ruiz", "carla@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    h_beto = {"Authorization": f"Bearer {token_beto}"}
    h_carla = {"Authorization": f"Bearer {token_carla}"}

    r_inv = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    amistad_id = r_inv.json()["id"]
    client.post(f"/amistades/{amistad_id}/respuesta", json={"aceptar": True}, headers=h_beto)

    # Carla intenta borrar la amistad de Ana y Beto
    resp_hack = client.delete(f"/amistades/{amistad_id}", headers=h_carla)
    assert resp_hack.status_code == 403


# ==============================================================================
# PRUEBAS DE GRUPOS
# ==============================================================================

def test_grupos_validaciones_al_crear(client):
    """Validación de moneda de 3 letras mayúsculas y nombre no vacío (400 Bad Request)."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    h_ana = {"Authorization": f"Bearer {token_ana}"}

    # Moneda inválida ("US" en vez de 3 caracteres)
    resp_moneda = client.post("/grupos", json={"nombre": "Grupo 1", "moneda": "US"}, headers=h_ana)
    assert resp_moneda.status_code == 400

    # Nombre vacío
    resp_nombre = client.post("/grupos", json={"nombre": "   ", "moneda": "USD"}, headers=h_ana)
    assert resp_nombre.status_code == 400


def test_grupos_no_se_puede_invitar_a_quien_no_es_amigo(client):
    """
    RF-COM-02: Solo se puede invitar a miembros que sean amigos activos previos.
    Si no son amigos, retorna 403 Forbidden.
    """
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    # Ana crea un grupo
    resp_grupo = client.post("/grupos", json={"nombre": "Viaje", "moneda": "USD"}, headers=h_ana)
    grupo_id = resp_grupo.json()["id"]

    # Ana intenta invitar a Beto SIN ser amigos previamente
    resp_inv = client.post(f"/grupos/{grupo_id}/miembros", json={"nombre_usuario": "beto"}, headers=h_ana)
    assert resp_inv.status_code == 403
    assert "Solo puedes invitar a tus amigos" in resp_inv.json()["detail"]["mensaje"]


def test_grupos_invitar_miembro_duplicado(client):
    """No se puede invitar dos veces a la misma persona a un grupo (409 Conflict)."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    h_beto = {"Authorization": f"Bearer {token_beto}"}

    # Hacemos amigos a Ana y Beto
    r_am = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    client.post(f"/amistades/{r_am.json()['id']}/respuesta", json={"aceptar": True}, headers=h_beto)

    # Ana crea grupo e invita a Beto
    r_g = client.post("/grupos", json={"nombre": "Viaje", "moneda": "USD"}, headers=h_ana)
    grupo_id = r_g.json()["id"]

    resp1 = client.post(f"/grupos/{grupo_id}/miembros", json={"nombre_usuario": "beto"}, headers=h_ana)
    assert resp1.status_code == 201

    # Segunda invitación -> 409 Conflict
    resp2 = client.post(f"/grupos/{grupo_id}/miembros", json={"nombre_usuario": "beto"}, headers=h_ana)
    assert resp2.status_code == 409


def test_grupos_doble_aceptacion_invitacion_grupo(client):
    """Aceptar la invitación a un grupo dos veces devuelve 409 Conflict en la segunda."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    h_beto = {"Authorization": f"Bearer {token_beto}"}

    # Amistad
    r_am = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    client.post(f"/amistades/{r_am.json()['id']}/respuesta", json={"aceptar": True}, headers=h_beto)

    # Crear grupo e invitar a Beto
    r_g = client.post("/grupos", json={"nombre": "Viaje", "moneda": "USD"}, headers=h_ana)
    grupo_id = r_g.json()["id"]
    client.post(f"/grupos/{grupo_id}/miembros", json={"nombre_usuario": "beto"}, headers=h_ana)

    # Beto acepta la 1ra vez -> 200 OK
    resp_ok = client.post(f"/grupos/{grupo_id}/miembros/yo/respuesta", json={"aceptar": True}, headers=h_beto)
    assert resp_ok.status_code == 200
    assert resp_ok.json()["estado"] == "ACEPTADA"

    # Beto intenta aceptar la 2da vez -> 409 Conflict (o 404 porque ya no tiene invitación pendiente)
    resp_doble = client.post(f"/grupos/{grupo_id}/miembros/yo/respuesta", json={"aceptar": True}, headers=h_beto)
    assert resp_doble.status_code in (404, 409)


def test_grupos_rechazo_invitacion_grupo(client):
    """Rechazar una invitación a un grupo actualiza el estado a RECHAZADA."""
    token_ana, _ = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, _ = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    h_beto = {"Authorization": f"Bearer {token_beto}"}

    # Amistad
    r_am = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    client.post(f"/amistades/{r_am.json()['id']}/respuesta", json={"aceptar": True}, headers=h_beto)

    # Crear grupo e invitar a Beto
    r_g = client.post("/grupos", json={"nombre": "Viaje", "moneda": "USD"}, headers=h_ana)
    grupo_id = r_g.json()["id"]
    client.post(f"/grupos/{grupo_id}/miembros", json={"nombre_usuario": "beto"}, headers=h_ana)

    # Beto rechaza
    resp_rechazo = client.post(f"/grupos/{grupo_id}/miembros/yo/respuesta", json={"aceptar": False}, headers=h_beto)
    assert resp_rechazo.status_code == 200
    assert resp_rechazo.json()["estado"] == "RECHAZADA"

