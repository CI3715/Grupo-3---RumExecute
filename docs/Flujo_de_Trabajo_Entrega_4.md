# Plan y Flujo de Trabajo — Entrega 4: Manejo de Monedas y Tasas de Cambio (The Exchange Flow)

**Asignatura:** CI-3715: Ingeniería de Software (Universidad Simón Bolívar)  
**Equipo:** Grupo 3 — RumExecute  
**Proyecto:** *Cuentas Claras*  
**Porcentaje de Evaluación:** 10% del Laboratorio  
**Stack Tecnológico:** Tauri 2 (Rust + SQLite embebido `rusqlite` + `rust_decimal`) + Next.js 15 (React 19 / TypeScript / Tailwind CSS)  
**Alcance Normativo:** `RF-CAM-01`, `RF-CAM-02`, `RF-CAM-03`, `RF-CAM-04`, `RF-INF-01`, `RF-CAM-06` (básica)  
**Invariantes de Arquitectura:** `INV-01`, `INV-02`, `INV-03`, `INV-13`, `INV-16`, `INV-18`  
**Restricciones y RNF:** `RNF-08` (Aritmética decimal exacta), `RNF-12` (Event sourcing inmutable), `RNF-13` / `RNF-14` (Arquitectura hexagonal), `RNF-18` (Mensajes de error en español)

---

## 1. Objetivo General y Reglas Estrictas de la Cátedra

Partiendo del sistema funcional de cuentas y movimientos locales construido en las entregas anteriores, el objetivo de la **Entrega 4 (The Exchange Flow)** es incorporar el **manejo formal de múltiples monedas**, el **registro manual de tasas de cambio históricas**, la **conversión temporal y bidireccional (inversa)** y la **pantalla de Patrimonio consolidado** (`RF-INF-01`) expresado en una **Moneda Base elegida por el usuario** (`RF-CAM-02`), bajo un estricto modelo de **Event Sourcing inmutable en SQLite**.

### 1.1. Reglas de Oro e Invariantes No Negociables

* **Regla 1: Aritmética Decimal Exacta y Par Indivisible (`INV-01`, `INV-03`, `RNF-08`):**  
  Prohibido usar `f32`, `f64` o `Number` de JavaScript para cálculos o tasas de cambio. Todo cálculo interno y conversión se realiza con `rust_decimal::Decimal`. Todo saldo y valor de cambio contiene obligatoriamente `{ cantidad: Decimal, moneda: CodigoMoneda }`. Al consolidar patrimonio, la suma convertida se redondea al número de decimales exacto definido para la moneda base (ej. 2 decimales para USD/EUR/VES), nunca antes de la suma final.

* **Regla 2: Prohibición Estricta de Sumar Monedas Distintas sin Tasa (`INV-02`):**  
  Jamás se deben sumar cantidades con monedas diferentes de forma directa. En la vista de Patrimonio se deben mostrar siempre los subtotales separados por cada moneda original, y únicamente se calcula el total consolidado si existen tasas válidas hacia la moneda base.

* **Regla 3: Catálogo de Fábrica y Monedas Propias (`RF-CAM-01`):**  
  Las 4 monedas de fábrica son inmutables y no se pueden modificar ni eliminar:

  | Código | Nombre | Símbolo | Decimales |
  | :---: | :--- | :---: | :---: |
  | `VES` | Bolívar | `Bs.` | `2` |
  | `USD` | Dólar estadounidense | `$` | `2` |
  | `EUR` | Euro | `€` | `2` |
  | `USDT` | Tether (dólar digital) | `USDT` | `2` |

  El usuario puede registrar monedas propias adicionales con un código alfabético único de 3 a 5 letras mayúsculas ASCII (`^[A-Z]{3,5}$`), nombre descriptivo (1-50 caracteres), símbolo (1-5 caracteres UTF-8) y decimales permitidos (entero entre 0 y 8 inclusive).

* **Regla 4: Moneda Base del Sistema (`RF-CAM-02`):**  
  Por defecto es `USD`. El usuario puede cambiarla en cualquier momento por cualquier moneda del catálogo. Cambiar la moneda base **NO muta ni recalcula los movimientos ni los saldos originales** de ninguna cuenta. Solo afecta la visualización de la consolidación.

* **Regla 5: Registro de Tasas de Cambio y Conversión Bidireccional (`RF-CAM-03`):**  
  Par de monedas origen y destino distintas (`origen != destino`), valor estrictamente mayor a 0 con hasta 8 decimales, fecha de vigencia `YYYY-MM-DD` no futura y fuente textual.  
  - **Cálculo inverso:** Si existe tasa registrada $A \to B$, la tasa $B \to A$ se calcula matemáticamente como $1 / \text{valor}$.  
  - **Prohibición de conversión triangular:** Si se tiene $A \to B$ y $B \to C$, está terminantemente prohibido inferir o inventar $A \to C$. Toda conversión requiere una tasa directa o inversa explícita.  
  - **Selección de tasa vigente:** A una fecha de consulta $T$, la tasa vigente para el par es la tasa más reciente registrada cuya $\text{fecha\_vigencia} \le T$.

* **Regla 6: Detección Estricta de Tasa Faltante (`RF-CAM-04`):**  
  Si al consultar el patrimonio consolidado a una fecha dada, alguna cuenta con saldo no nulo está en una moneda sin tasa vigente (directa ni inversa) hacia la moneda base, el sistema **nunca debe inventar una tasa, ni asumir 1:1, ni omitir la cuenta**. La consulta retorna las cuentas y subtotales en su moneda original, pero el Total Consolidado debe ser `null`, y se genera una alerta inequívoca indicando explícitamente el par faltante y la fecha requerida.

* **Regla 7: Historial y Eliminación Inmutable de Tasas (`RF-CAM-06` básica, `INV-13`):**  
  Las tasas registradas se consultan por par con su historial. Se permite corregir un error eliminando una tasa mediante un evento inmutable `TasaEliminada` (sin ejecutar jamás un `DELETE` SQL físico).

* **Regla 8: Política Obligatoria de Pruebas Centralizadas:**  
  Los módulos de desarrollo (**Dominio**, **Aplicación**, **Infraestructura** e **Interfaz**) **NO escriben tests unitarios ni de integración**. Toda la suite de pruebas automatizadas (`cargo test`) se centraliza exclusivamente en la **Etapa 5 (Documentación y QA)**.

---

## 2. Flujo de Trabajo del Equipo en Git y Ciclo de Integración

Siguiendo las buenas prácticas de la cátedra para evitar conflictos de merge:

### 2.1. Protección de la Rama Principal (`main`)
* Ningún miembro realiza `push` directo sobre `main`.
* Toda integración se realiza mediante Pull Request (PR) revisado y aprobado.

### 2.2. Ramas de Capa / Funcionalidad
Cada etapa técnica se desarrollará en su propia rama derivada de `main`:
* `feature/dominio-monedas-tasas` (Etapa 1)
* `feature/aplicacion-casos-de-uso` (Etapa 2)
* `feature/infraestructura-sqlite-tauri` (Etapa 3)
* `feature/ui-patrimonio-tasas` (Etapa 4)
* `feature/docs-tests-entrega4` (Etapa 5)

### 2.3. Secuencia Obligatoria de Integración de Pull Requests

| Orden | Rama de Trabajo | Contenido Principal | Verificación Pre-PR | ¿Lleva Tests? |
| :---: | :--- | :--- | :--- | :---: |
| **PR 1** | `feature/dominio-...` | Entidades `Moneda`, `TasaCambio`, inversa, eventos | `cargo check` | **NO** |
| **PR 2** | `feature/aplicacion-...` | Casos de uso, DTOs, `consultar_patrimonio`, `RF-CAM-04` | `cargo check` + `cargo clippy` | **NO** |
| **PR 3** | `feature/infraestructura-...` | Persistencia SQLite, triggers anti-mutación, IPC | `cargo check` | **NO** |
| **PR 4** | `feature/ui-...` | Pantalla Patrimonio, alertas, modales de tasas | `npx tsc --noEmit` + `pnpm build` | **NO** |
| **PR 5** | `feature/docs-tests-...` | **Suite completa `cargo test`** + ADR-005 + Docs | `cargo test` (100% pasando) | **SÍ (Centralizado)** |

### 2.4. Protocolo Previo a Abrir o Mergear un PR
Antes de solicitar revisión, el autor de la rama debe sincronizarse con `main` localmente:
```bash
git fetch origin
git merge origin/main
```
Y verificar que el código compile limpiamente sin advertencias ni errores.

---

## 3. Contrato Interno de Datos y Frontera Arquitectónica

Para que las distintas etapas puedan avanzar en paralelo y desacopladas, se fijan los contratos exactos de la arquitectura.

### 3.1. Estructura de Capas en Rust (`front_end/src-tauri/src/`)

```text
front_end/src-tauri/src/
├── lib.rs                          # Registro de comandos Tauri y estado global
├── comandos.rs                     # Handlers IPC ( #[tauri::command] )
├── dominio/                        # CAPA DE DOMINIO PURA (Sin dependencias externas)
│   ├── mod.rs
│   ├── errores.rs                  # ErrorDominio { campo, mensaje }
│   ├── moneda.rs                   # MonedaPropia, CodigoMoneda, catálogo extensible (RF-CAM-01)
│   ├── tasa_cambio.rs              # TasaCambio, ParMonedas, inversa 1/x, selección temporal (RF-CAM-03)
│   ├── monto.rs                    # Monto { cantidad: Decimal, moneda: CodigoMoneda } (INV-01, INV-03)
│   └── evento.rs                   # MonedaCreada, MonedaBaseCambiada, TasaRegistrada, TasaEliminada (INV-13)
├── aplicacion/                     # CAPA DE APLICACIÓN (Casos de uso y orquestación)
│   ├── mod.rs
│   ├── puertos.rs                  # Traits RepositorioLibro y Reloj
│   ├── dto.rs                      # DTOs de entrada y salida JSON
│   └── casos_de_uso/
│       ├── registrar_tasa.rs       # RF-CAM-03
│       ├── eliminar_tasa.rs        # RF-CAM-06
│       ├── cambiar_moneda_base.rs  # RF-CAM-02
│       ├── crear_moneda.rs         # RF-CAM-01
│       ├── consultar_patrimonio.rs # RF-INF-01 y detección estricta RF-CAM-04
│       └── listar_historial_tasas.rs
└── infraestructura/                # CAPA DE INFRAESTRUCTURA (SQLite y adaptadores)
    ├── mod.rs
    ├── persistencia/sqlite.rs      # Serialización de eventos en eventos_libro y triggers
    └── sistema.rs                  # Reloj y generador de IDs
```

### 3.2. Contratos de los Comandos Tauri IPC (`invoke`)

#### 1) `invoke('consultar_patrimonio', { fechaConsulta: "2026-10-25" })`
* **Entrada:** `fechaConsulta` (opcional, string ISO `YYYY-MM-DD`. Por defecto fecha actual).
* **Salida Exitosa (`PatrimonioDto`):**
  ```json
  {
    "fecha_consulta": "2026-10-25",
    "moneda_base": "USD",
    "cuentas": [
      {
        "id": "uuid-cta-1",
        "nombre": "Banesco",
        "tipo": "banco",
        "saldo_original": "1000.00",
        "moneda_original": "VES",
        "tasa_aplicada": "0.025",
        "saldo_convertido": "25.00"
      },
      {
        "id": "uuid-cta-2",
        "nombre": "Efectivo",
        "tipo": "efectivo",
        "saldo_original": "50.00",
        "moneda_original": "USD",
        "tasa_aplicada": "1.0",
        "saldo_convertido": "50.00"
      }
    ],
    "subtotales": [
      { "moneda": "VES", "total": "1000.00" },
      { "moneda": "USD", "total": "50.00" }
    ],
    "total_consolidado": "75.00",
    "tasa_faltante": null
  }
  ```
* **Salida con Tasa Faltante (`RF-CAM-04`):**
  ```json
  {
    "fecha_consulta": "2026-10-25",
    "moneda_base": "USD",
    "cuentas": [ ... ],
    "subtotales": [ ... ],
    "total_consolidado": null,
    "tasa_faltante": {
      "moneda_origen": "EUR",
      "moneda_destino": "USD",
      "fecha_requerida": "2026-10-25",
      "mensaje": "No se puede calcular el patrimonio consolidado al 2026-10-25 porque falta la tasa de cambio para el par EUR -> USD."
    }
  }
  ```

#### 2) `invoke('registrar_tasa', { entrada })`
* **Entrada (`RegistrarTasaDto`):**
  ```json
  {
    "origen": "VES",
    "destino": "USD",
    "valor": "0.025",
    "fecha_vigencia": "2026-10-20",
    "fuente": "BCV Oficial"
  }
  ```
* **Salida exitosa:** `{ "exito": true, "id_tasa": "uuid-tasa-1" }`
* **Salida de error (`ErrorValidacionDto`):**
  ```json
  {
    "campo": "fecha_vigencia",
    "mensaje": "La fecha de vigencia no puede ser futura."
  }
  ```

#### 3) `invoke('eliminar_tasa', { idTasa: "uuid-tasa-1" })`
* **Entrada:** `idTasa: string`
* **Salida:** `{ "exito": true }`

#### 4) `invoke('cambiar_moneda_base', { nuevoCodigo: "EUR" })`
* **Entrada:** `nuevoCodigo: string`
* **Salida:** `{ "exito": true, "moneda_base": "EUR" }`

#### 5) `invoke('crear_moneda', { entrada })`
* **Entrada (`CrearMonedaDto`):**
  ```json
  {
    "codigo": "CAD",
    "nombre": "Dólar Canadiense",
    "simbolo": "C$",
    "decimales": 2
  }
  ```
* **Salida:** `{ "exito": true, "codigo": "CAD" }`

---

## 4. Detalle de las 5 Etapas del Flujo de Trabajo

### 🔹 Etapa 1: Capa de Dominio Financiero, Monedas Propias y Tasas (`dominio`)
* **Rama:** `feature/dominio-monedas-tasas`
* **Directorio:** `front_end/src-tauri/src/dominio/`
* **Entradas:** Especificación de requerimientos de Entrega 4.
* **Actividades paso a paso:**
  1. Extender `dominio/moneda.rs`:
     - Validación de código alfabético en mayúsculas de 3 a 5 letras (`^[A-Z]{3,5}$`).
     - Validación de nombre descriptivo, símbolo y decimales entre 0 y 8 (`RF-CAM-01`).
     - Protección inmutable del catálogo de fábrica (`VES`, `USD`, `EUR`, `USDT`).
  2. Implementar `dominio/tasa_cambio.rs`:
     - Value Object `ParMonedas(origen, destino)` con `origen != destino`.
     - Entidad `TasaCambio` con valor decimal exacto > 0 y fecha de vigencia no futura.
     - Lógica de conversión directa e inversa ($1 / \text{valor}$) (`RF-CAM-03`).
     - Algoritmo de tasa vigente: dada una fecha $T$, seleccionar la tasa más reciente registrada con $\text{fecha\_vigencia} \le T$.
     - Prohibición estricta de triangulación o conversión cruzada.
  3. Definir en `dominio/evento.rs` los nuevos eventos inmutables:
     - `MonedaCreada`, `MonedaBaseCambiada`, `TasaRegistrada`, `TasaEliminada`.
* **Criterio de Aceptación (DoD):** `cargo check` compila exitosamente. Cero dependencias de SQLite ni Tauri.
* **Regla de pruebas:** **Cero tests automatizados asignados a esta etapa.**

---

### 🔹 Etapa 2: Capa de Aplicación, Casos de Uso y Orquestación (`aplicacion`)
* **Rama:** `feature/aplicacion-casos-de-uso`
* **Directorio:** `front_end/src-tauri/src/aplicacion/`
* **Entradas:** Entidades y eventos terminados de la Etapa 1.
* **Actividades paso a paso:**
  1. Definir los DTOs en `aplicacion/dto.rs` correspondientes a las firmas de la Sección 3.2.
  2. Implementar `registrar_tasa.rs`, `eliminar_tasa.rs`, `cambiar_moneda_base.rs` y `crear_moneda.rs`.
  3. Implementar el caso de uso central `consultar_patrimonio.rs`:
     - Obtener las cuentas activas y proyectar sus saldos a la `fecha_consulta`.
     - Agrupar los subtotales por moneda original (`INV-02`).
     - Resolver si existe tasa vigente para cada moneda presente hacia la moneda base.
     - **Aplicación estricta de `RF-CAM-04`:** Si existe una cuenta con saldo en una moneda sin tasa vigente:
       - Retornar las cuentas y subtotales en su moneda original.
       - Asignar `total_consolidado = None`.
       - Llenar el objeto `tasa_faltante` con el par faltante exacto y la fecha requerida.
     - Si todas las tasas requeridas existen:
       - Convertir cada saldo a la moneda base.
       - Sumar con precisión `rust_decimal` y redondear al número de decimales de la moneda base.
       - Retornar el DTO con el total consolidado exacto.
* **Criterio de Aceptación (DoD):** `cargo check` y `cargo clippy` sin advertencias.
* **Regla de pruebas:** **Cero tests automatizados asignados a esta etapa.**

---

### 🔹 Etapa 3: Persistencia Local SQLite y Comandos Tauri IPC (`infraestructura` + `comandos`)
* **Rama:** `feature/infraestructura-sqlite-tauri`
* **Directorio:** `front_end/src-tauri/src/infraestructura/` y `front_end/src-tauri/src/comandos.rs`
* **Entradas:** Casos de uso de la Etapa 2.
* **Actividades paso a paso:**
  1. En `infraestructura/persistencia/sqlite.rs`:
     - Adaptar `eventos_libro` para almacenar los nuevos eventos `MonedaCreada`, `MonedaBaseCambiada`, `TasaRegistrada`, `TasaEliminada` como JSON inmutable (`INV-13`).
     - Asegurar triggers SQLite `BEFORE UPDATE` y `BEFORE DELETE` que rechacen mutaciones directas.
     - Replay de eventos para reconstruir el historial de tasas y el catálogo de monedas.
  2. En `comandos.rs` y `lib.rs`:
     - Implementar los handlers `#[tauri::command]` para los métodos definidos en la Sección 3.2.
     - Garantizar que los errores devuelvan mensajes descriptivos en español sin trazas de pánico (`RNF-18`).
* **Criterio de Aceptación (DoD):** `cargo check` compila y el ejecutable Tauri inicializa el backend.
* **Regla de pruebas:** **Cero tests automatizados asignados a esta etapa.**

---

### 🔹 Etapa 4: Interfaz de Usuario en Next.js — Pantalla de Patrimonio y Tasas (`front_end/src/`)
* **Rama:** `feature/ui-patrimonio-tasas`
* **Directorio:** `front_end/src/app/` y `front_end/src/components/`
* **Entradas:** Comandos Tauri disponibles de la Etapa 3.
* **Actividades paso a paso:**
  1. Construir la vista de **Patrimonio** (`RF-INF-01`):
     - Selector interactivo de **Fecha de Consulta** (con opción de consultar fecha actual o fechas pasadas).
     - Visualizador y selector de cambio de **Moneda Base** (`RF-CAM-02`).
     - Tabla con las cuentas: Nombre, Tipo, Saldo Original, Tasa aplicada a la fecha y Saldo en Moneda Base.
     - Tarjetas de **Subtotales por Moneda Original** (`INV-02`).
     - Tarjeta destacada de **Total Consolidado en Moneda Base**.
  2. Implementar la experiencia de **Tasa Faltante (`RF-CAM-04`)**:
     - Si `total_consolidado === null`, ocultar la tarjeta de total y mostrar un **Banner de Alerta inequívoco**:  
       *"No se puede calcular el patrimonio consolidado al DD/MM/AAAA porque falta la tasa de cambio para [ORIGEN] -> [DESTINO]."*
     - Botón de acción rápida: *"Registrar Tasa Ahora"*, abriendo el modal con los datos preseleccionados.
  3. Modales de **Registro de Tasas** e **Historial de Tasas** por par con botón de eliminación (`RF-CAM-06`).
  4. Modal de **Creación de Moneda Propia** (`RF-CAM-01`) protegiendo las 4 monedas de fábrica.
* **Criterio de Aceptación (DoD):** `npx tsc --noEmit` sin errores y `pnpm build` genera el bundle sin fallos.
* **Regla de pruebas:** **Cero tests automatizados asignados a esta etapa.**

---

### 🔹 Etapa 5: Suite Completa de Pruebas Automatizadas Centralizadas y Documentación (`docs/` + `tests/`)
* **Rama:** `feature/docs-tests-entrega4`
* **Directorio:** `front_end/src-tauri/tests/`, `docs/` y `front_end/README.md`
* **Entradas:** Código integrado de las Etapas 1 a 4.
* **Actividades paso a paso:**
  1. **Suite Integral de Pruebas Automatizadas en Rust (`cargo test`):**
     *Esta etapa asume el 100% de la autoría de las pruebas del sistema:*
     - **Pruebas de Dominio:** Validación de códigos de moneda, 0-8 decimales, catálogo de fábrica inmutable, rechazo de tasas $\le 0$, monedas iguales, o fechas futuras.
     - **Pruebas de Conversión e Invariantes:** Precisión decimal de tasa inversa ($1 / \text{valor}$), prohibición estricta de conversiones triangulares y rechazo de sumas con monedas distintas (`INV-02`).
     - **Pruebas de Aplicación y Patrimonio:** Selección temporal de tasas por fecha $\le T$, comportamiento ante tasas eliminadas, cambio de moneda base sin mutación de cuentas.
     - **Prueba Automatizada de `RF-CAM-04`:** Detección de tasa faltante reportando el par y fecha exacta, con total consolidado en `None`.
     - **Automatización del Escenario Oficial de la Sección 2.3 de la Cátedra:**
       - Simular cuentas con saldos en `VES` y `USD`.
       - Registrar tasa $t_1$ y verificar conversión correcta a fecha $t_1$.
       - Registrar tasa actualizada en $t_2$ y verificar selección temporal correcta a fecha $t_2$.
       - Simular cuenta en `EUR` sin tasa y verificar emisión de alerta de tasa faltante.
     - **Pruebas de Inmutabilidad SQLite (`INV-13`):** Comprobar que los triggers bloquean `UPDATE` y `DELETE` en `eventos_libro`.
  2. **Documentación Formal:**
     - `Documento_Tecnico_Explicativo_Entrega_4.md`.
     - `ADR-005` (Manejo de Tasas de Cambio, Tasa Inversa y Detección de Tasa Faltante).
     - Actualización de `front_end/README.md` con la guía de reproducción en vivo.
* **Criterio de Aceptación (DoD):** `cargo test` ejecutando 100% verde (todos los tests pasando) y documentación lista para entrega.

---

## 5. Checklist Oficial de Criterios de Aceptación (Defensa en Vivo / Sección 2.3)

Durante la evaluación de la Entrega 4, se debe demostrar en vivo el siguiente flujo estricto:

- [ ] **1. Catálogo y Monedas Propias (`RF-CAM-01`):**
  - Mostrar que existen de fábrica `VES`, `USD`, `EUR` y `USDT` y no se pueden editar ni borrar.
  - Registrar una moneda propia (ej. `CAD`, 2 decimales) validando código de 3 a 5 mayúsculas.
- [ ] **2. Selección de Moneda Base (`RF-CAM-02`):**
  - Cambiar la moneda base de `USD` a `VES` o `EUR` y confirmar que los saldos originales de las cuentas no se alteran.
- [ ] **3. Demostración del Escenario Sección 2.3 de la Cátedra:**
  - Tener una cuenta en `VES` con 1.000 Bs. y una cuenta en `USD` con 50 $.
  - Consultar patrimonio a fecha $t_1$ sin tasa registrada $\rightarrow$ El sistema muestra las cuentas y subtotales, pero **bloquea el consolidado** indicando que falta la tasa `VES -> USD` al día $t_1$ (`RF-CAM-04`).
  - Registrar tasa `VES -> USD = 0.025` con fecha $t_1$.
  - Consultar a fecha $t_1$ $\rightarrow$ Se calcula el total consolidado exacto (ej. 75.00 USD).
  - Registrar nueva tasa `VES -> USD = 0.020` con fecha posterior $t_2$.
  - Consultar a $t_1$ $\rightarrow$ Aplica la tasa de 0.025.
  - Consultar a $t_2$ $\rightarrow$ Aplica la tasa más reciente de 0.020.
- [ ] **4. Demostración de Tasa Inversa (`RF-CAM-03`):**
  - Con solo la tasa `VES -> USD` registrada, cambiar la moneda base a `VES` y verificar que el sistema calcula la conversión inversa (`1 / valor`) sin exigir registrar `USD -> VES` manualmente.
- [ ] **5. Rechazo de Conversión Triangular:**
  - Tener `VES -> USD` y `EUR -> USD`, pero ninguna tasa `VES -> EUR`. Cambiar moneda base a `EUR` y verificar que **no inventa** la tasa cruzada y exige registrarla.
- [ ] **6. Rechazo de Validaciones:**
  - Intentar registrar tasa con fecha futura $\rightarrow$ Rechazo inmediato indicando fecha no permitida.
  - Intentar registrar tasa con valor $\le 0$ $\rightarrow$ Rechazo inmediato.
  - Intentar registrar tasa con monedas idénticas $\rightarrow$ Rechazo inmediato.
- [ ] **7. Historial y Eliminación Inmutable (`RF-CAM-06`):**
  - Consultar el historial de tasas de un par y eliminar una tasa. Confirmar que ya no aplica para cálculos futuros pero el evento persiste en la base de datos.
- [ ] **8. Ejecución de la Suite de Pruebas Automatizadas:**
  - Ejecutar `cargo test` mostrando la aprobación de todos los tests de dominio, aplicación, inmutabilidad y el escenario de la Sección 2.3.
