class ErrorDominio(Exception):
    def __init__(self, campo: str, mensaje: str):
        self.campo = campo
        self.mensaje = mensaje
        super().__init__(mensaje)

class ErrorConflicto(ErrorDominio):
    """Estado incompatible con la operación (se traduce a 409)."""


class ErrorPermiso(ErrorDominio):
    """El usuario no tiene permiso para la acción (se traduce a 403)."""