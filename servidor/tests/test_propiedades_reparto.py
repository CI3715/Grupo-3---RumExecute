from datetime import datetime, timezone
from decimal import Decimal as D

from hypothesis import given, strategies as st

from dominio.balance import Gasto, calcular_balances
from dominio.division import dividir_partes_iguales, dividir_por_montos, dividir_por_porcentajes

AHORA = datetime.now(timezone.utc)
totales = st.decimals(min_value="0.01", max_value="1000000000", places=2)


def nombres(n):
    return [f"u{i:02d}" for i in range(n)]


@given(total=totales, n=st.integers(1, 25))
def test_partes_iguales_suman_el_total(total, n):
    cuotas = dividir_partes_iguales(total, nombres(n))
    assert sum((c.monto for c in cuotas), D("0")) == total
    assert all(c.monto == c.monto.quantize(D("0.01")) for c in cuotas)


@given(total=totales, n=st.integers(1, 25))
def test_el_residuo_va_a_los_primeros_en_orden_alfabetico(total, n):
    cuotas = dividir_partes_iguales(total, list(reversed(nombres(n))))
    con_residuo = [c.lleva_residuo for c in cuotas]
    # los que llevan residuo forman un prefijo del orden alfabético
    assert con_residuo == sorted(con_residuo, reverse=True)


@st.composite
def reparto_en_basis_points(draw):
    n = draw(st.integers(1, 12))
    cortes = sorted(draw(st.lists(st.integers(0, 10000), min_size=n - 1, max_size=n - 1)))
    puntos = [b - a for a, b in zip([0] + cortes, cortes + [10000])]
    return {nombre: D(p) / D(100) for nombre, p in zip(nombres(n), puntos)}


@given(total=totales, pct=reparto_en_basis_points())
def test_porcentajes_suman_el_total(total, pct):
    cuotas = dividir_por_porcentajes(total, pct)
    assert sum((c.monto for c in cuotas), D("0")) == total


@given(total=totales, n=st.integers(1, 12))
def test_montos_fijos_suman_el_total_sin_residuo(total, n):
    base = (total / n).quantize(D("0.01"), rounding="ROUND_DOWN")
    montos = {nombre: base for nombre in nombres(n)}
    montos[nombres(n)[0]] += total - base * n
    cuotas = dividir_por_montos(total, montos)
    assert sum((c.monto for c in cuotas), D("0")) == total
    assert not any(c.lleva_residuo for c in cuotas)


@given(
    gastos=st.lists(
        st.tuples(totales, st.integers(0, 4), st.sets(st.integers(0, 4), min_size=1)),
        max_size=15,
    )
)
def test_los_balances_siempre_suman_cero(gastos):
    miembros = tuple(nombres(5))
    lista = []
    for total, pagador, participantes in gastos:
        cuotas = tuple(dividir_partes_iguales(total, [miembros[i] for i in participantes]))
        lista.append(Gasto("Gasto", miembros[pagador], total, cuotas, AHORA))
    balances = calcular_balances(miembros, lista)
    assert sum(balances.values(), D("0")) == D("0")