from dataclasses import replace
from datetime import datetime, timezone
from decimal import Decimal as D
from unittest.mock import Mock

import pytest

from aplicacion.dtos.cambios_respuesta import EventoCambioRespuesta
from aplicacion.errores import ErrorAplicacion
from aplicacion.obtener_cambios_usuario import ObtenerCambiosUsuario
from dominio.errores import ErrorDominio
from dominio.evento_cambio import EventoCambioUsuario


def evento(posicion: int, usuario_id: str = "usuario-1") -> EventoCambioUsuario:
    return EventoCambioUsuario(
        id=f"evento-{posicion}",
        posicion=posicion,
        usuario_id=usuario_id,
        grupo_id="grupo-1",
        nombre_grupo="Viaje",
        gasto_id=f"gasto-{posicion}",
        concepto="Cena",
        fecha=datetime(2026, 11, 20, 12, 0, tzinfo=timezone.utc),
        moneda="USD",
        monto_parte=D("3.33"),
        incluye_redondeo=False,
    )


@pytest.fixture
def repo_eventos():
    repo = Mock()
    repo.listar_posteriores.return_value = []
    return repo


@pytest.fixture
def caso_uso(repo_eventos):
    return ObtenerCambiosUsuario(repo_eventos)


def test_consulta_solo_los_eventos_del_usuario_pidiendo_un_evento_extra(
    caso_uso, repo_eventos
):
    caso_uso.ejecutar("usuario-1", "0")

    repo_eventos.listar_posteriores.assert_called_once_with("usuario-1", 0, 101)


def test_devuelve_eventos_en_orden_y_el_nuevo_cursor(caso_uso, repo_eventos):
    repo_eventos.listar_posteriores.return_value = [evento(3), evento(4)]

    respuesta = caso_uso.ejecutar("usuario-1", "2")

    assert [e.posicion for e in respuesta.eventos] == [3, 4]
    assert respuesta.nuevo_cursor == 4
    assert respuesta.quedan_mas is False


def test_con_mas_de_cien_eventos_devuelve_cien_y_avisa_que_quedan_mas(
    caso_uso, repo_eventos
):
    repo_eventos.listar_posteriores.return_value = [
        evento(i) for i in range(1, 102)
    ]

    respuesta = caso_uso.ejecutar("usuario-1", "0")

    assert len(respuesta.eventos) == 100
    assert respuesta.nuevo_cursor == 100
    assert respuesta.quedan_mas is True


def test_con_exactamente_cien_eventos_devuelve_cien_y_quedan_mas_es_falso(
    caso_uso, repo_eventos
):
    repo_eventos.listar_posteriores.return_value = [
        evento(i) for i in range(1, 101)
    ]

    respuesta = caso_uso.ejecutar("usuario-1", "0")

    assert len(respuesta.eventos) == 100
    assert respuesta.nuevo_cursor == 100
    assert respuesta.quedan_mas is False


def test_sin_eventos_nuevos_conserva_el_cursor(caso_uso):
    respuesta = caso_uso.ejecutar("usuario-1", "7")

    assert respuesta.eventos == ()
    assert respuesta.nuevo_cursor == 7
    assert respuesta.quedan_mas is False


def test_desde_como_entero_valido_es_aceptado(caso_uso, repo_eventos):
    caso_uso.ejecutar("usuario-1", 5)

    repo_eventos.listar_posteriores.assert_called_once_with("usuario-1", 5, 101)


def test_mapea_todos_los_campos_al_dto_de_respuesta(caso_uso, repo_eventos):
    ev_dominio = EventoCambioUsuario(
        id="evt-123",
        posicion=10,
        usuario_id="usuario-1",
        grupo_id="grupo-viaje",
        nombre_grupo="Viaje a Mérida",
        gasto_id="gasto-hotel",
        concepto="Hotel",
        fecha=datetime(2026, 11, 22, 14, 30, tzinfo=timezone.utc),
        moneda="USD",
        monto_parte=D("33.33"),
        incluye_redondeo=True,
    )
    repo_eventos.listar_posteriores.return_value = [ev_dominio]

    respuesta = caso_uso.ejecutar("usuario-1", "0")

    assert len(respuesta.eventos) == 1
    dto = respuesta.eventos[0]
    assert isinstance(dto, EventoCambioRespuesta)
    assert dto.id == "evt-123"
    assert dto.posicion == 10
    assert dto.usuario_id == "usuario-1"
    assert dto.grupo_id == "grupo-viaje"
    assert dto.nombre_grupo == "Viaje a Mérida"
    assert dto.gasto_id == "gasto-hotel"
    assert dto.concepto == "Hotel"
    assert dto.fecha == datetime(2026, 11, 22, 14, 30, tzinfo=timezone.utc)
    assert dto.moneda == "USD"
    assert dto.monto_parte == D("33.33")
    assert dto.incluye_redondeo is True


def test_evento_sin_posicion_es_un_estado_inconsistente(caso_uso, repo_eventos):
    repo_eventos.listar_posteriores.return_value = [
        replace(evento(1), posicion=None)
    ]

    with pytest.raises(ErrorAplicacion) as exc:
        caso_uso.ejecutar("usuario-1", "0")

    assert exc.value.campo == "posicion"


@pytest.mark.parametrize(
    "desde",
    ["-1", "abc", "", "1.5", " 5 ", "1_0", "+5", "٣", None, True, False, -5, 1.5],
)
def test_desde_invalido_lanza_error_y_no_consulta(caso_uso, repo_eventos, desde):
    with pytest.raises(ErrorDominio) as exc:
        caso_uso.ejecutar("usuario-1", desde)

    assert exc.value.campo == "desde"
    repo_eventos.listar_posteriores.assert_not_called()


def test_cursor_negativo_lanza_error_dominio():
    repo = Mock()
    caso_uso = ObtenerCambiosUsuario(repo)

    with pytest.raises(ErrorDominio) as exc:
        caso_uso.ejecutar(usuario_id="u1", desde_cursor=-1)

    assert exc.value.campo == "desde"
    repo.consultar_cambios_usuario.assert_not_called()


def test_obtener_cambios_exitoso():
    repo = Mock()
    ev = EventoCambioUsuario(
        id="ev-1",
        posicion=1,
        usuario_id="u1",
        grupo_id="g1",
        nombre_grupo="Viaje",
        gasto_id="gasto-1",
        concepto="Hotel",
        fecha=datetime.now(timezone.utc),
        moneda="USD",
        monto_parte=D("50.00"),
        incluye_redondeo=False,
    )
    repo.consultar_cambios_usuario.return_value = ([ev], 1, False)

    caso_uso = ObtenerCambiosUsuario(repo)
    res = caso_uso.ejecutar(usuario_id="u1", desde_cursor=0, limite=100)

    repo.consultar_cambios_usuario.assert_called_once_with(
        usuario_id="u1",
        desde_cursor=0,
        limite=100,
    )
    assert res.nuevo_cursor == 1
    assert res.quedan_mas is False
    assert len(res.eventos) == 1
    assert res.eventos[0].id == "ev-1"
