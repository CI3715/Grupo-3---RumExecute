# ADR-008: Reparto determinista de gastos y cálculo de balances

- **Estado:** Aceptado
- **Contexto:** Entrega 6
- **Fecha:** 2026-11-13

## Contexto

Los importes compartidos deben conservar exactamente el total, incluidos los centavos que no se distribuyen uniformemente. Las cuotas y los balances deben poder reproducirse a partir de los gastos registrados y mantenerse coherentes cuando varias personas operan sobre un grupo.

## Decisión

Los importes y porcentajes se procesan con `Decimal`, no con aritmética binaria de punto flotante. Cada gasto contiene el importe original, su pagador, regla de división y las cuotas resultantes. La persistencia usa columnas `NUMERIC` con dos decimales para montos y cuotas; en el contrato JSON del cliente, los importes se serializan como cadenas para no perder precisión al pasar por JavaScript.

Las búsquedas, comparaciones y relaciones de las personas usan `usuario_id`, ya que el nombre de usuario no se ha definido como inmutable. Para que el residuo sea comprensible y reproducible para los integrantes, cada cuota conserva tanto el ID como el nombre de usuario: el ID identifica al participante y el nombre determina el orden alfabético de asignación.

Se admiten tres modalidades:

1. **Partes iguales:** se calcula la cuota base al centavo y los centavos residuales se asignan en orden alfabético de nombre de usuario. Para 100,00 entre Ana, Beto y Carla: Ana recibe 33,34 y los otros dos 33,33.
2. **Porcentajes:** los porcentajes deben sumar exactamente 100,00 %. Los importes se calculan con precisión decimal y los residuos se asignan de forma reproducible para conservar el total.
3. **Montos fijos:** los importes indicados deben sumar exactamente el total del gasto; no se acepta una diferencia implícita.

Al consultar el detalle del grupo, se derivan los balances desde los gastos y cuotas. Para cada miembro, el pago realizado se compensa con su cuota; la suma algebraica del grupo debe ser exactamente 0,00 (INV-08). No se actualiza una columna de saldo independiente.

El acceso a consultas y registro de gastos se limita a miembros aceptados. La API responde 404 indistinguible para grupos inexistentes y grupos ajenos al usuario (INV-17 / ESC-08).

## Alternativas consideradas

1. **`float` para moneda:** se descarta porque su representación binaria puede introducir residuos y comparaciones inexactas.
2. **Redondeo independiente sin regla de residuo:** se descarta porque la suma de cuotas podría diferir del gasto.
3. **Saldo mutable por miembro:** se descarta porque actualizaciones concurrentes o fallidas pueden desincronizar el saldo con los gastos.
4. **Elegir al azar quién recibe el centavo restante:** se descarta porque dificultaría explicar y reproducir los resultados.
5. **Usar el ID como criterio para repartir el residuo:** se descarta porque los identificadores no expresan un orden entendible para los miembros y el requerimiento de reparto establece el orden alfabético de los nombres de usuario.

## Consecuencias

- El orden determinista debe emplearse de igual manera en los cálculos y pruebas.
- Las pruebas de propiedades verifican conservación del total, precisión de las cuotas y suma cero de balances para una variedad de entradas.
- Las cuotas almacenadas permiten explicar cómo se dividió cada gasto, mientras los balances se recalculan a partir de esos hechos.
- Las consultas de grupos deben verificar membresía antes de revelar detalle o aceptar escrituras.
