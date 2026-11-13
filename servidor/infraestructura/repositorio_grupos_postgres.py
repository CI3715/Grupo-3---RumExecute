"""
Implementación del Repositorio de Grupos e Invitaciones en PostgreSQL con SQLAlchemy.
Cumple con aplicacion.puertos.repositorio_grupos.RepositorioGrupos.
"""
from typing import Optional
from sqlalchemy.orm import Session  # type: ignore

from dominio.grupo import GrupoCompartido
from dominio.invitacion_grupo import InvitacionGrupo as InvitacionDominio, EstadoInvitacionGrupo
from dominio.errores import ErrorConflicto
from aplicacion.puertos.repositorio_grupos import RepositorioGrupos

from .modelos_bd import Grupo, MiembroGrupo


class RepositorioGruposPostgres(RepositorioGrupos):
    def __init__(self, db: Session):
        self.db = db

    def guardar_grupo(self, grupo: GrupoCompartido) -> None:
        nuevo_grupo = Grupo(
            id=grupo.id,
            nombre=grupo.nombre_grupo,
            descripcion=grupo.descripcion,
            moneda=grupo.moneda,
            actividad_fecha=grupo.actividad,
            creador_id=grupo.id_administrador,
        )
        self.db.add(nuevo_grupo)

        for miembro_id in grupo.id_miembros:
            rol = "ADMIN" if miembro_id == grupo.id_administrador else "MIEMBRO"
            miembro = MiembroGrupo(
                grupo_id=grupo.id,
                usuario_id=miembro_id,
                rol=rol,
                estado="ACEPTADO",
            )
            self.db.add(miembro)

        self.db.commit()

    def obtener_por_id(self, grupo_id: str) -> Optional[GrupoCompartido]:
        g = self.db.query(Grupo).filter(Grupo.id == grupo_id).first()
        if not g:
            return None

        miembros_aceptados = self.db.query(MiembroGrupo).filter(
            MiembroGrupo.grupo_id == grupo_id,
            MiembroGrupo.estado == "ACEPTADO"
        ).all()
        id_miembros = tuple(m.usuario_id for m in miembros_aceptados)

        return GrupoCompartido(
            id=g.id,
            nombre_grupo=g.nombre,
            moneda=g.moneda,
            id_miembros=id_miembros,
            id_administrador=g.creador_id,
            descripcion=g.descripcion or "",
            actividad=g.actividad_fecha or "",
        )

    def listar_de_usuario(self, usuario_id: str) -> list[GrupoCompartido]:
        membresias = self.db.query(MiembroGrupo).filter(
            MiembroGrupo.usuario_id == usuario_id,
            MiembroGrupo.estado == "ACEPTADO"
        ).all()
        
        grupos = []
        for m in membresias:
            g = self.obtener_por_id(m.grupo_id)
            if g:
                grupos.append(g)
        return grupos

    def listar_invitaciones_recibidas(self, usuario_id: str) -> list[InvitacionDominio]:
        membresias = self.db.query(MiembroGrupo).filter(
            MiembroGrupo.usuario_id == usuario_id,
            MiembroGrupo.estado == "PENDIENTE"
        ).all()

        invitaciones = []
        for m in membresias:
            g = self.db.query(Grupo).filter(Grupo.id == m.grupo_id).first()
            if g:
                invitaciones.append(
                    InvitacionDominio(
                        id=m.id,
                        grupo_id=m.grupo_id,
                        invitado_id=m.usuario_id,
                        invitador_id=g.creador_id,
                        estado=EstadoInvitacionGrupo.PENDIENTE,
                    )
                )
        return invitaciones

    def listar_invitaciones_de_grupo(self, grupo_id: str) -> list[InvitacionDominio]:
        membresias = self.db.query(MiembroGrupo).filter(
            MiembroGrupo.grupo_id == grupo_id,
            MiembroGrupo.estado == "PENDIENTE"
        ).all()
        
        g = self.db.query(Grupo).filter(Grupo.id == grupo_id).first()
        invitador_id = g.creador_id if g else ""

        return [
            InvitacionDominio(
                id=m.id,
                grupo_id=m.grupo_id,
                invitado_id=m.usuario_id,
                invitador_id=invitador_id,
                estado=EstadoInvitacionGrupo.PENDIENTE,
            )
            for m in membresias
        ]

    def guardar_invitacion(self, invitacion: InvitacionDominio) -> None:
        existente = self.db.query(MiembroGrupo).filter(
            MiembroGrupo.grupo_id == invitacion.grupo_id,
            MiembroGrupo.usuario_id == invitacion.invitado_id
        ).first()

        if existente:
            if existente.estado == "ACEPTADO":
                raise ErrorConflicto("invitado", "El usuario ya es miembro del grupo.")
            if existente.estado == "PENDIENTE":
                raise ErrorConflicto("invitado", "Ya existe una invitación pendiente para este usuario.")

        nuevo = MiembroGrupo(
            id=invitacion.id,
            grupo_id=invitacion.grupo_id,
            usuario_id=invitacion.invitado_id,
            rol="MIEMBRO",
            estado="PENDIENTE",
        )
        self.db.add(nuevo)
        self.db.commit()

    def obtener_invitacion_por_id(self, invitacion_id: str) -> Optional[InvitacionDominio]:
        m = self.db.query(MiembroGrupo).filter(MiembroGrupo.id == invitacion_id).first()
        if not m:
            return None

        g = self.db.query(Grupo).filter(Grupo.id == m.grupo_id).first()
        invitador = g.creador_id if g else ""

        estado_map = {
            "PENDIENTE": EstadoInvitacionGrupo.PENDIENTE,
            "ACEPTADO": EstadoInvitacionGrupo.ACEPTADA,
            "RECHAZADO": EstadoInvitacionGrupo.RECHAZADA,
        }

        return InvitacionDominio(
            id=m.id,
            grupo_id=m.grupo_id,
            invitado_id=m.usuario_id,
            invitador_id=invitador,
            estado=estado_map.get(m.estado, EstadoInvitacionGrupo.PENDIENTE),
        )

    def actualizar_respuesta_invitacion(self, invitacion: InvitacionDominio) -> None:
        m = self.db.query(MiembroGrupo).filter(MiembroGrupo.id == invitacion.id).with_for_update().first()
        if not m or m.estado != "PENDIENTE":
            raise ErrorConflicto("invitacion", "La invitación ya fue respondida.")

        if invitacion.estado == EstadoInvitacionGrupo.ACEPTADA:
            m.estado = "ACEPTADO"
        else:
            self.db.delete(m)

        self.db.commit()

