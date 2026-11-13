"""
Pruebas de Integración HTTP: The Split Flow (Entrega 6).
Verifica:
1. Búsqueda exacta de usuarios (200 OK) y 404 ante búsquedas parciales.
2. Invitaciones de amistad y aceptación mutua (INV-10).
3. Creación de grupos en moneda ISO e invitación de amigos (RF-COM-01, RF-COM-02).
4. División en partes iguales con conservación de céntimos y residuo alfabético (INV-07 / ESC-10).
5. Rechazo ante porcentajes que no suman 100% con cálculo exacto de faltante (400 Bad Request).
6. Suma de balances = 0.00 en todo momento (INV-08).
7. Aislamiento estricto: 404 a usuarios ajenos que consultan el grupo (ESC-08 / INV-17).
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


def test_split_flow_completo(client):
    # Crear 3 usuarios: Ana, Beto y Carla
    token_ana, id_ana = registrar_y_loguear(client, "ana", "Ana Gomez", "ana@test.com", "ClaveSegura123")
    token_beto, id_beto = registrar_y_loguear(client, "beto", "Beto Perez", "beto@test.com", "ClaveSegura123")
    token_carla, id_carla = registrar_y_loguear(client, "carla", "Carla Ruiz", "carla@test.com", "ClaveSegura123")

    # 1. Búsqueda exacta vs Búsqueda parcial (RF-SOC-04)
    # Búsqueda exacta -> 200 OK y nunca devuelve correo
    h_ana = {"Authorization": f"Bearer {token_ana}"}
    resp_busqueda = client.get("/usuarios?usuario=beto", headers=h_ana)
    assert resp_busqueda.status_code == 200
    assert resp_busqueda.json()["nombre_usuario"] == "beto"
    assert "correo" not in resp_busqueda.json()

    # Búsqueda parcial -> 404 Not Found
    resp_busqueda_parcial = client.get("/usuarios?usuario=be", headers=h_ana)
    assert resp_busqueda_parcial.status_code == 404

    # 2. Amistad: Ana invita a Beto y a Carla
    resp_inv_beto = client.post("/amistades", json={"nombre_usuario": "beto"}, headers=h_ana)
    assert resp_inv_beto.status_code == 201
    amistad_beto_id = resp_inv_beto.json()["id"]

    resp_inv_carla = client.post("/amistades", json={"nombre_usuario": "carla"}, headers=h_ana)
    assert resp_inv_carla.status_code == 201
    amistad_carla_id = resp_inv_carla.json()["id"]

    # Ambos aceptan desde sus sesiones (INV-10)
    h_beto = {"Authorization": f"Bearer {token_beto}"}
    client.post(f"/amistades/{amistad_beto_id}/respuesta", json={"aceptar": True}, headers=h_beto)

    h_carla = {"Authorization": f"Bearer {token_carla}"}
    client.post(f"/amistades/{amistad_carla_id}/respuesta", json={"aceptar": True}, headers=h_carla)

    # 3. Crear Grupo "Viaje a Mérida" en USD e invitar a Beto y Carla
    resp_grupo = client.post("/grupos", json={
        "nombre": "Viaje a Mérida",
        "moneda": "USD",
        "descripcion": "Vacaciones de marzo"
    }, headers=h_ana)
    assert resp_grupo.status_code == 201
    grupo_id = resp_grupo.json()["id"]

    # Ana invita a Beto y Carla
    client.post(f"/grupos/{grupo_id}/miembros", json={"nombre_usuario": "beto"}, headers=h_ana)
    client.post(f"/grupos/{grupo_id}/miembros", json={"nombre_usuario": "carla"}, headers=h_ana)

    # Beto y Carla aceptan la invitación
    client.post(f"/grupos/{grupo_id}/miembros/yo/respuesta", json={"aceptar": True}, headers=h_beto)
    client.post(f"/grupos/{grupo_id}/miembros/yo/respuesta", json={"aceptar": True}, headers=h_carla)

    # 4. Gasto 1: Partes iguales (100.00 entre Ana, Beto y Carla)
    # Ejemplo oficial de la especificación:
    # 100.00 entre 3 -> truncado 33.33 cada uno, residuo 0.01 asignado a 'ana' por orden alfabético
    resp_gasto1 = client.post(f"/grupos/{grupo_id}/gastos", json={
        "concepto": "Cena de bienvenida",
        "monto": 100.00,
        "pagador_id": id_ana,
        "regla_division": "IGUALES",
        "participantes": [id_ana, id_beto, id_carla]
    }, headers=h_ana)
    assert resp_gasto1.status_code == 201
    cuotas1 = {c["nombre_usuario"]: c for c in resp_gasto1.json()["cuotas"]}
    assert Decimal(str(cuotas1["ana"]["monto"])) == Decimal("33.34")
    assert cuotas1["ana"]["lleva_residuo"] is True
    assert Decimal(str(cuotas1["beto"]["monto"])) == Decimal("33.33")
    assert cuotas1["beto"]["lleva_residuo"] is False
    assert Decimal(str(cuotas1["carla"]["monto"])) == Decimal("33.33")
    assert cuotas1["carla"]["lleva_residuo"] is False

    # 5. Gasto 2: Porcentajes incorrectos (suman 95%) -> Rechazo 400
    resp_gasto_invalido = client.post(f"/grupos/{grupo_id}/gastos", json={
        "concepto": "Tour turístico",
        "monto": 60.00,
        "pagador_id": id_beto,
        "regla_division": "PORCENTAJES",
        "participantes": [id_ana, id_beto, id_carla],
        "valores": {id_ana: 50.0, id_beto: 25.0, id_carla: 20.0}  # Suma 95%
    }, headers=h_beto)
    assert resp_gasto_invalido.status_code == 400
    assert "faltan 5" in resp_gasto_invalido.json()["detail"]["mensaje"]

    # Corregir a 50% / 25% / 25% (Suma 100%) -> Exitoso
    resp_gasto_corregido = client.post(f"/grupos/{grupo_id}/gastos", json={
        "concepto": "Tour turístico",
        "monto": 60.00,
        "pagador_id": id_beto,
        "regla_division": "PORCENTAJES",
        "participantes": [id_ana, id_beto, id_carla],
        "valores": {id_ana: 50.0, id_beto: 25.0, id_carla: 25.0}
    }, headers=h_beto)
    assert resp_gasto_corregido.status_code == 201

    # 6. Balances del Grupo: La suma de balances de todos los miembros es exactamente 0.00 (INV-08)
    resp_detalle = client.get(f"/grupos/{grupo_id}", headers=h_ana)
    assert resp_detalle.status_code == 200
    balances = {m["nombre_usuario"]: Decimal(str(m["balance"])) for m in resp_detalle.json()["miembros"]}
    assert sum(balances.values()) == Decimal("0.00")
    # Valores exactos según la tabla del ejemplo oficial:
    # Ana: +36.66 | Beto: +11.67 | Carla: -48.33
    assert balances["ana"] == Decimal("36.66")
    assert balances["beto"] == Decimal("11.67")
    assert balances["carla"] == Decimal("-48.33")

    # 7. Aislamiento Estricto (ESC-08 / INV-17)
    # Un cuarto usuario que NO es miembro intenta consultar el grupo
    token_ajeno, _ = registrar_y_loguear(client, "daniel", "Daniel Perez", "daniel@test.com", "ClaveSegura123")
    h_ajeno = {"Authorization": f"Bearer {token_ajeno}"}

    resp_ajeno_ver = client.get(f"/grupos/{grupo_id}", headers=h_ajeno)
    assert resp_ajeno_ver.status_code == 404
    assert resp_ajeno_ver.json()["detail"] == "Grupo no encontrado."

    resp_ajeno_gasto = client.post(f"/grupos/{grupo_id}/gastos", json={
        "concepto": "Intruso",
        "monto": 10.0,
        "pagador_id": id_ana,
        "regla_division": "IGUALES",
        "participantes": [id_ana]
    }, headers=h_ajeno)
    assert resp_ajeno_gasto.status_code == 404
