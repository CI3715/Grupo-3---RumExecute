pub mod buscar_usuario;
pub mod cambiar_moneda_base;
pub mod cerrar_sesion;
pub mod consultar_patrimonio;
pub mod corregir_movimiento;
pub mod crear_categoria;
pub mod crear_cuenta;
pub mod crear_grupo;
pub mod crear_moneda;
pub mod dto;
pub mod eliminar_amistad;
pub mod eliminar_tasa;
pub mod enviar_invitacion_amistad;
pub mod iniciar_sesion;
pub mod invitar_miembro_grupo;
pub mod listar_amistades;
pub mod listar_categorias;
pub mod listar_cuentas;
pub mod listar_historial_tasas;
pub mod listar_mis_grupos;
pub mod listar_monedas;
pub mod obtener_detalle_cuenta;
pub mod obtener_detalle_grupo;
pub mod obtener_perfil_propio;
pub mod proyeccion_monedas;
pub mod puertos;
pub mod registrar_gasto_compartido;
pub mod registrar_movimiento;
pub mod registrar_tasa;
pub mod registrar_usuario;
pub mod responder_invitacion_amistad;
pub mod responder_invitacion_grupo;
pub mod restaurar_sesion;

#[cfg(test)]
mod pruebas_entrega4;

#[cfg(test)]
mod pruebas;
