# ADR-005: Manejo de Tasas de Cambio, Tasa Inversa y Detección de Tasa Faltante

## Estado
Aceptado

## Fecha
2026-10-30

## Contexto
La Entrega 4 (*The Exchange Flow*) incorpora múltiples monedas, el registro manual de tasas de cambio históricas y la pantalla de Patrimonio consolidado en una Moneda Base elegida por el usuario (RF-CAM-01, RF-CAM-02, RF-CAM-03, RF-CAM-04, RF-CAM-06 y RF-INF-01).

Consolidar un patrimonio en varias monedas obliga a convertir montos, y toda conversión depende de un dato financiero (la tasa) que el sistema no puede conocer por sí solo: el usuario la registra a mano, con su fecha de vigencia y su fuente. El riesgo principal es que el sistema, para "completar" un total, invente o asuma una tasa y muestre una cifra falsa con apariencia de exacta.

Este ADR extiende el modelo de eventos inmutables del ADR-003 y la arquitectura hexagonal del ADR-002, y se apoya en las invariantes INV-01, INV-02, INV-03 e INV-13.

## Decisiones Adoptadas

### Tasa inversa calculada como 1 / valor
* **Decisión:** Si existe una tasa registrada A → B, la tasa B → A se calcula como `1 / valor` en tiempo de consulta. El usuario no necesita registrar ambos sentidos.
* **Razón de Elección:**
  * **Una sola fuente de verdad:** guardar A → B y B → A permitiría que ambas se contradigan.
  * **Menos trabajo para el usuario:** cambiar la moneda base a una moneda que solo aparece como destino de una tasa no obliga a registrar nada adicional.
* **Alternativa descartada:** persistir siempre las dos direcciones. Duplica datos y exige mantenerlos sincronizados.

### Prohibición de conversión triangular
* **Decisión:** Si existen A → B y B → C, el sistema **no** infiere A → C. Toda conversión requiere una tasa directa o inversa explícita entre las dos monedas.
* **Razón de Elección:**
  * Una tasa cruzada calculada es un dato financiero que el usuario nunca registró ni validó; las tasas reales de mercado entre A y C pueden diferir de la compuesta.
* **Alternativa descartada:** triangular a través de una moneda puente (por ejemplo USD). Produce totales plausibles pero no verificables.

### Selección de la tasa vigente por fecha
* **Decisión:** Para una fecha de consulta `T`, la tasa vigente de un par es la registrada con `fecha_vigencia <= T` más reciente. Si dos tasas comparten fecha de vigencia, gana la registrada más recientemente.
* **Razón de Elección:** permite consultar el patrimonio a fechas pasadas con la tasa que regía entonces, y que una tasa nueva con fecha posterior no altere consultas anteriores.
* **Preferencia por tasa directa:** el sistema busca primero una tasa vigente A → B; solo si no existe ninguna recurre a la inversa de B → A. No se compara la fecha entre ambas direcciones: una tasa directa vigente siempre prevalece, aunque la inversa sea más reciente.
* **Validaciones de registro:** monedas origen y destino distintas, valor estrictamente mayor que 0 con hasta 8 decimales, fecha de vigencia no futura y fuente no vacía. Estas reglas viven en el dominio, no en la interfaz.

### Detección estricta de tasa faltante (RF-CAM-04)
* **Decisión:** Si alguna cuenta con saldo no nulo está en una moneda sin tasa vigente (directa ni inversa) hacia la moneda base, la consulta devuelve las cuentas y los subtotales en su moneda original, pero `total_consolidado = None` y un objeto `tasa_faltante` con el par exacto y la fecha requerida.
* **Razón de Elección:**
  * **Honestidad de los datos:** un total incompleto presentado como total es peor que ningún total.
  * **Accionable:** el mensaje indica qué tasa registrar y a qué fecha, y la interfaz puede ofrecer registrarla de inmediato.
* **Alternativas descartadas:** asumir 1:1, omitir la cuenta del total o usar la última tasa conocida aunque sea posterior a la fecha consultada.

### Eliminación de tasas mediante el evento `TasaEliminada`
* **Decisión:** Corregir una tasa errónea se hace emitiendo un evento inmutable `TasaEliminada`, nunca con un `DELETE` físico. Al reconstruir el historial por replay, una tasa eliminada deja de ser candidata a tasa vigente.
* **Razón de Elección:** coherente con INV-13 y el ADR-003; la tabla `eventos_libro` mantiene triggers `BEFORE UPDATE` y `BEFORE DELETE` que rechazan cualquier mutación, de modo que el historial de qué se registró y qué se corrigió permanece auditable.

### Aritmética decimal exacta y redondeo al final
* **Decisión:** Toda tasa y conversión usa `rust_decimal::Decimal` (INV-03); está prohibido `f32`, `f64` y `Number` de JavaScript. Los montos viajan como pares `{cantidad, moneda}` (INV-01). Al consolidar, se convierte cada saldo, se suma y el redondeo a los decimales de la moneda base se aplica una sola vez sobre el resultado final.
* **Razón de Elección:** redondear antes de sumar acumula errores de centavo.

### Subtotales separados y moneda base sin efecto sobre los datos
* **Decisión:** Los subtotales se muestran siempre por moneda original (INV-02); nunca se suman cantidades de monedas distintas sin tasa. Cambiar la moneda base emite `MonedaBaseCambiada` y solo afecta la presentación de la consolidación: no modifica ni recalcula movimientos ni saldos originales.

### Catálogo de monedas extensible con base de fábrica protegida
* **Decisión:** VES, USD, EUR y USDT son inmutables y no pueden modificarse ni eliminarse. El usuario puede crear monedas propias (`MonedaCreada`) con código `^[A-Z]{3,5}$`, nombre de 1 a 50 caracteres, símbolo de 1 a 5 y decimales entre 0 y 8.

## Consecuencias

### Positivas
* Ningún total consolidado se basa en datos que el usuario no haya registrado: el sistema falla de forma explícita en lugar de aproximar.
* Consultas históricas reproducibles: el mismo patrimonio a la misma fecha da siempre el mismo resultado, incluso tras registrar tasas posteriores.
* La lógica de conversión vive en dominio y aplicación, y se prueba con el repositorio en memoria y también sobre SQLite real, incluida la persistencia tras cerrar y reabrir la base.
* Los eventos nuevos (`MonedaCreada`, `MonedaBaseCambiada`, `TasaRegistrada`, `TasaEliminada`) heredan la inmutabilidad de la tabla `eventos_libro` sin cambios de esquema.

### Negativas
* El usuario debe registrar manualmente cada par que necesite; con muchas monedas, el trabajo crece y la ausencia de triangulación puede resultar incómoda.
* La tasa inversa `1 / valor` puede tener más decimales de los que admite la moneda destino; el redondeo solo se aplica al resultado final y su precisión depende de la representación de `Decimal`.
* Recalcular el patrimonio proyecta todos los eventos en cada consulta, con el mismo costo de crecimiento del log descrito en el ADR-003.
* La alerta informa un único par faltante por consulta (el primero encontrado); si faltan varios, se descubren de uno en uno.
* Por la preferencia directa sobre inversa, una tasa directa antigua puede prevalecer sobre una inversa más reciente; el usuario debe eliminar la antigua si quiere que aplique la nueva.