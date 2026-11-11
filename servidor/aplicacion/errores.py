class ErrorAplicacion(Exception):

    def __init__(
            self,
            campo: str,
            mensaje: str
    ):
        self.campo = campo
        self.mensaje = mensaje
        super().__init__(mensaje)



class UsuarioYaExistente(ErrorAplicacion):

    def __init__(self, campo: str):
        mensajes = {
            "nombre_usuario": "El nombre de usuario ya está en uso.",
            "correo": "El correo ya está en uso."
        }

        super().__init__(campo, mensajes[campo])



class ContrasenaInvalida(ErrorAplicacion):

    def __init__(self):
        super().__init__(
            "contrasena",
            "La contraseña debe tener al menos 10 caracteres."
        )