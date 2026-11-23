# Documento técnico explicativo — Entrega 3

**Proyecto:** Cuentas Claras · **Equipo:** Grupo 3 — RumExecute
**Asignatura:** CI-3715 Ingeniería de Software · Universidad Simón Bolívar

## 1. Propósito y alcance

La Entrega 3 implementa el flujo contable de movimientos y correcciones inmutables en el libro personal (**The Ledger Flow**):

1. Registro de movimientos financieros de tipo **Ingreso** y **Gasto** asociados a una cuenta personal, con fecha, categoría (de fábrica o personalizada) y descripción.
2. Consulta del detalle de una cuenta con su historial cronológico y el desglose de la ecuación contable: `Saldo Inicial + Ingresos - Gastos ± Ajustes = Saldo Actual`.
3. Corrección no destructiva de movimientos mediante eventos compensatorios de tipo **Ajuste** (`INV-04`, `INV-05`, `INV-13`), prohibiendo editar o eliminar movimientos históricos tanto en la interfaz como en la base de datos SQLite.

## 2. Requerimientos e invariantes cubiertos

| Código | Descripción y cumplimiento en el sistema |
|---|---|
| `RF-LIB-04` / `RF-LIB-05` | Registro de gastos e ingresos con validación de monto estrictamente positivo (`> 0`), categoría del tipo correspondiente y rechazo de fechas futuras. |
| `RF-LIB-06` | Consulta del detalle de cuenta con saldo inicial, total de ingresos, total de gastos, neto de ajustes, saldo actual e historial completo. |
| `RF-LIB-07` | Catálogo oficial de categorías de gasto e ingreso y creación de categorías personalizadas (`crear_categoria`, `listar_categorias`) sin duplicados por tipo. |
| `INV-04` | Inmutabilidad contable y corrección por ajuste: ningún movimiento se borra ni se edita. Al corregir un movimiento, el original pasa a mostrarse como `"Corregido"` y se emite un nuevo movimiento de tipo `"Ajuste"` que referencia al original (`id_movimiento_referenciado`) por la diferencia exacta. |
| `INV-05` | Proyección exacta de saldos: el saldo actual de toda cuenta es siempre igual a la suma algebraica de su saldo inicial, sus ingresos, sus gastos y sus ajustes compensatorios. |
| `INV-13` | Persistencia inmutable en SQLite: los triggers `eventos_libro_no_update` y `eventos_libro_no_delete` impiden cualquier modificación o eliminación de filas en `eventos_libro`. |

## 3. Cálculo matemático del ajuste compensatorio (`INV-04`)

Cuando el usuario corrige un movimiento original con monto $M_{\text{orig}}$ indicando el nuevo monto correcto $M_{\text{nuevo}}$ (donde $M_{\text{nuevo}} \ge 0$ y $M_{\text{nuevo}} \ne M_{\text{orig}}$):

- **Si el movimiento original fue un Gasto:**
  $$\Delta_{\text{ajuste}} = M_{\text{orig}} - M_{\text{nuevo}}$$
  Por ejemplo, si se registró por error un gasto de $500{,}00$ y el monto real era $50{,}00$, se genera un ajuste de $+450{,}00$ que restituye la diferencia al saldo de la cuenta.
- **Si el movimiento original fue un Ingreso:**
  $$\Delta_{\text{ajuste}} = M_{\text{nuevo}} - M_{\text{orig}}$$
  Por ejemplo, si se registró un ingreso de $800{,}00$ y el monto real era $850{,}00$, se genera un ajuste de $+50{,}00$.

Un movimiento de tipo `Ajuste` no puede ser objeto de una nueva corrección directa; las correcciones siempre se aplican sobre movimientos de ingreso o gasto vigentes.

## 4. Comandos IPC y pruebas de verificación

- **Comandos Tauri (`front_end/src-tauri/src/comandos.rs`):** `registrar_movimiento`, `corregir_movimiento`, `obtener_detalle_cuenta`, `crear_categoria` y `listar_categorias`.
- **Pruebas automatizadas (`cargo test`):**
  - Verificación del escenario de la Sección 2.2 (cuenta con saldo inicial `1.000,00 USD`, ingreso de `800,00`, gasto erróneo de `500,00` corregido a `50,00` mediante ajuste de `+450,00`, resultando en un saldo final exacto de `1.750,00 USD`).
  - Pruebas de contrato e inmutabilidad en `sqlite_repo.rs` que comprueban que `UPDATE` y `DELETE` sobre `eventos_libro` son rechazados por el motor SQLite.

## 5. Decisiones de diseño

- [ADR-003 — Almacenamiento local SQLite y Event Sourcing](adr/ADR-003-almacenamiento-local-sqlite-y-event-sourcing.md).
- [ADR-004 — Inmutabilidad contable y corrección mediante ajustes compensatorios](adr/ADR-004-inmutabilidad-y-correccion-por-ajustes.md).
