# ADR-004: Inmutabilidad contable y corrección mediante ajustes compensatorios

- **Estado:** Aceptado
- **Contexto:** Entrega 3 (The Ledger Flow)
- **Fecha:** 2026-10-22

## Contexto

En un libro contable financiero, permitir la edición destructiva (`UPDATE`) o la eliminación (`DELETE`) de movimientos registrados destruye la trazabilidad de auditoría y puede producir inconsistencias en la evolución histórica del saldo y del patrimonio. Sin embargo, los usuarios cometen errores de digitación al anotar gastos o ingresos y necesitan un mecanismo claro para corregirlos (`INV-04`, `INV-05`).

## Decisión

1. **Prohibición de edición y borrado de movimientos:**
   - Los eventos `MovimientoRegistrado` almacenados en `eventos_libro` nunca se modifican ni se eliminan. Los triggers de base de datos `eventos_libro_no_update` y `eventos_libro_no_delete` rechazan cualquier sentencia `UPDATE` o `DELETE` a nivel de SQLite.
   - La interfaz de usuario no ofrece botones de eliminar ni de sobreescribir un movimiento existente; únicamente ofrece la acción **"Corregir"** sobre movimientos de gasto o ingreso vigentes.

2. **Corrección mediante evento compensatorio (`MovimientoCorregido` / `Ajuste`):**
   - Al corregir un movimiento original con monto $M_{\text{orig}}$ a un nuevo monto válido $M_{\text{nuevo}}$, el dominio calcula la diferencia algebraica compensatoria ($\Delta = M_{\text{orig}} - M_{\text{nuevo}}$ para gastos; $\Delta = M_{\text{nuevo}} - M_{\text{orig}}$ para ingresos) y persiste un nuevo evento inmutable `EventoLibro::MovimientoCorregido`.
   - En la proyección de la cuenta (`obtener_detalle_cuenta`), el movimiento original permanece visible en el historial marcado con el estado `"corregido"`, mientras que el nuevo movimiento aparece como tipo `"ajuste"` con referencia explícita al identificador del movimiento original (`id_movimiento_referenciado`).

3. **Proyección determinista del saldo (`INV-05`):**
   - El saldo actual de la cuenta se deriva siempre de la ecuación:
     $$\text{Saldo Actual} = \text{Saldo Inicial} + \sum \text{Ingresos} - \sum \text{Gastos} + \sum \text{Ajustes}$$

## Alternativas consideradas

1. **Modificar el monto directamente con `UPDATE` en SQLite:** se descarta porque infringe el patrón de Event Sourcing (`RNF-12`) y el invariante `INV-04`, perdiendo el registro del valor original y el motivo de la corrección.
2. **Eliminar el movimiento erróneo y crear uno nuevo:** se descarta porque altera la numeración histórica y no deja constancia auditable de la rectificación.

## Consecuencias

- El historial de cada cuenta conserva una pista de auditoría completa e inalterable.
- El cálculo de saldos y patrimonio en cualquier fecha histórica es completamente reproducible a partir de la secuencia de eventos del libro local.
