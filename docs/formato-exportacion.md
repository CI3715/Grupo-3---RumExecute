# Formato de exportacion del libro personal

La exportacion sin cifrar es texto UTF-8 con este encabezado de cuatro lineas,
seguido por un objeto JSON `EventoLibro` por linea:

```text
CUENTAS_CLARAS_LIBRO v1
FECHA_EXPORTACION: 2026-11-26T18:00:00Z
CANTIDAD_EVENTOS: 1
---
{"clase_evento":"CuentaCreada","id":"...","tipo":"cuenta_creada","momento_registro":"...","dispositivo_origen":"...","datos":{...}}
```

La cantidad del encabezado debe coincidir con los eventos serializados. Cada
evento conserva su identificador, tipo, instante, dispositivo de origen y datos
de dominio. El archivo completo se valida antes de fusionar eventos con el libro
local.

## Exportacion cifrada

El archivo cifrado es binario. Su encabezado tiene 68 bytes: el prefijo ASCII
`CC_CIFRADO_V1\n`, un byte de version, un byte de algoritmo, los parametros
Argon2id `m`, `t` y `p` como enteros little-endian de 32 bits, una sal aleatoria
de 16 bytes y un nonce XChaCha20 de 24 bytes. Le siguen el texto cifrado y la
etiqueta de autenticacion Poly1305.

Los parametros actuales de derivacion son 19 456 KiB de memoria, 2 iteraciones
y paralelismo 1; la clave derivada tiene 256 bits. El encabezado completo se
autentica como datos asociados. Una frase incorrecta o cualquier modificacion
del archivo impide descifrarlo. La frase de paso y las credenciales Git no se
incluyen en la exportacion.
