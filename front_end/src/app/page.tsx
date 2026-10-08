"use client";

import { useState } from "react";

interface Response {
  msg: string;
  code: number;
}

export default function HomePage() {
  const [data, setData] = useState<Response | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFetch = async () => {
    setLoading(true);
    setError(null);
    setData(null);

    try {
      const res = await fetch("http://localhost:8000");
      if (!res.ok) {
        throw new Error(`HTTP error! status: ${res.status}`);
      }
      const jsonData: Response = await res.json();
      setData(jsonData);
    } catch (err: any) {
      setError(err.message || "Error al conectar con el servidor");
    } finally {
      setLoading(false);
    }
  };

  return (
    <main style={{ padding: "2rem", fontFamily: "sans-serif" }}>
      <h1>¡Bienvenido a Cuentas Claras!</h1>
      <p>Presiona el botón de abajo para verificar la conexión del servidor.</p>

      <button
        onClick={handleFetch}
        disabled={loading}
        style={{
          padding: "0.5rem 1rem",
          cursor: loading ? "not-allowed" : "pointer",
        }}
      >
        {loading ? "Cargando..." : "Verificar conexión"}
      </button>

      {error && (
        <div style={{ color: "red", marginTop: "1rem" }}>
          <p>Error: {error}</p>
        </div>
      )}

      {data && (
        <div
          style={{
            marginTop: "1rem",
            padding: "1rem",
            backgroundColor: "#f0f0f0",
            borderRadius: "5px",
          }}
        >
          <h3>Respuesta:</h3>
          <pre>{JSON.stringify(data, null, 2)}</pre>
        </div>
      )}
    </main>
  );
}
