from dataclasses import dataclass
from dominio.errores import ErrorDominio, ErrorPermiso
from decimal import Decimal


@dataclass(frozen=True)
class GrupoCompartido:
    nombre_grupo: str
    moneda: str
    miembros: tuple[str, ...]
    administrador: str
    descripción: str = ""
    actividad: str = ""

    def __post_init__ (self):
        object.__setattr__(self, "nombre_grupo", self.nombre_grupo.strip())
        if not self.nombre_grupo.strip():
            raise ErrorDominio ("nombre_grupo", "El grupo debe poseer un nombre válido.")
        if len(self.moneda) != 3 or not self.moneda.isalpha() or not self.moneda.isupper() or not self.moneda.isascii():
            raise ErrorDominio ("moneda", "El codigo ISO de la moneda debe tener 3 letras en mayúsculas.")
        if not self.miembros:
            raise ErrorDominio ("miembros", "Debe existir la lista de miembros.")
        if len(set(self.miembros)) != len(self.miembros):
            raise ErrorDominio("miembros", "Existen miembros duplicados.")
        if self.administrador not in self.miembros:
            raise ErrorDominio("administrador", "El administrador debe ser miembro del grupo.")

def es_miembro(grupo: GrupoCompartido, nombre_usuario: str) -> bool:
    return nombre_usuario in grupo.miembros

def validar_es_admin(grupo: GrupoCompartido, nombre_usuario: str) -> None:
    if nombre_usuario != grupo.administrador:
        raise ErrorPermiso("usuario", "Solo el administrador puede realizar esta acción")












    