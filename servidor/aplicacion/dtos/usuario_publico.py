from dataclasses import dataclass


@dataclass(frozen=True)
class UsuarioPublico:
    nombre_usuario: str
    nombre_visible: str