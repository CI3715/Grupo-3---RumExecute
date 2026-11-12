"""
Módulo de Infraestructura: Seguridad y Hashing de Contraseñas (RNF-05).
Utiliza Argon2id con sal única generada automáticamente y derivación lenta.
"""
from argon2 import PasswordHasher
from argon2.exceptions import VerifyMismatchError, VerificationError, InvalidHashError

# Instancia configurada de Argon2id (los parámetros por defecto de argon2-cffi
# cumplen con las recomendaciones de OWASP para Argon2id con sal criptográfica única)
_hasher = PasswordHasher(
    time_cost=3,        # Iteraciones
    memory_cost=65536,  # 64 MB de memoria
    parallelism=4,      # Hilos en paralelo
    hash_len=32,        # Longitud del hash resultante
    salt_len=16         # Sal criptográfica única de 16 bytes
)


def hashear_password(password_plano: str) -> str:
    """
    Genera un hash seguro Argon2id a partir de una contraseña en texto plano.
    
    Invariante RNF-05:
    - Se genera una sal única aleatoria por cada ejecución.
    - El resultado es una cadena codificada que contiene parámetros, sal y hash.
    - Nunca almacena ni expone la contraseña en texto plano.
    """
    if not isinstance(password_plano, str) or not password_plano:
        raise ValueError("La contraseña a hashear debe ser una cadena no vacía.")
    
    return _hasher.hash(password_plano)


def verificar_password(password_plano: str, password_hash: str) -> bool:
    """
    Verifica si una contraseña en texto plano coincide con el hash Argon2id almacenado.
    
    Retorna True si coincide, False en caso contrario.
    Maneja internamente errores de verificación sin revelar detalles.
    """
    if not password_plano or not password_hash:
        return False

    try:
        return _hasher.verify(password_hash, password_plano)
    except (VerifyMismatchError, VerificationError, InvalidHashError):
        return False


def necesita_rehasheo(password_hash: str) -> bool:
    """
    Comprueba si el hash almacenado requiere actualizarse a parámetros más fuertes.
    """
    try:
        return _hasher.check_needs_rehash(password_hash)
    except Exception:
        return False