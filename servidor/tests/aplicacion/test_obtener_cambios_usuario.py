from unittest.mock import Mock
from datetime import datetime, timezone
from decimal import Decimal as D
import pytest

from aplicacion.obtener_cambios_usuario import ObtenerCambiosUsuario
from dominio.errores import ErrorDominio
from dominio.evento_cambio import EventoCambioUsuario


def test_cursor_negativo_lanza_error_dominio():
    repo = Mock()
    caso_uso = ObtenerCambiosUsuario(repo)

    with pytest.raises(ErrorDominio) as exc:
        caso_uso.ejecutar(usuario_id="u1", desde_cursor=-1)

    assert exc.value.campo == "desde"
    repo.consultar_cambios_usuario.assert_not_called()


def test_obtener_cambios_exitoso():
    repo = Mock()
    evento = EventoCambioUsuario(
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
    repo.consultar_cambios_usuario.return_value = ([evento], 1, False)

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
