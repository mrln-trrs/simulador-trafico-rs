# Simulador de Tráfico

Proyecto en Rust para construir un simulador de tráfico urbano.
En este momento el foco está en la base visual y en la organización de la interfaz: ya hay una ventana de escritorio, un plano cuadriculado simple y una UI básica para explorar el espacio.

## Estado actual

- Ventana nativa creada con `eframe` / `egui`.
- Plano base con grilla infinita y navegación por zoom y desplazamiento.
- Barra superior con acciones básicas de vista.
- Barra inferior con información del viewport y del plano.
- Persistencia simple del estado de la ventana.
- Estructura de módulos separada para que la UI pueda crecer sin mezclar responsabilidades.

Todavía no hay simulación de tráfico real. No existen aún calles, carriles, intersecciones, vehículos, semáforos, rutas ni colisiones.

## Qué ya está listo

- Base de aplicación de escritorio funcionando.
- Plano visual estable para empezar a construir la red vial.
- Organización interna de la UI en módulos pequeños.
- Soporte para seguir iterando sobre la ventana sin rehacer la estructura.

## Próximo progreso

1. Crear el modelo de red vial: nodos, tramos, carriles e intersecciones.
2. Definir reglas básicas de circulación y movimiento.
3. Añadir vehículos y su lógica de desplazamiento.
4. Introducir semáforos, prioridades y conflictos.
5. Conectar métricas, depuración y persistencia de escenarios.

## Estructura del proyecto

```text
src/
├── app/
├── generation/
├── integration/
├── model/
├── persistence/
├── simulation/
└── ui/
    └── screens/
        └── simulator/
            ├── bars/
            ├── canvas/
            ├── components/
            └── state/
```

## Tecnologías

- Rust 2021
- `eframe`
- `egui`
- `serde`
- `serde_json`
- `toml`

## Desarrollo

```bash
cargo run
```

```bash
cargo test
```

```bash
cargo fmt
```

```bash
cargo clippy
```

## Objetivo

La meta es pasar de esta base visual a un simulador de tráfico completo y mantenible, empezando por la red vial y el motor de simulación, y luego sumando edición, métricas y persistencia de escenarios.

## Licencia

MIT
