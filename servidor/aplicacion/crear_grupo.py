from uuid import uuid4

from aplicacion.dtos.grupo_creado import GrupoCreado
from aplicacion.errores import UsuarioNoEncontrado
from aplicacion.puertos.repositorio_grupos import RepositorioGrupos
from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from dominio.grupo import GrupoCompartido


class CrearGrupo:

    def __init__(
        self,
        repositorio_usuarios: RepositorioUsuarios,
        repositorio_grupos: RepositorioGrupos,
    ):
        self.repositorio_usuarios = repositorio_usuarios
        self.repositorio_grupos = repositorio_grupos

    def ejecutar(
        self,
        usuario_autenticado_id: str,
        nombre_grupo: str,
        moneda: str,
        descripcion: str = "",
    ) -> GrupoCreado:

        usuario = self.repositorio_usuarios.obtener_usuario_por_id(
            usuario_autenticado_id
        )

        if usuario is None:
            raise UsuarioNoEncontrado()

        grupo = GrupoCompartido(
            id=str(uuid4()),
            nombre_grupo=nombre_grupo,
            moneda=moneda,
            id_miembros=(usuario.id,),
            id_administrador=usuario.id,
            descripcion=descripcion,
        )

        self.repositorio_grupos.guardar_grupo(grupo)

        return GrupoCreado(
            id=grupo.id,
            nombre_grupo=grupo.nombre_grupo,
            moneda=grupo.moneda,
            administrador_id=grupo.id_administrador,
            descripcion=grupo.descripcion,
        )
