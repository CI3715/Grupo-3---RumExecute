from dataclasses import dataclass

from dominio.errores import ErrorDominio


@dataclass(frozen=True)
class Credenciales:
    usuario_id: str
    hash_contrasena: str

    def __post_init__(self):
        if not self.usuario_id:
            raise ErrorDominio(
                "usuario_id",
                "El identificador del usuario es obligatorio."
            )

        if not self.hash_contrasena:
            raise ErrorDominio(
                "hash_contrasena",
                "El hash de la contraseña es obligatorio."
            )