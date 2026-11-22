"""
Pruebas de Integración HTTP: Sincronización e Idempotencia (Entrega 7 - Subtarea 3.2).
Verifica:
1. GET /cambios sin token -> 401 Unauthorized.
2. GET /cambios con cursor negativo o inválido -> 400 Bad Request.
3. POST /grupos/{id}/gastos con header Idempotency-Key:
   - 201 Created en la primera llamada.
   - 200 OK en reintento con la misma clave y mismos datos (sin duplicar gasto ni eventos).
   - 409 Conflict si se reutiliza la clave con datos distintos (RNF-17 / ESC-09).
4. GET /cambios?desde=<cursor>:
   - Aislamiento estricto por participante (INV-17): un miembro no incluido en el reparto no recibe eventos.
   - Avance correcto del cursor (nuevo_cursor) y paginación.
   - Reinicio del cursor a 0 devuelve los eventos históricos sin alterar nada.
"""
import pytest
from decimal import Decimal


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


def crear_grupo_con_miembros(client, token_admin, nombre_grupo, amigos):
    h = {"Authorization": f"Bearer {token_admin}"}

    # Para invitar miembros a un grupo, deben ser amigos previamente
    for token_amigo, username_amigo in amigos:
        resp_inv = client.post("/amistades", json={"nombre_usuario": username_amigo}, headers=h)
        assert resp_inv.status_code == 201
        amistad_id = resp_inv.json()["id"]
        resp_resp = client.post(
            f"/amistades/{amistad_id}/respuesta",
            json={"aceptar": True},
            headers={"Authorization": f"Bearer {token_amigo}"}
        )
        assert resp_resp.status_code == 200

    resp_g = client.post("/grupos", json={
        "nombre": nombre_grupo,
        "moneda": "USD",
        "descripcion": "Grupo de prueba"
    }, headers=h)
    assert resp_g.status_code == 201
    grupo_id = resp_g.json()["id"]

    for token_amigo, username_amigo in amigos:
        # Invitar al amigo al grupo
        resp_inv_g = client.post(f"/grupos/{grupo_id}/miembros", json={"nombre_usuario": username_amigo}, headers=h)
        assert resp_inv_g.status_code == 201
        # Aceptar la invitacion de grupo
        resp_resp_g = client.post(
            f"/grupos/{grupo_id}/miembros/yo/respuesta",
            json={"aceptar": True},
            headers={"Authorization": f"Bearer {token_amigo}"}
        )
        assert resp_resp_g.status_code == 200

    return grupo_id


def test_get_cambios_requiere_autenticacion(client):
    # Sin header Authorization -> 401
    resp = client.get("/cambios?desde=0")
    assert resp.status_code == 401


def test_get_cambios_cursor_invalido_devuelve_400(client):
    token_ana, _ = registrar_y_loguear(client, "ana_cursor", "Ana C", "ana_c@test.com", "ClaveSegura123")
    h = {"Authorization": f"Bearer {token_ana}"}

    # Cursor negativo -> 400
    resp_neg = client.get("/cambios?desde=-1", headers=h)
    assert resp_neg.status_code == 400

    # Cursor no numérico -> 400
    resp_str = client.get("/cambios?desde=invalido", headers=h)
    assert resp_str.status_code == 400


def test_post_gasto_con_idempotency_key_flujo_completo(client):
    token_ana, id_ana = registrar_y_loguear(client, "ana_idemp", "Ana I", "ana_i@test.com", "ClaveSegura123")
    token_beto, id_beto = registrar_y_loguear(client, "beto_idemp", "Beto I", "beto_i@test.com", "ClaveSegura123")

    grupo_id = crear_grupo_con_miembros(client, token_ana, "Viaje Idemp", [(token_beto, "beto_idemp")])
    h_ana = {"Authorization": f"Bearer {token_ana}"}

    payload_gasto = {
        "concepto": "Hotel Mérida",
        "monto": 100.00,
        "pagador_id": id_ana,
        "regla_division": "IGUALES",
        "participantes": [id_ana, id_beto],
    }
    clave_idemp = "idemp-hotel-001"

    # 1. Primera petición: nuevo gasto con Idempotency-Key -> 201 Created
    headers_primera = {**h_ana, "Idempotency-Key": clave_idemp}
    resp1 = client.post(f"/grupos/{grupo_id}/gastos", json=payload_gasto, headers=headers_primera)
    assert resp1.status_code == 201
    gasto_id = resp1.json()["id"]
    assert gasto_id is not None
    assert resp1.json()["concepto"] == "Hotel Mérida"

    # 2. Reintento con la MISMA Idempotency-Key y MISMOS datos -> 200 OK (no 201)
    resp2 = client.post(f"/grupos/{grupo_id}/gastos", json=payload_gasto, headers=headers_primera)
    assert resp2.status_code == 200
    assert resp2.json()["id"] == gasto_id
    assert resp2.json()["concepto"] == "Hotel Mérida"

    # Verificar que el grupo sigue teniendo SOLO 1 gasto
    resp_det = client.get(f"/grupos/{grupo_id}", headers=h_ana)
    assert resp_det.status_code == 200
    assert len(resp_det.json()["gastos"]) == 1

    # 3. Petición con la MISMA Idempotency-Key pero DISTINTO monto -> 409 Conflict
    payload_conflicto = {**payload_gasto, "monto": 150.00}
    resp3 = client.post(f"/grupos/{grupo_id}/gastos", json=payload_conflicto, headers=headers_primera)
    assert resp3.status_code == 409


def test_get_cambios_aislamiento_estricto_y_cursores(client):
    """
    Invariante 17 (INV-17):
    Ana, Beto y Carla están en el grupo.
    Ana registra un gasto dividido ÚNICAMENTE entre Ana y Beto (Carla excluida).
    Ana y Beto deben recibir exactamente 1 evento de cambio cada uno.
    Carla NO debe recibir ningún evento de cambio.
    """
    token_ana, id_ana = registrar_y_loguear(client, "ana_sync", "Ana S", "ana_s@test.com", "ClaveSegura123")
    token_beto, id_beto = registrar_y_loguear(client, "beto_sync", "Beto S", "beto_s@test.com", "ClaveSegura123")
    token_carla, id_carla = registrar_y_loguear(client, "carla_sync", "Carla S", "carla_s@test.com", "ClaveSegura123")

    grupo_id = crear_grupo_con_miembros(
        client,
        token_admin=token_ana,
        nombre_grupo="Viaje a Mérida",
        amigos=[(token_beto, "beto_sync"), (token_carla, "carla_sync")]
    )

    h_ana = {"Authorization": f"Bearer {token_ana}"}
    h_beto = {"Authorization": f"Bearer {token_beto}"}
    h_carla = {"Authorization": f"Bearer {token_carla}"}

    # Ana registra un gasto de 100.00 solo entre Ana y Beto
    resp_gasto = client.post(f"/grupos/{grupo_id}/gastos", json={
        "concepto": "Hotel",
        "monto": 100.00,
        "pagador_id": id_ana,
        "regla_division": "IGUALES",
        "participantes": [id_ana, id_beto],
    }, headers=h_ana)
    assert resp_gasto.status_code == 201

    # Ana consulta sus cambios desde cursor 0
    resp_cambios_ana = client.get("/cambios?desde=0", headers=h_ana)
    assert resp_cambios_ana.status_code == 200
    datos_ana = resp_cambios_ana.json()
    assert len(datos_ana["eventos"]) == 1
    ev_ana = datos_ana["eventos"][0]
    assert ev_ana["usuario_id"] == id_ana
    assert ev_ana["nombre_grupo"] == "Viaje a Mérida"
    assert ev_ana["concepto"] == "Hotel"
    assert Decimal(str(ev_ana["monto_parte"])) == Decimal("50.00")
    assert datos_ana["quedan_mas"] is False
    cursor_ana = datos_ana["nuevo_cursor"]
    assert cursor_ana > 0

    # Beto consulta sus cambios desde cursor 0
    resp_cambios_beto = client.get("/cambios?desde=0", headers=h_beto)
    assert resp_cambios_beto.status_code == 200
    datos_beto = resp_cambios_beto.json()
    assert len(datos_beto["eventos"]) == 1
    ev_beto = datos_beto["eventos"][0]
    assert ev_beto["usuario_id"] == id_beto
    assert ev_beto["nombre_grupo"] == "Viaje a Mérida"
    assert Decimal(str(ev_beto["monto_parte"])) == Decimal("50.00")

    # Carla consulta sus cambios desde cursor 0 (INV-17: Carla fue excluida de la división)
    resp_cambios_carla = client.get("/cambios?desde=0", headers=h_carla)
    assert resp_cambios_carla.status_code == 200
    datos_carla = resp_cambios_carla.json()
    assert len(datos_carla["eventos"]) == 0
    assert datos_carla["nuevo_cursor"] == 0
    assert datos_carla["quedan_mas"] is False

    # Ana consulta de nuevo desde su nuevo_cursor -> 0 eventos
    resp_ana_siguiente = client.get(f"/cambios?desde={cursor_ana}", headers=h_ana)
    assert resp_ana_siguiente.status_code == 200
    assert len(resp_ana_siguiente.json()["eventos"]) == 0

    # Reiniciar cursor a 0 (Paso 3 de la demostración):
    # Ana vuelve a consultar desde 0 y recupera el mismo evento sin duplicaciones ni errores
    resp_ana_reinicio = client.get("/cambios?desde=0", headers=h_ana)
    assert resp_ana_reinicio.status_code == 200
    assert len(resp_ana_reinicio.json()["eventos"]) == 1
    assert resp_ana_reinicio.json()["eventos"][0]["id"] == ev_ana["id"]
