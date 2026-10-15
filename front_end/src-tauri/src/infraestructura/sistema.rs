use std::time::{SystemTime, UNIX_EPOCH};

use crate::aplicacion::puertos::{GeneradorId, Reloj};

pub struct RelojSistema;

impl Reloj for RelojSistema {
    fn ahora(&self) -> String {
        let instante = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("El reloj del sistema debe estar después de UNIX_EPOCH");
        formatear_utc(instante.as_secs(), instante.subsec_millis())
    }
}

pub struct GeneradorIdUuid;

impl GeneradorId for GeneradorIdUuid {
    fn nuevo_id(&mut self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

fn formatear_utc(segundos: u64, milisegundos: u32) -> String {
    let dias = segundos / 86_400;
    let segundos_del_dia = segundos % 86_400;
    let (anio, mes, dia) = fecha_desde_dias(dias as i64);
    let hora = segundos_del_dia / 3_600;
    let minuto = (segundos_del_dia % 3_600) / 60;
    let segundo = segundos_del_dia % 60;

    format!("{anio:04}-{mes:02}-{dia:02}T{hora:02}:{minuto:02}:{segundo:02}.{milisegundos:03}Z")
}

fn fecha_desde_dias(dias_desde_epoch: i64) -> (i64, i64, i64) {
    let dias = dias_desde_epoch + 719_468;
    let era = dias / 146_097;
    let dia_de_era = dias - era * 146_097;
    let anio_de_era =
        (dia_de_era - dia_de_era / 1_460 + dia_de_era / 36_524 - dia_de_era / 146_096) / 365;
    let mut anio = anio_de_era + era * 400;
    let dia_del_anio = dia_de_era - (365 * anio_de_era + anio_de_era / 4 - anio_de_era / 100);
    let mes_de_marzo = (5 * dia_del_anio + 2) / 153;
    let dia = dia_del_anio - (153 * mes_de_marzo + 2) / 5 + 1;
    let mes = mes_de_marzo + if mes_de_marzo < 10 { 3 } else { -9 };
    anio += i64::from(mes <= 2);

    (anio, mes, dia)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatea_instantes_utc_con_fecha_iso_8601() {
        assert_eq!(formatear_utc(0, 0), "1970-01-01T00:00:00.000Z");
        assert_eq!(formatear_utc(1_792_022_400, 42), "2026-10-15T00:00:00.042Z");
    }

    #[test]
    fn genera_identificadores_uuid_distintos() {
        let mut generador = GeneradorIdUuid;
        assert_ne!(generador.nuevo_id(), generador.nuevo_id());
    }
}
