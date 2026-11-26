# Simulador de Tráfico

Simulador de tráfico urbano en Rust: editor gráfico de redes viales sobre egui y un motor de simulación por ticks en desarrollo.

![Estado](https://img.shields.io/badge/estado-experimental-orange)
![Licencia](https://img.shields.io/badge/licencia-MIT-blue)
![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)

## Estado del proyecto

**Experimental** (v0.1.0). Lo que compila y se ejecuta hoy es el **editor gráfico** (`src/`). El **motor de simulación** (grafo vial, vehículos, rutas, semáforos, persistencia en JSON) existe como código anterior en `legacy/`, que **no forma parte de la compilación actual**: está pendiente de reintegrarse en el editor.

## Características

### Editor gráfico (`src/`, compilado)

- Ventana de escritorio con `eframe`/`egui` y ventanas adicionales mediante `egui-multiwin` (configuración).
- Lienzo 2D con rejilla infinita rotulada, desplazamiento y zoom.
- Herramientas:
  - **Vía**: trazado con ajuste magnético, ancho y número de carriles configurables.
  - **Edificio**: dibujo de polígonos (incluidos cóncavos, con triangulación por orejas).
  - **Inspeccionar**: selección y datos básicos del objeto.
  - **Borrar**: borrado de elementos completos o de partes, con vista previa.
  - **Editar**: estructura base de la herramienta de edición.
- Detección de colisiones geométricas entre vías y edificios.
- Reloj de paso fijo (ticks por segundo) preparado para conducir el motor.
- Ajustes de interfaz (zoom de la UI y tamaño del texto) y persistencia del estado de la ventana.

### Motor de simulación (`legacy/`, no compilado)

- Red vial como grafo dirigido (nodos, tramos y carriles) con invariantes.
- Simulación discreta por ticks, reproducible mediante semillas.
- Vehículos con rutas más cortas (Dijkstra con montículo) y recálculo de ruta.
- Métricas de tráfico y detección de conflictos.
- Semáforos por fases, eventos, *snapshots* y deltas de estado.
- Generación de escenarios y guardado en JSON.

## Stack

| Componente | Versión |
|---|---|
| Rust | edición 2021 (probado con rustc 1.100) |
| egui / eframe | 0.27 (con `persistence`) |
| egui-multiwin | 0.5.2 |
| clipper2-rust | 1.0 |
| serde, serde_json, toml, chrono | 1.x / 1.x / 0.8 / 0.4 |

## Requisitos

- Rust estable con `cargo` (<https://rustup.rs>).
- Windows, macOS o Linux con soporte de OpenGL.

## Instalación y uso

```bash
git clone https://github.com/mrln-trrs/simulador-trafico-rs.git
cd simulador-trafico-rs

cargo run --release   # abre el editor
cargo test --lib      # pruebas unitarias de src/ (ver limitaciones)
cargo clippy          # análisis estático
```

## Estructura del proyecto

```
src/
├── main.rs, lib.rs         # punto de entrada
├── app/                    # arranque, runtime de ventanas y reloj de paso fijo
└── ui/screens/
    ├── menu/               # ajustes de interfaz
    └── simulator/
        ├── bars/           # barra de menú y barra de estado
        ├── canvas/         # rejilla infinita, cámara y caché de dibujo
        ├── components/     # panel lateral de herramientas
        ├── geom/           # ángulos, distancias, colisiones y triangulación
        ├── state/          # estado persistente de la ventana
        ├── tools/          # vía, edificio, inspección, borrado y edición
        └── windows/        # ventana de configuración
legacy/                     # motor de simulación anterior (no compilado)
├── model/  simulation/  integration/  generation/  persistence/  tests/
documentation/              # ideas, planes y arquitectura (incluye un PDF consolidado)
assets/lucide.ttf           # fuente de iconos
```

## Limitaciones conocidas

- El motor de simulación (`legacy/`) no está conectado al editor: todavía no se ven vehículos ni semáforos en el lienzo. Es el siguiente hito: integrar el motor, controles de reproducción (iniciar, pausar, reiniciar, avanzar por tick) y el dibujo de vehículos y semáforos.
- **egui 0.27 fijado**: `egui-multiwin` 0.5.2 (sin mantenimiento desde mayo de 2024) exige egui 0.27 y winit 0.29. Para subir a egui 0.37 hay que sustituirlo por los *viewports* nativos de eframe.
- `cargo test` sin filtros falla en un *doctest* que genera la macro `multi_window!` de `egui-multiwin` (busca una fuente que no existe); las pruebas propias se ejecutan con `cargo test --lib`.
- `toml` 0.8 (la actual es 1.x) pendiente de actualizar; `serde_json` y `toml` solo los usa el código de `legacy/`.
- Sin integración continua.

## Autor

Marlon Torres Espinoza ([@mrln-trrs](https://github.com/mrln-trrs)), con aportes de javi-alfaro.

## Licencia

Distribuido bajo la licencia MIT. Ver [LICENSE](LICENSE).
