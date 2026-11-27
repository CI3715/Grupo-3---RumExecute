"use client";

import React, { useState, useEffect, useCallback } from "react";
import { downloadDir, join } from "@tauri-apps/api/path";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  ConfiguracionSyncDispositivos,
  InformeSincronizacionDispositivos,
  ResumenImportacion,
  TipoDestinoSync,
  actualizarCredencialesGit,
  exportarCopiaSeguridad,
  guardarConfiguracionSync,
  importarCopiaSeguridad,
  obtenerConfiguracionSync,
  sincronizarDispositivosAhora,
} from "../services/respaldoService";

interface VistaRespaldoProps {
  onLibroActualizado: () => Promise<void> | void;
}

const cardStyle: React.CSSProperties = {
  backgroundColor: "var(--surface-card)",
  border: "1px solid var(--border-subtle)",
  borderRadius: "14px",
  padding: "20px 24px",
};

const enTauri = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export default function VistaRespaldo({ onLibroActualizado }: VistaRespaldoProps) {
  const [subTab, setSubTab] = useState<"copias" | "dispositivos">("copias");
  const [aviso, setAviso] = useState<{ tipo: "exito" | "error"; texto: string } | null>(null);

  // Estado de Exportación
  const [modoCifrado, setModoCifrado] = useState<boolean>(true);
  const [rutaExportar, setRutaExportar] = useState("");
  const [fraseExport1, setFraseExport1] = useState("");
  const [fraseExport2, setFraseExport2] = useState("");
  const [errorFraseExport1, setErrorFraseExport1] = useState<string | null>(null);
  const [errorFraseExport2, setErrorFraseExport2] = useState<string | null>(null);
  const [confirmarExportSinCifrar, setConfirmarExportSinCifrar] = useState(false);
  const [exportando, setExportando] = useState(false);
  const [ultimoExport, setUltimoExport] = useState<{
    ruta: string;
    eventos: number;
    cifrado: boolean;
  } | null>(null);

  // Estado de Importación
  const [rutaImportar, setRutaImportar] = useState("");
  const [archivoContenidoWeb, setArchivoContenidoWeb] = useState("");
  const [archivoNombreWeb, setArchivoNombreWeb] = useState("");
  const [fraseImportar, setFraseImportar] = useState("");
  const [errorFraseImportar, setErrorFraseImportar] = useState<string | null>(null);
  const [confirmarImportacion, setConfirmarImportacion] = useState(false);
  const [importando, setImportando] = useState(false);
  const [resumenImportacion, setResumenImportacion] = useState<ResumenImportacion | null>(null);

  // Estado de Sincronización entre dispositivos
  const [configSync, setConfigSync] = useState<ConfiguracionSyncDispositivos | null>(null);
  const [editandoDestino, setEditandoDestino] = useState(false);
  const [tipoDestino, setTipoDestino] = useState<TipoDestinoSync>("git");
  const [rutaOUrl, setRutaOUrl] = useState("");
  const [usuarioGit, setUsuarioGit] = useState("");
  const [tokenGit, setTokenGit] = useState("");
  const [fraseSync, setFraseSync] = useState("");
  const [guardandoDestino, setGuardandoDestino] = useState(false);

  // Actualización rápida de credenciales Git
  const [mostrarFormCredenciales, setMostrarFormCredenciales] = useState(false);
  const [credUsuario, setCredUsuario] = useState("");
  const [credToken, setCredToken] = useState("");
  const [actualizandoCreds, setActualizandoCreds] = useState(false);

  // Sincronización ahora
  const [sincronizando, setSincronizando] = useState(false);
  const [informeSync, setInformeSync] = useState<InformeSincronizacionDispositivos | null>(null);

  const cargarConfig = useCallback(async () => {
    const cfg = await obtenerConfiguracionSync();
    setConfigSync(cfg);
    if (cfg?.configurado) {
      setTipoDestino(cfg.tipo);
      setRutaOUrl(cfg.ruta_o_url);
      setUsuarioGit(cfg.usuario_git || "");
    }
  }, []);

  useEffect(() => {
    cargarConfig();
    if (enTauri()) {
      const hoy = new Date().toISOString().slice(0, 10);
      downloadDir()
        .then((dir) => join(dir, `cuentas-claras-respaldo-${hoy}.ccbak`))
        .then((ruta) => {
          setRutaExportar((prev) => prev || ruta);
          setRutaImportar((prev) => prev || ruta);
        })
        .catch(() => {});
    }
  }, [cargarConfig]);

  const handleExportar = async (e: React.FormEvent) => {
    e.preventDefault();
    setAviso(null);
    setErrorFraseExport1(null);
    setErrorFraseExport2(null);

    if (modoCifrado) {
      if (fraseExport1.length < 12) {
        setErrorFraseExport1("La frase de paso debe tener al menos 12 caracteres.");
        return;
      }
      if (fraseExport1 !== fraseExport2) {
        setErrorFraseExport2("Las dos frases de paso no coinciden.");
        return;
      }
    } else if (!confirmarExportSinCifrar) {
      setConfirmarExportSinCifrar(true);
      return;
    }

    setExportando(true);
    try {
      const res = await exportarCopiaSeguridad({
        ruta_destino: rutaExportar.trim() || undefined,
        cifrado: modoCifrado,
        frase_paso: modoCifrado ? fraseExport1 : undefined,
        confirmacion_frase_paso: modoCifrado ? fraseExport2 : undefined,
      });

      if (res.contenido_descarga && typeof window !== "undefined") {
        const blob = new Blob([res.contenido_descarga], { type: "text/plain;charset=utf-8" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = res.ruta_o_nombre;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
      }

      setUltimoExport({
        ruta: res.ruta_o_nombre,
        eventos: res.cantidad_eventos,
        cifrado: res.cifrado,
      });
      setFraseExport1("");
      setFraseExport2("");
      setConfirmarExportSinCifrar(false);
      setAviso({
        tipo: "exito",
        texto: `Copia de seguridad generada (${res.cantidad_eventos} eventos).`,
      });
    } catch (err: any) {
      const msg = err?.message || "No se pudo exportar la copia.";
      if (msg.toLowerCase().includes("coinciden")) {
        setErrorFraseExport2(msg);
      } else if (msg.toLowerCase().includes("frase")) {
        setErrorFraseExport1(msg);
      } else {
        setAviso({ tipo: "error", texto: msg });
      }
    } finally {
      setExportando(false);
    }
  };

  const handleSeleccionarArchivoWeb = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    setArchivoNombreWeb(file.name);
    setRutaImportar(file.name);
    setConfirmarImportacion(false);
    const reader = new FileReader();
    reader.onload = () => {
      setArchivoContenidoWeb(typeof reader.result === "string" ? reader.result : "");
    };
    reader.readAsText(file);
  };

  const handleImportar = async (e: React.FormEvent) => {
    e.preventDefault();
    setAviso(null);
    setErrorFraseImportar(null);

    if (enTauri() && !rutaImportar.trim()) {
      setAviso({ tipo: "error", texto: "Indica la ruta del archivo de respaldo a importar." });
      return;
    }
    if (!enTauri() && !archivoContenidoWeb) {
      setAviso({ tipo: "error", texto: "Selecciona primero el archivo de respaldo." });
      return;
    }

    if (!confirmarImportacion) {
      setConfirmarImportacion(true);
      return;
    }

    setImportando(true);
    setResumenImportacion(null);
    try {
      const res = await importarCopiaSeguridad({
        ruta_origen: rutaImportar.trim() || undefined,
        contenido_archivo: archivoContenidoWeb || undefined,
        nombre_archivo: archivoNombreWeb || undefined,
        frase_paso: fraseImportar || undefined,
      });
      setResumenImportacion(res);
      setConfirmarImportacion(false);
      setFraseImportar("");
      await onLibroActualizado();
      setAviso({
        tipo: "exito",
        texto: `Importación completada: ${res.eventos_nuevos} nuevos, ${res.eventos_existentes} ya existían.`,
      });
    } catch (err: any) {
      setConfirmarImportacion(false);
      const msg = err?.message || "Frase de paso incorrecta o archivo dañado.";
      if (msg.toLowerCase().includes("frase")) {
        setErrorFraseImportar(msg);
      } else {
        setAviso({ tipo: "error", texto: msg });
      }
    } finally {
      setImportando(false);
    }
  };

  const handleGuardarDestino = async (e: React.FormEvent) => {
    e.preventDefault();
    setAviso(null);
    if (!rutaOUrl.trim()) {
      setAviso({
        tipo: "error",
        texto:
          tipoDestino === "git"
            ? "Ingresa la URL del repositorio Git privado."
            : "Ingresa la ruta de la carpeta compartida.",
      });
      return;
    }
    if (fraseSync.length < 12) {
      setAviso({
        tipo: "error",
        texto: "La frase de paso de sincronización debe tener al menos 12 caracteres.",
      });
      return;
    }

    setGuardandoDestino(true);
    try {
      const cfg = await guardarConfiguracionSync({
        tipo: tipoDestino,
        ruta_o_url: rutaOUrl.trim(),
        usuario_git: tipoDestino === "git" ? usuarioGit.trim() : undefined,
        token_o_clave_git: tipoDestino === "git" ? tokenGit.trim() : undefined,
        frase_paso: fraseSync,
      });
      setConfigSync(cfg);
      setEditandoDestino(false);
      setTokenGit("");
      setFraseSync("");
      setAviso({
        tipo: "exito",
        texto: "Destino de sincronización guardado de forma segura en este dispositivo.",
      });
    } catch (err: any) {
      setAviso({ tipo: "error", texto: err?.message || "No se pudo guardar el destino." });
    } finally {
      setGuardandoDestino(false);
    }
  };

  const handleSincronizarAhora = async () => {
    setAviso(null);
    setSincronizando(true);
    try {
      const inf = await sincronizarDispositivosAhora();
      setInformeSync(inf);
      await cargarConfig();
      await onLibroActualizado();
      setAviso({
        tipo: "exito",
        texto: `Sincronización completada: ${inf.enviados} enviados, ${inf.recibidos} recibidos.`,
      });
    } catch (err: any) {
      const msg = err?.message || "No se pudo completar la sincronización.";
      if (msg.toLowerCase().includes("credencial") || msg.toLowerCase().includes("autentica")) {
        setMostrarFormCredenciales(true);
      }
      setAviso({ tipo: "error", texto: msg });
    } finally {
      setSincronizando(false);
    }
  };

  const handleActualizarCredenciales = async (e: React.FormEvent) => {
    e.preventDefault();
    setAviso(null);
    setActualizandoCreds(true);
    try {
      await actualizarCredencialesGit({
        usuario_git: credUsuario,
        token_o_clave_git: credToken,
      });
      setMostrarFormCredenciales(false);
      setCredToken("");
      setAviso({
        tipo: "exito",
        texto: "Credenciales de Git actualizadas en el almacén seguro del sistema.",
      });
      await cargarConfig();
    } catch (err: any) {
      setAviso({ tipo: "error", texto: err?.message || "No se pudieron actualizar las credenciales." });
    } finally {
      setActualizandoCreds(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "20px" }}>
      <div className="hero-section">
        <div>
          <h1 className="hero-title">Copia de Seguridad y Sincronización</h1>
          <p className="hero-subtitle">
            Respalda tu libro personal o sincronízalo entre tus propios dispositivos sin pasar por el servidor.
          </p>
        </div>
        <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
          <button
            type="button"
            onClick={() => {
              setSubTab("copias");
              setAviso(null);
            }}
            className={subTab === "copias" ? "btn btn-primary" : "btn btn-secondary"}
          >
            Exportar e Importar
          </button>
          <button
            type="button"
            onClick={() => {
              setSubTab("dispositivos");
              setAviso(null);
            }}
            className={subTab === "dispositivos" ? "btn btn-primary" : "btn btn-secondary"}
          >
            Sincronizar Dispositivos
          </button>
        </div>
      </div>

      {aviso && (
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            padding: "12px 16px",
            borderRadius: "10px",
            fontSize: "13px",
            fontWeight: 600,
            backgroundColor: aviso.tipo === "exito" ? "#ECFDF5" : "#FEF2F2",
            color: aviso.tipo === "exito" ? "#065F46" : "#991B1B",
            border: `1px solid ${aviso.tipo === "exito" ? "#A7F3D0" : "#FCA5A5"}`,
          }}
        >
          <span>{aviso.texto}</span>
          <button
            type="button"
            onClick={() => setAviso(null)}
            style={{ background: "none", border: "none", fontWeight: 700, cursor: "pointer", color: "inherit" }}
          >
            ✕
          </button>
        </div>
      )}

      {subTab === "copias" && (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(340px, 1fr))", gap: "18px" }}>
          {/* Exportar copia de seguridad */}
          <form onSubmit={handleExportar} style={{ ...cardStyle, display: "flex", flexDirection: "column", gap: "14px" }}>
            <div>
              <h2 style={{ fontSize: "17px", fontWeight: 800 }}>Exportar copia de seguridad</h2>
              <p style={{ fontSize: "12.5px", color: "var(--text-secondary)", marginTop: "3px" }}>
                Genera un archivo único con todo tu libro personal sin incluir contraseñas ni sesiones.
              </p>
            </div>

            <div className="tab-grid" style={{ margin: 0 }}>
              <button
                type="button"
                onClick={() => {
                  setModoCifrado(true);
                  setConfirmarExportSinCifrar(false);
                  setRutaExportar((prev) => prev.replace(/\.txt$/i, ".ccbak"));
                }}
                className={`tab-btn ${modoCifrado ? "active-income" : ""}`}
              >
                Archivo cifrado (Recomendado)
              </button>
              <button
                type="button"
                onClick={() => {
                  setModoCifrado(false);
                  setConfirmarExportSinCifrar(false);
                  setRutaExportar((prev) => prev.replace(/\.ccbak$/i, ".txt"));
                }}
                className={`tab-btn ${!modoCifrado ? "active-expense" : ""}`}
              >
                Texto sin cifrar
              </button>
            </div>

            {enTauri() && (
              <div className="form-group">
                <label className="form-label">Ruta de guardado</label>
                <input
                  type="text"
                  value={rutaExportar}
                  onChange={(e) => setRutaExportar(e.target.value)}
                  placeholder={
                    modoCifrado
                      ? "Ej. C:\\Respaldos\\cuentas-claras.ccbak"
                      : "Ej. C:\\Respaldos\\cuentas-claras.txt"
                  }
                  className="form-input"
                  disabled={exportando}
                  required
                />
              </div>
            )}

            {modoCifrado ? (
              <>
                <div className="info-box">
                  <span>🔒</span>
                  <p>
                    Usa una frase de al menos <strong>12 caracteres</strong>. Si la olvidas,{" "}
                    <strong>el respaldo no se podrá recuperar</strong>.
                  </p>
                </div>

                <div className="form-group">
                  <label className="form-label">Frase de paso</label>
                  <input
                    type="password"
                    value={fraseExport1}
                    onChange={(e) => {
                      setFraseExport1(e.target.value);
                      setErrorFraseExport1(null);
                    }}
                    placeholder="Mínimo 12 caracteres"
                    className={`form-input ${errorFraseExport1 ? "input-error" : ""}`}
                    disabled={exportando}
                    required
                  />
                  {errorFraseExport1 && <span className="error-text">{errorFraseExport1}</span>}
                </div>

                <div className="form-group">
                  <label className="form-label">Repite la frase de paso</label>
                  <input
                    type="password"
                    value={fraseExport2}
                    onChange={(e) => {
                      setFraseExport2(e.target.value);
                      setErrorFraseExport2(null);
                    }}
                    placeholder="Confirma tu frase de paso"
                    className={`form-input ${errorFraseExport2 ? "input-error" : ""}`}
                    disabled={exportando}
                    required
                  />
                  {errorFraseExport2 && <span className="error-text">{errorFraseExport2}</span>}
                </div>
              </>
            ) : (
              <div className="warning-box">
                <strong>Advertencia de seguridad</strong>
                <p>
                  El archivo sin cifrar será legible por cualquier persona que tenga acceso a él. Úsalo solo para auditoría o revisión personal.
                </p>
              </div>
            )}

            {confirmarExportSinCifrar && !modoCifrado ? (
              <div
                style={{
                  padding: "12px",
                  borderRadius: "10px",
                  backgroundColor: "#FEF2F2",
                  border: "1px solid #FCA5A5",
                  display: "flex",
                  flexDirection: "column",
                  gap: "10px",
                }}
              >
                <span style={{ fontSize: "12.5px", fontWeight: 700, color: "#991B1B" }}>
                  ¿Confirmas exportar tu libro financiero en texto legible sin cifrado?
                </span>
                <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
                  <button
                    type="button"
                    className="btn btn-secondary"
                    onClick={() => setConfirmarExportSinCifrar(false)}
                    disabled={exportando}
                  >
                    Cancelar
                  </button>
                  <button type="submit" className="btn btn-danger" disabled={exportando}>
                    {exportando ? "Exportando..." : "Confirmar exportación sin cifrar"}
                  </button>
                </div>
              </div>
            ) : (
              <button type="submit" className="btn btn-primary" disabled={exportando}>
                {exportando ? "Generando copia..." : "Exportar copia de seguridad"}
              </button>
            )}

            {ultimoExport && (
              <div style={{ fontSize: "12px", color: "var(--text-secondary)", borderTop: "1px solid var(--border-subtle)", paddingTop: "10px", display: "flex", justifyContent: "space-between", alignItems: "center", gap: "10px", flexWrap: "wrap" }}>
                <span>
                  Última exportación: <strong>{ultimoExport.ruta}</strong> ({ultimoExport.eventos} eventos,{" "}
                  {ultimoExport.cifrado ? "cifrada" : "sin cifrar"}).
                </span>
                {enTauri() && (
                  <button
                    type="button"
                    className="btn btn-secondary"
                    style={{ fontSize: "11.5px", padding: "4px 10px" }}
                    onClick={() => {
                      revealItemInDir(ultimoExport.ruta).catch(() => {});
                    }}
                  >
                    Ver en carpeta
                  </button>
                )}
              </div>
            )}
          </form>

          {/* Importar copia de seguridad */}
          <form onSubmit={handleImportar} style={{ ...cardStyle, display: "flex", flexDirection: "column", gap: "14px" }}>
            <div>
              <h2 style={{ fontSize: "17px", fontWeight: 800 }}>Importar copia</h2>
              <p style={{ fontSize: "12.5px", color: "var(--text-secondary)", marginTop: "3px" }}>
                Restaura un respaldo o fusiona eventos de otra copia sin borrar ni duplicar tus movimientos actuales.
              </p>
            </div>

            {enTauri() ? (
              <div className="form-group">
                <label className="form-label">Ruta del archivo de respaldo</label>
                <input
                  type="text"
                  value={rutaImportar}
                  onChange={(e) => {
                    setRutaImportar(e.target.value);
                    setConfirmarImportacion(false);
                  }}
                  placeholder="Ej. C:\Respaldos\cuentas-claras.ccbak"
                  className="form-input"
                  disabled={importando}
                  required
                />
              </div>
            ) : (
              <div className="form-group">
                <label className="form-label">Archivo de respaldo (.ccbak o .txt)</label>
                <input
                  type="file"
                  accept=".ccbak,.txt"
                  onChange={handleSeleccionarArchivoWeb}
                  className="form-input"
                  disabled={importando}
                />
                {archivoNombreWeb && (
                  <span className="helper-text">Archivo seleccionado: {archivoNombreWeb}</span>
                )}
              </div>
            )}

            <div className="form-group">
              <label className="form-label">Frase de paso (si el archivo está cifrado)</label>
              <input
                type="password"
                value={fraseImportar}
                onChange={(e) => {
                  setFraseImportar(e.target.value);
                  setErrorFraseImportar(null);
                  setConfirmarImportacion(false);
                }}
                placeholder="Déjalo vacío si el respaldo no tiene cifrado"
                className={`form-input ${errorFraseImportar ? "input-error" : ""}`}
                disabled={importando}
              />
              {errorFraseImportar && <span className="error-text">{errorFraseImportar}</span>}
            </div>

            {confirmarImportacion ? (
              <div
                style={{
                  padding: "12px",
                  borderRadius: "10px",
                  backgroundColor: "#EFF6FF",
                  border: "1px solid #BFDBFE",
                  display: "flex",
                  flexDirection: "column",
                  gap: "10px",
                }}
              >
                <span style={{ fontSize: "12.5px", fontWeight: 700, color: "#1E40AF" }}>
                  ¿Confirmas importar los eventos de {rutaImportar || archivoNombreWeb} en tu libro personal?
                </span>
                <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
                  <button
                    type="button"
                    className="btn btn-secondary"
                    onClick={() => setConfirmarImportacion(false)}
                    disabled={importando}
                  >
                    Cancelar
                  </button>
                  <button type="submit" className="btn btn-blue" disabled={importando}>
                    {importando ? "Importando..." : "Confirmar importación"}
                  </button>
                </div>
              </div>
            ) : (
              <button type="submit" className="btn btn-secondary" disabled={importando}>
                {importando ? "Validando e importando..." : "Importar copia"}
              </button>
            )}

            {resumenImportacion && (
              <div
                style={{
                  borderTop: "1px solid var(--border-subtle)",
                  paddingTop: "12px",
                  display: "flex",
                  flexDirection: "column",
                  gap: "6px",
                  fontSize: "12.5px",
                }}
              >
                <strong>Resultado de la importación:</strong>
                <div style={{ display: "flex", gap: "12px", flexWrap: "wrap" }}>
                  <span style={{ color: "#065F46", fontWeight: 700 }}>
                    {resumenImportacion.eventos_nuevos} nuevos
                  </span>
                  <span style={{ color: "var(--text-secondary)", fontWeight: 600 }}>
                    {resumenImportacion.eventos_existentes} ya existían
                  </span>
                  <span
                    style={{
                      color: resumenImportacion.eventos_rechazados.length > 0 ? "#B91C1C" : "var(--text-muted)",
                      fontWeight: 700,
                    }}
                  >
                    {resumenImportacion.eventos_rechazados.length} rechazados
                  </span>
                </div>
                {resumenImportacion.eventos_rechazados.length > 0 && (
                  <ul style={{ margin: "4px 0 0 18px", color: "#991B1B", fontSize: "12px" }}>
                    {resumenImportacion.eventos_rechazados.map((r, i) => (
                      <li key={`${r.id_evento || "ev"}-${i}`}>
                        {r.id_evento ? `${r.id_evento}: ` : ""}
                        {r.motivo}
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}
          </form>
        </div>
      )}

      {subTab === "dispositivos" && (
        <div style={{ display: "flex", flexDirection: "column", gap: "18px" }}>
          {/* Estado actual y Sincronizar ahora */}
          <div style={{ ...cardStyle, display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: "16px" }}>
            <div>
              <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                <h2 style={{ fontSize: "17px", fontWeight: 800 }}>
                  {configSync?.configurado
                    ? configSync.tipo === "git"
                      ? "Repositorio Git privado"
                      : "Carpeta compartida"
                    : "Sin destino configurado"}
                </h2>
                {configSync?.configurado && (
                  <span
                    style={{
                      fontSize: "11px",
                      fontWeight: 700,
                      padding: "2px 9px",
                      borderRadius: "10px",
                      backgroundColor: "#ECFDF5",
                      color: "#065F46",
                    }}
                  >
                    Configurado
                  </span>
                )}
              </div>
              <p style={{ fontSize: "13px", color: "var(--text-secondary)", marginTop: "4px" }}>
                {configSync?.configurado
                  ? configSync.ruta_o_url
                  : "Configura un repositorio Git privado o una carpeta compartida (Syncthing, Drive, Dropbox) para mantener tus dispositivos al día."}
              </p>
              <p style={{ fontSize: "12px", color: "var(--text-muted)", marginTop: "4px" }}>
                {configSync?.fecha_ultima_sincronizacion
                  ? `Última sincronización exitosa: ${new Date(configSync.fecha_ultima_sincronizacion).toLocaleString("es-VE")}`
                  : "Aún no se ha sincronizado este dispositivo."}
              </p>
            </div>

            <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
              {configSync?.configurado && configSync.tipo === "carpeta" && enTauri() && (
                <button
                  type="button"
                  onClick={() => {
                    revealItemInDir(configSync.ruta_o_url).catch(() => {});
                  }}
                  className="btn btn-secondary"
                >
                  Ver carpeta
                </button>
              )}
              {configSync?.configurado && configSync.tipo === "git" && (
                <button
                  type="button"
                  onClick={() => setMostrarFormCredenciales(!mostrarFormCredenciales)}
                  className="btn btn-secondary"
                >
                  Actualizar credenciales Git
                </button>
              )}
              <button
                type="button"
                onClick={() => setEditandoDestino(!editandoDestino)}
                className="btn btn-secondary"
              >
                {editandoDestino
                  ? "Cerrar configuración"
                  : configSync?.configurado
                  ? "Cambiar destino"
                  : "Configurar destino"}
              </button>
              {configSync?.configurado && (
                <button
                  type="button"
                  onClick={handleSincronizarAhora}
                  disabled={sincronizando}
                  className="btn btn-primary"
                >
                  {sincronizando ? "Sincronizando..." : "Sincronizar ahora"}
                </button>
              )}
            </div>
          </div>

          {informeSync && (
            <div style={{ ...cardStyle, backgroundColor: "#F8FAFC" }}>
              <strong style={{ fontSize: "13.5px" }}>Resumen de la última sincronización:</strong>
              <div style={{ display: "flex", gap: "18px", marginTop: "8px", fontSize: "13px", flexWrap: "wrap" }}>
                <span>
                  Enviados: <strong>{informeSync.enviados}</strong>
                </span>
                <span>
                  Recibidos: <strong>{informeSync.recibidos}</strong>
                </span>
                <span>
                  Rechazados:{" "}
                  <strong style={{ color: informeSync.rechazados.length > 0 ? "#B91C1C" : "inherit" }}>
                    {informeSync.rechazados.length}
                  </strong>
                </span>
              </div>
              {informeSync.rechazados.length > 0 && (
                <ul style={{ margin: "8px 0 0 18px", color: "#991B1B", fontSize: "12.5px" }}>
                  {informeSync.rechazados.map((r, i) => (
                    <li key={`${r.id_evento || "sync"}-${i}`}>
                      {r.id_evento ? `${r.id_evento}: ` : ""}
                      {r.motivo}
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}

          {mostrarFormCredenciales && (
            <form onSubmit={handleActualizarCredenciales} style={{ ...cardStyle, border: "2px solid #2563EB", display: "flex", flexDirection: "column", gap: "12px" }}>
              <h3 style={{ fontSize: "15.5px", fontWeight: 800 }}>Actualizar credenciales de Git</h3>
              <p style={{ fontSize: "12.5px", color: "var(--text-secondary)" }}>
                Las credenciales se guardan únicamente en el almacén seguro del sistema operativo, nunca dentro del repositorio ni en el respaldo.
              </p>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
                <input
                  type="text"
                  value={credUsuario}
                  onChange={(e) => setCredUsuario(e.target.value)}
                  placeholder="Usuario de Git"
                  className="form-input"
                  required
                />
                <input
                  type="password"
                  value={credToken}
                  onChange={(e) => setCredToken(e.target.value)}
                  placeholder="Token de acceso personal (PAT)"
                  className="form-input"
                  required
                />
              </div>
              <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
                <button type="button" onClick={() => setMostrarFormCredenciales(false)} className="btn btn-secondary">
                  Cancelar
                </button>
                <button type="submit" disabled={actualizandoCreds} className="btn btn-blue">
                  {actualizandoCreds ? "Guardando..." : "Guardar credenciales"}
                </button>
              </div>
            </form>
          )}

          {(editandoDestino || !configSync?.configurado) && (
            <form onSubmit={handleGuardarDestino} style={{ ...cardStyle, display: "flex", flexDirection: "column", gap: "14px" }}>
              <h3 style={{ fontSize: "16px", fontWeight: 800 }}>Configurar destino de sincronización</h3>
              <div className="tab-grid" style={{ margin: 0, maxWidth: "420px" }}>
                <button
                  type="button"
                  onClick={() => setTipoDestino("git")}
                  className={`tab-btn ${tipoDestino === "git" ? "active-income" : ""}`}
                >
                  Repositorio Git remoto
                </button>
                <button
                  type="button"
                  onClick={() => setTipoDestino("carpeta")}
                  className={`tab-btn ${tipoDestino === "carpeta" ? "active-income" : ""}`}
                >
                  Carpeta compartida
                </button>
              </div>

              <div className="form-group">
                <label className="form-label">
                  {tipoDestino === "git" ? "URL del repositorio Git privado" : "Ruta de la carpeta compartida"}
                </label>
                <input
                  type="text"
                  value={rutaOUrl}
                  onChange={(e) => setRutaOUrl(e.target.value)}
                  placeholder={
                    tipoDestino === "git"
                      ? "https://github.com/tu-usuario/cuentas-claras-respaldo.git"
                      : "C:\\Users\\TuUsuario\\Syncthing\\CuentasClaras"
                  }
                  className="form-input"
                  required
                />
              </div>

              {tipoDestino === "git" && (
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
                  <div className="form-group">
                    <label className="form-label">Usuario de Git</label>
                    <input
                      type="text"
                      value={usuarioGit}
                      onChange={(e) => setUsuarioGit(e.target.value)}
                      placeholder="tu-usuario"
                      className="form-input"
                    />
                  </div>
                  <div className="form-group">
                    <label className="form-label">Token o contraseña de Git</label>
                    <input
                      type="password"
                      value={tokenGit}
                      onChange={(e) => setTokenGit(e.target.value)}
                      placeholder="ghp_..."
                      className="form-input"
                    />
                  </div>
                </div>
              )}

              <div className="form-group">
                <label className="form-label">Frase de paso de cifrado (mínimo 12 caracteres)</label>
                <input
                  type="password"
                  value={fraseSync}
                  onChange={(e) => setFraseSync(e.target.value)}
                  placeholder="Misma frase en todos tus dispositivos"
                  className="form-input"
                  required
                />
                <span className="helper-text">
                  Todos los archivos enviados se cifran en tu dispositivo antes de escribirse en el destino. La frase se resguarda en el almacén seguro del sistema.
                </span>
              </div>

              <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
                {configSync?.configurado && (
                  <button type="button" onClick={() => setEditandoDestino(false)} className="btn btn-secondary">
                    Cancelar
                  </button>
                )}
                <button type="submit" disabled={guardandoDestino} className="btn btn-primary">
                  {guardandoDestino ? "Guardando..." : "Guardar destino"}
                </button>
              </div>
            </form>
          )}
        </div>
      )}
    </div>
  );
}
