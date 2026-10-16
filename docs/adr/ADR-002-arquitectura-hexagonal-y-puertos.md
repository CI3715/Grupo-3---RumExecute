# ADR-002: Arquitectura Hexagonal y Puertos

## Estado
Aceptado

## Fecha
2026-10-16

## Contexto
El cliente de escritorio de **Cuentas Claras** debe gestionar reglas de negocio financieras complejas manteniendo un desacoplamiento total entre el dominio y los detalles técnicos de infraestructura (interfaz gráfica en Next.js, comandos IPC de Tauri y motores de persistencia).

Para garantizar la mantenibilidad, escalabilidad y facilitación de pruebas automatizadas, se requiere definir una arquitectura estructurada que impida la contaminación del núcleo de la aplicación con librerías o frameworks externos.

## Decisiones Adoptadas

### 1. Adopción del Patrón de Arquitectura Hexagonal (Puertos y Adaptadores)
* **Razón de Elección:**
  * **Aislamiento del Dominio:** La lógica del motor financiero, el cálculo de balances y los modelos de negocio residen en el núcleo interno en Rust, sin dependencias de infraestructura ni de Tauri.
  * **Definición de Puertos (Traits):** La comunicación desde el núcleo hacia el exterior se realiza mediante interfaces abstractas (*traits* de Rust). El dominio únicamente define qué necesita (ej. un puerto de almacenamiento `EventRepository`), sin importar cómo se implemente.
  * **Adaptadores Intercambiables:** Permite implementar adaptadores de infraestructura para la producción (SQLite) y adaptadores simulados (*In-Memory Mocks*) para pruebas unitarias instantáneas sin I/O en disco.
  * **Comandos IPC como Adaptadores de Entrada:** Los comandos Tauri actúan como adaptadores primarios que reciben invocaciones de la interfaz en Next.js y delegan la ejecución a los casos de uso del dominio.

## Consecuencias

### Positivas
* Total independencia de la interfaz de usuario y del motor de persistencia.
* Capacidad de probar el 100% de la lógica de negocio mediante `cargo test` usando repositorios en memoria sin depender de la base de datos ni de Tauri.
* Alta mantenibilidad y facilidad para refactorizar módulos internos sin afectar el resto del sistema.

### Negativas
* Requiere definir *traits*, estructuras de mapeo y múltiples capas conceptuales. Considerando la experiencia en Rust de algunos miembros del equipo, genera una alta complejidad inicial.
