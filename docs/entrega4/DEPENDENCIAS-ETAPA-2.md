# Etapa 2 — avance y dependencias de Etapa 3

## Implementado

- `src-tauri/src/aplicacion/dto.rs` define contratos para consulta de patrimonio, tasas, monedas e historial.
- Los casos de uso de aplicación registran/eliminan tasas, cambian la moneda base, crean/listan monedas y consultan el historial de tasas.
- `src-tauri/src/aplicacion/consultar_patrimonio.rs` reconstruye los saldos desde los eventos hasta la fecha solicitada, usa `Decimal`, agrupa subtotales sin mezclar monedas, elige la tasa directa más reciente (o la inversa si no hay directa), prohíbe tasas triangulares, redondea una sola vez al final y no devuelve consolidado si falta una tasa para un saldo distinto de cero.
- El historial expone las tasas eliminadas como tales. Se añadió el ID de la tasa al evento de eliminación para que se elimine exactamente la solicitada; el campo es opcional al deserializar para conservar compatibilidad con eventos previos.

## Pendiente de la Etapa 3

1. Exponer estos casos de uso como comandos Tauri usando el estado del repositorio SQLite.
2. Cambiar el comando de listado de monedas para que lea las monedas propias desde el libro, en vez de devolver solamente el catálogo de fábrica.
3. Conectar los DTOs de entrada/salida con los nombres y casing exactos de `invoke`.
4. Ejecutar `cargo check` y `cargo clippy` una vez disponible la herramienta Rust; la suite automatizada permanece centralizada en Etapa 5 según el flujo acordado.
