use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{rand_core::RngCore, Aead, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use zeroize::Zeroize;

use crate::aplicacion::puertos::sincronizacion::{CifradorLibro, ErrorCifradoLibro};

/// Mensaje de error legible exigido por RNF-18 / ESC-03 ante fallo de descifrado o manipulación.
pub const MENSAJE_ERROR_DESCIFRADO: &str = "Frase de paso incorrecta o archivo dañado.";

const MAGIC_ENCABEZADO: &[u8] = b"CC_CIFRADO_V1\n";
const VERSION_CIFRADO: u8 = 1;
const ALGORITMO_ARGON2ID_XCHACHA20: u8 = 1;

// Parámetros recomendados por OWASP para Argon2id en escritorio: 19 MiB, 2 iteraciones, 1 hilo.
const ARGON2_M_COST_KIB: u32 = 19 * 1024;
const ARGON2_T_COST: u32 = 2;
const ARGON2_P_COST: u32 = 1;

const TAMANO_SAL: usize = 16;
const TAMANO_NONCE: usize = 24;
const TAMANO_CLAVE: usize = 32;
const TAMANO_TAG_AEAD: usize = 16;

// 14 (magic) + 1 (version) + 1 (alg) + 4 (m) + 4 (t) + 4 (p) + 16 (sal) + 24 (nonce) = 68 bytes.
const TAMANO_ENCABEZADO: usize = MAGIC_ENCABEZADO.len() + 1 + 1 + 4 + 4 + 4 + TAMANO_SAL + TAMANO_NONCE;

/// Adaptador de cifrado autenticado para copias de seguridad y lotes de sincronización.
///
/// Usa Argon2id para derivar una clave de 256 bits a partir de la frase de paso y una sal
/// aleatoria única, y XChaCha20-Poly1305 para cifrado autenticado vinculando el encabezado
/// en claro como AAD (*Additional Authenticated Data*).
#[derive(Debug, Default, Clone, Copy)]
pub struct CifradorArgon2XChaCha20;

impl CifradorArgon2XChaCha20 {
    pub fn nuevo() -> Self {
        Self
    }

    fn derivar_clave(
        frase_paso: &str,
        sal: &[u8; TAMANO_SAL],
        m_cost: u32,
        t_cost: u32,
        p_cost: u32,
    ) -> Result<[u8; TAMANO_CLAVE], ErrorCifradoLibro> {
        let params = Params::new(m_cost, t_cost, p_cost, Some(TAMANO_CLAVE))
            .map_err(|_| ErrorCifradoLibro::OperacionFallida)?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut clave = [0u8; TAMANO_CLAVE];
        argon2
            .hash_password_into(frase_paso.as_bytes(), sal, &mut clave)
            .map_err(|_| ErrorCifradoLibro::OperacionFallida)?;
        Ok(clave)
    }

    fn construir_encabezado(
        m_cost: u32,
        t_cost: u32,
        p_cost: u32,
        sal: &[u8; TAMANO_SAL],
        nonce: &[u8; TAMANO_NONCE],
    ) -> Vec<u8> {
        let mut enc = Vec::with_capacity(TAMANO_ENCABEZADO);
        enc.extend_from_slice(MAGIC_ENCABEZADO);
        enc.push(VERSION_CIFRADO);
        enc.push(ALGORITMO_ARGON2ID_XCHACHA20);
        enc.extend_from_slice(&m_cost.to_le_bytes());
        enc.extend_from_slice(&t_cost.to_le_bytes());
        enc.extend_from_slice(&p_cost.to_le_bytes());
        enc.extend_from_slice(sal);
        enc.extend_from_slice(nonce);
        enc
    }
}

impl CifradorLibro for CifradorArgon2XChaCha20 {
    fn esta_cifrado(&self, contenido: &[u8]) -> bool {
        contenido.starts_with(MAGIC_ENCABEZADO)
    }

    fn cifrar(&self, contenido: &[u8], frase_paso: &str) -> Result<Vec<u8>, ErrorCifradoLibro> {
        if frase_paso.is_empty() {
            return Err(ErrorCifradoLibro::OperacionFallida);
        }

        let mut sal = [0u8; TAMANO_SAL];
        let mut nonce_bytes = [0u8; TAMANO_NONCE];
        OsRng.fill_bytes(&mut sal);
        OsRng.fill_bytes(&mut nonce_bytes);

        let mut clave = Self::derivar_clave(
            frase_paso,
            &sal,
            ARGON2_M_COST_KIB,
            ARGON2_T_COST,
            ARGON2_P_COST,
        )?;

        let encabezado = Self::construir_encabezado(
            ARGON2_M_COST_KIB,
            ARGON2_T_COST,
            ARGON2_P_COST,
            &sal,
            &nonce_bytes,
        );

        let cipher = XChaCha20Poly1305::new(Key::from_slice(&clave));
        let nonce = XNonce::from_slice(&nonce_bytes);
        let resultado_cifrado = cipher.encrypt(
            nonce,
            Payload {
                msg: contenido,
                aad: &encabezado,
            },
        );
        clave.zeroize();

        let ciphertext = resultado_cifrado.map_err(|_| ErrorCifradoLibro::OperacionFallida)?;

        let mut salida = Vec::with_capacity(encabezado.len() + ciphertext.len());
        salida.extend_from_slice(&encabezado);
        salida.extend_from_slice(&ciphertext);
        Ok(salida)
    }

    fn descifrar(
        &self,
        contenido: &[u8],
        frase_paso: &str,
    ) -> Result<Vec<u8>, ErrorCifradoLibro> {
        if frase_paso.is_empty()
            || !self.esta_cifrado(contenido)
            || contenido.len() < TAMANO_ENCABEZADO + TAMANO_TAG_AEAD
        {
            return Err(ErrorCifradoLibro::FraseIncorrectaODatosDanados);
        }

        let (encabezado, ciphertext) = contenido.split_at(TAMANO_ENCABEZADO);
        let mut offset = MAGIC_ENCABEZADO.len();

        let version = encabezado[offset];
        offset += 1;
        let algoritmo = encabezado[offset];
        offset += 1;
        if version != VERSION_CIFRADO || algoritmo != ALGORITMO_ARGON2ID_XCHACHA20 {
            return Err(ErrorCifradoLibro::FraseIncorrectaODatosDanados);
        }

        let m_cost = u32::from_le_bytes(
            encabezado[offset..offset + 4]
                .try_into()
                .map_err(|_| ErrorCifradoLibro::FraseIncorrectaODatosDanados)?,
        );
        offset += 4;
        let t_cost = u32::from_le_bytes(
            encabezado[offset..offset + 4]
                .try_into()
                .map_err(|_| ErrorCifradoLibro::FraseIncorrectaODatosDanados)?,
        );
        offset += 4;
        let p_cost = u32::from_le_bytes(
            encabezado[offset..offset + 4]
                .try_into()
                .map_err(|_| ErrorCifradoLibro::FraseIncorrectaODatosDanados)?,
        );
        offset += 4;

        let sal: [u8; TAMANO_SAL] = encabezado[offset..offset + TAMANO_SAL]
            .try_into()
            .map_err(|_| ErrorCifradoLibro::FraseIncorrectaODatosDanados)?;
        offset += TAMANO_SAL;

        let nonce_bytes: [u8; TAMANO_NONCE] = encabezado[offset..offset + TAMANO_NONCE]
            .try_into()
            .map_err(|_| ErrorCifradoLibro::FraseIncorrectaODatosDanados)?;

        let mut clave = Self::derivar_clave(frase_paso, &sal, m_cost, t_cost, p_cost)
            .map_err(|_| ErrorCifradoLibro::FraseIncorrectaODatosDanados)?;

        let cipher = XChaCha20Poly1305::new(Key::from_slice(&clave));
        let nonce = XNonce::from_slice(&nonce_bytes);
        let resultado = cipher.decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad: encabezado,
            },
        );
        clave.zeroize();

        resultado.map_err(|_| ErrorCifradoLibro::FraseIncorrectaODatosDanados)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRASE_VALIDA: &str = "mi-frase-secreta-muy-segura-2026";
    const CONTENIDO_PRUEBA: &[u8] = b"CUENTAS_CLARAS_LIBRO v1\nFECHA_EXPORTACION: 2026-11-25T18:00:00Z\nCANTIDAD_EVENTOS: 1\n---\n{\"clase_evento\":\"CuentaCreada\",\"nombre\":\"Efectivo USD\"}";

    #[test]
    fn round_trip_cifrar_y_descifrar() {
        let cifrador = CifradorArgon2XChaCha20::nuevo();

        let cifrado = cifrador
            .cifrar(CONTENIDO_PRUEBA, FRASE_VALIDA)
            .expect("Debe cifrar");
        assert!(cifrador.esta_cifrado(&cifrado));
        assert!(!cifrador.esta_cifrado(CONTENIDO_PRUEBA));

        let descifrado = cifrador
            .descifrar(&cifrado, FRASE_VALIDA)
            .expect("Debe descifrar");
        assert_eq!(descifrado, CONTENIDO_PRUEBA);
    }

    #[test]
    fn rechaza_frase_de_paso_incorrecta() {
        let cifrador = CifradorArgon2XChaCha20::nuevo();
        let cifrado = cifrador.cifrar(CONTENIDO_PRUEBA, FRASE_VALIDA).unwrap();

        let err = cifrador
            .descifrar(&cifrado, "frase-totalmente-equivocada")
            .unwrap_err();
        assert_eq!(err, ErrorCifradoLibro::FraseIncorrectaODatosDanados);
    }

    #[test]
    fn rechaza_archivo_alterado_byte_a_byte() {
        let cifrador = CifradorArgon2XChaCha20::nuevo();
        let cifrado = cifrador.cifrar(CONTENIDO_PRUEBA, FRASE_VALIDA).unwrap();

        // Alterar un byte del ciphertext
        let mut alterado_cuerpo = cifrado.clone();
        let ultimo = alterado_cuerpo.len() - 1;
        alterado_cuerpo[ultimo] ^= 0xFF;
        assert_eq!(
            cifrador.descifrar(&alterado_cuerpo, FRASE_VALIDA).unwrap_err(),
            ErrorCifradoLibro::FraseIncorrectaODatosDanados
        );

        // Alterar un byte del encabezado (sal/nonce) también debe fallar por AAD
        let mut alterado_header = cifrado;
        alterado_header[MAGIC_ENCABEZADO.len() + 14] ^= 0x01;
        assert_eq!(
            cifrador.descifrar(&alterado_header, FRASE_VALIDA).unwrap_err(),
            ErrorCifradoLibro::FraseIncorrectaODatosDanados
        );
    }

    #[test]
    fn archivo_cifrado_no_revela_datos_ni_frase_de_paso() {
        let cifrador = CifradorArgon2XChaCha20::nuevo();
        let cifrado = cifrador.cifrar(CONTENIDO_PRUEBA, FRASE_VALIDA).unwrap();

        let contiene = |aguja: &[u8]| cifrado.windows(aguja.len()).any(|w| w == aguja);
        assert!(!contiene(b"Efectivo USD"));
        assert!(!contiene(b"CuentaCreada"));
        assert!(!contiene(b"CANTIDAD_EVENTOS"));
        assert!(!contiene(FRASE_VALIDA.as_bytes()));
    }

    #[test]
    fn cada_cifrado_usa_sal_y_nonce_distintos() {
        let cifrador = CifradorArgon2XChaCha20::nuevo();
        let c1 = cifrador.cifrar(CONTENIDO_PRUEBA, FRASE_VALIDA).unwrap();
        let c2 = cifrador.cifrar(CONTENIDO_PRUEBA, FRASE_VALIDA).unwrap();
        assert_ne!(c1, c2);
    }
}
