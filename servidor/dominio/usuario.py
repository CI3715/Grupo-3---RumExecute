import re
from dataclasses import dataclass
from datetime import datetime, timezone

from dominio.errores import ErrorDominio
# Caractéres requeridos para el nombre de usuario
PATRON_NOMBRE_USUARIO = re.compile(r"[a-z0-9._]{3,30}")
# Verificación simple para el correo (algo@algo.algo)
PATRON_CORREO = re.compile(r"[^@\s]+@[^@\s]+\.[^@\s]+")

@dataclass(frozen=True)
class Usuario:
    """
    Se utiliza dataclass con el propósito de hacer 
    las instancias inmutables
    """
    id: str
    nombre_usuario: str
    nombre_visible: str
    correo: str
    fecha_alta: datetime

    def __post_init__(self):
        # Primero se normaliza el correo
        object.__setattr__(self, "correo", self.correo.strip().lower())
        # Ahora, normalizamos los espacios al final y comienzo del nombre
        object.__setattr__(self, "nombre_visible", self.nombre_visible.strip())

        if self.fecha_alta.tzinfo is None:
            raise ErrorDominio("fecha_alta", "La fecha de alta debe incluir zona horaria (UTC).")
                # Normalizar a UTC-0
        if self.fecha_alta.tzinfo != timezone.utc:
            object.__setattr__(self, "fecha_alta", self.fecha_alta.astimezone(timezone.utc))
        
        if not PATRON_NOMBRE_USUARIO.fullmatch(self.nombre_usuario):
            raise ErrorDominio(
                "nombre_usuario",
                "El nombre de usuario debe tener de 3 a 30 caracteres: "
                "solo minúsculas, dígitos, punto y guion bajo.",
            )

        if not self.nombre_visible.strip():
            raise ErrorDominio(
                "nombre_visible",
                "El nombre visible es obligatorio.",
            )
        if not PATRON_CORREO.fullmatch(self.correo):
            raise ErrorDominio(
                "correo",
                "El correo no tiene un formato válido",
            )


