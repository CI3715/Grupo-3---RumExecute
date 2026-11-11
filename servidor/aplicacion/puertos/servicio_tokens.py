from typing import Protocol
from datetime import datetime
from dataclasses import dataclass



@dataclass(frozen=True)
class TOkenSesion:
    valor: str
    fecha_expiracion: datetime



class ServicioTokens(Protocol):
    """
    Puerto para generar y verificar tokens de sesión.
    """

    def generar_token(
        self,
        usuario_id: str,
        duracion_minutos: int
    ) -> TokenSesion:
        """
        Genera un token firmado para un usuario.

        Args:
            usuario_id: Identificador del usuario.
            duracion_minutos: Vigencia del token.
        
        Returns: TokenSesion con el token y su vencimiento.
        """
        ...
    
    def verificar_token(
        self,
        token: str
    ) -> str:
        """
        Verifica la firma y vigencia del token

        Returns:
            Identificador del usuario autenticado.
        
        Raises:
            TokenInvalido:
                Si el token es inválido o está vencido.
        """
        ...