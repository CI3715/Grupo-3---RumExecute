"""
Adaptadores de infraestructura para los servicios de Hash y Tokens
que implementan los puertos definidos en la capa de aplicación.
"""
from datetime import datetime
from aplicacion.puertos.servicio_hash_contrasenas import ServicioHashContrasenas
from aplicacion.puertos.servicio_tokens import ServicioTokens, TokenSesion
from aplicacion.errores import TokenInvalido

from .seguridad import hashear_password, verificar_password
from .jwt_servicio import generar_token, decodificar_token


class AdaptadorServicioHash(ServicioHashContrasenas):
    def generar_hash(self, contrasena: str) -> str:
        return hashear_password(contrasena)

    def verificar(self, contrasena: str, hash_almacenado: str) -> bool:
        return verificar_password(contrasena, hash_almacenado)

    def verificar_contrasena(self, contrasena: str, hash_almacenado: str) -> bool:
        """Alias compatible con el caso de uso IniciarSesion."""
        return self.verificar(contrasena, hash_almacenado)


class AdaptadorServicioTokens(ServicioTokens):
    def generar_token(self, usuario_id: str, duracion_minutos: int) -> TokenSesion:
        datos = {"sub": usuario_id}
        res = generar_token(datos, vigencia_minutos=duracion_minutos)
        
        # Parsear fecha de expiración
        expiracion = datetime.fromisoformat(res["expires_at"].replace("Z", "+00:00"))
        
        return TokenSesion(
            valor=res["access_token"],
            fecha_expiracion=expiracion
        )

    def verificar_token(self, token: str) -> str:
        payload = decodificar_token(token)
        if not payload or "sub" not in payload:
            raise TokenInvalido()
        return str(payload["sub"])
