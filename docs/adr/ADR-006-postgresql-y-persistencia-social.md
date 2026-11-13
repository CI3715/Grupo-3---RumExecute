# ADR-006: PostgreSQL y persistencia de datos sociales

- **Estado:** Aceptado
- **Contexto:** Entregas 5 y 6
- **Fecha:** 2026-11-13

## Contexto

El servidor coordina identidades, amistades, grupos y gastos compartidos. Necesita transacciones e integridad referencial para operaciones concurrentes, pero no debe almacenar los datos financieros personales que pertenecen al cliente local (INV-16 / RES-02). Las pruebas de integración deben ejecutarse sobre el mismo motor relacional que usa el servidor.

Los registros sociales requieren identificadores estables para búsquedas y relaciones entre entidades. La fecha y hora se normaliza a UTC para que el almacenamiento y la comparación no dependan de la zona horaria del equipo; la interfaz puede convertirla a la zona local al mostrarla.

## Decisión

Se adopta PostgreSQL 16 como persistencia del servidor, ejecutado localmente mediante Docker Compose y con volumen nombrado `postgres_data`. El puerto del host es 5433 para evitar colisiones con instalaciones locales que ya usan el puerto predeterminado 5432; el servicio se publica solo en `127.0.0.1`. SQLAlchemy configura `pool_pre_ping=True` para detectar conexiones interrumpidas antes de utilizarlas.

Los identificadores son UUID v4 generados en la aplicación antes de persistir y representados como `String(36)` tanto en claves primarias como foráneas. El dominio conserva así la responsabilidad de generar la identidad, mientras que la representación textual evita incompatibilidades al comparar parámetros UUID con columnas `VARCHAR` y se mantiene uniforme entre dominio, persistencia y DTO.

Los adaptadores se separan por responsabilidad:

- `RepositorioUsuariosPostgres` implementa los puertos de usuarios, credenciales y persistencia del registro.
- `RepositorioAmistadesPostgres` implementa la persistencia de amistades.
- `RepositorioGruposPostgres` implementa grupos, miembros e invitaciones.
- `RepositorioGastosPostgres` implementa gastos y cuotas.

La inyección de dependencias de FastAPI construye los casos de uso con estos adaptadores. La base del servidor solo contiene usuarios, amistades, grupos, miembros, gastos compartidos y cuotas: no existen allí cuentas, presupuestos ni movimientos privados.

Las marcas temporales se guardan con zona horaria y se producen en UTC. Así, las operaciones y consultas son comparables entre clientes ubicados en zonas distintas; la conversión para presentación corresponde al cliente.

La suite de integración utiliza PostgreSQL real para cubrir restricciones, tipos decimales y comportamiento del motor. El fixture de API limpia las tablas sociales con `TRUNCATE ... CASCADE` antes y después de cada prueba; por ello debe apuntar exclusivamente a una base descartable.

## Alternativas consideradas

1. **SQLite en memoria para pruebas de API:** se descarta porque difiere de PostgreSQL en tipos, restricciones, concurrencia y manejo de fechas.
2. **Tipo UUID nativo de PostgreSQL en las columnas:** se descarta en favor de `String(36)`, pues mezclar columnas de texto con parámetros UUID provoca errores de comparación/casting en SQLAlchemy/PostgreSQL y rompe la representación uniforme entre dominio, persistencia y DTO.
3. **Guardar los datos personales en el servidor:** se descarta por el principio de privacidad y el invariante INV-16.
4. **Guardar fechas sin zona horaria o en la hora local del servidor:** se descarta porque hace ambiguas las comparaciones y conversiones cuando clientes y servidor operan en zonas distintas.

## Consecuencias

- La ejecución del backend y de las pruebas de integración requiere PostgreSQL disponible.
- Cada prueba que use el fixture de API elimina datos de las tablas compartidas; una base separada para pruebas es obligatoria.
- El volumen Docker hace persistentes los datos locales al reiniciar contenedores.
- El puerto 5433 evita una colisión habitual en desarrollo local, y el enlace a `127.0.0.1` evita publicar PostgreSQL en interfaces externas.
- La URL de conexión se configura con `DATABASE_URL`; en Compose, el adaptador puede construirla con parámetros separados para evitar problemas con caracteres reservados en la contraseña.
- Las fechas se persisten en UTC; la hora local es únicamente una decisión de presentación del cliente.
- No se mantiene un saldo mutable: los gastos y cuotas persistidos son la fuente de verdad y los balances se derivan al consultar.
