# ADR-007: Autenticación, hashing y sesiones

- **Estado:** Aceptado
- **Contexto:** Entrega 5
- **Fecha:** 2026-11-13

## Contexto

Las funciones sociales requieren identificar al usuario que realiza una petición, sin convertir el inicio de sesión en requisito para las funciones personales locales. Las contraseñas no deben almacenarse ni transmitirse de vuelta como texto plano, y las credenciales de sesión del cliente no deben persistirse en almacenamiento accesible a la interfaz web.

## Decisión

### Separar identidad de credenciales

El dominio representa el perfil en `Usuario` y mantiene las credenciales en una entidad `Credenciales` asociada mediante `usuario_id`. Las consultas de perfil y los DTO públicos no incluyen el hash; el adaptador de persistencia recupera las credenciales por separado para el inicio de sesión. Esto reduce la exposición accidental del hash y evita trasladarlo en operaciones que solo necesitan la identidad.

El `usuario_id` es la clave estable para consultas y comparaciones. No se usa el nombre de usuario como identidad porque el dominio no lo declara inmutable.

### Hash de contraseñas

El registro valida los datos y genera un hash Argon2id mediante `argon2-cffi`. Se configura con una sal aleatoria de 16 bytes, `time_cost=3`, `memory_cost=65536` KiB, `parallelism=4` y `hash_len=32`; la sal y los parámetros quedan codificados junto al hash. Los casos de uso dependen de puertos para hashing y credenciales, por lo que la política de autenticación no acopla el dominio al framework. La contraseña plana solo se usa para la verificación y no se conserva.

### Token de sesión

El inicio de sesión emite JWT firmado con HS256, con una vigencia de 60 minutos. El payload incluye el identificador de usuario (`sub`), emisión (`iat`) y expiración (`exp`). Los tiempos se calculan en UTC; la expiración devuelta por la API se serializa como ISO-8601 terminado en `Z`. La clave de firma se configura mediante `JWT_SECRET`.

Los endpoints protegidos exigen `Authorization: Bearer <token>`. El middleware valida firma y expiración, y rechaza credenciales ausentes, inválidas o vencidas con 401. El inicio de sesión devuelve el mismo mensaje para correo o contraseña incorrectos, evitando distinguir cuál dato falló. Colisiones de usuario/correo se informan como 409; validaciones de contraseña, como 400. Las respuestas de usuario nunca incluyen contraseña ni hash.

En el cliente Tauri, el adaptador de infraestructura usa `keyring` para guardar, recuperar y eliminar el token mediante el almacén de credenciales del sistema operativo. Las llamadas HTTP usan `reqwest`. Registrar o iniciar sesión sigue siendo opcional para el uso personal sin conexión.

## Alternativas consideradas

1. **Contraseña en texto plano o hash rápido:** se descartan por el riesgo de exposición y la falta de resistencia adecuada a ataques de diccionario.
2. **Incluir el hash como atributo del perfil de usuario:** se descarta para que las consultas ordinarias y DTO públicos no transporten información de autenticación.
3. **Usar el nombre de usuario como identificador:** se descarta porque puede cambiar; las relaciones y búsquedas usan el ID estable.
4. **Sesión sin expiración:** se descarta; las credenciales temporales limitan la ventana de uso de un token filtrado.
5. **Guardar el token en `localStorage`:** se descarta en favor del almacén de credenciales del sistema operativo.
6. **Devolver errores distintos para correo y contraseña:** se descarta para evitar filtración de existencia de cuentas durante el login.

## Consecuencias y límites

- El backend debe recibir `JWT_SECRET` por configuración y mantenerlo fuera del repositorio.
- El valor alternativo de secreto definido en el código sirve solo para desarrollo local; no es aceptable en un despliegue.
- JWT firmado no cifra el payload; no se deben incluir contraseñas ni datos sensibles en sus claims.
- La API debe desplegarse detrás de HTTPS/TLS para proteger credenciales y tokens en tránsito.
- Un token expirado requiere iniciar sesión de nuevo; el almacenamiento seguro local no extiende su vigencia en el servidor.
