# Simulador de Tráfico

Proyecto en Rust para construir un simulador de tráfico urbano.

## Estado actual

El proyecto cuenta con un motor de simulación funcional y una interfaz gráfica interactiva, desarrollados de forma desacoplada:

- **Backend / Motor de simulación (`src/simulation`, `src/model`):**
  - Representación de la red vial mediante un grafo dirigido (nodos, tramos y carriles).
  - Simulación discreta por ticks determinista y reproducible con semillas.
  - Generación de vehículos con planificación de rutas dinámicas (Dijkstra) y evasión de congestión.
  - Semáforos lógicos estructurados por fases temporales.
  - Registro automático de eventos e historial por tick.
  - Recopilación de métricas de rendimiento del tráfico (vehículos completados, tiempo de viaje, tiempos de espera).
- **Persistencia (`src/persistence`):**
  - Serialización y deserialización de escenarios completos a formato JSON.
- **Frontend / Interfaz Gráfica (`src/ui`, `src/app`):**
  - Ventana nativa de escritorio creada con `eframe` / `egui`.
  - Lienzo 2D con rejilla infinita, desplazamiento fluido y zoom.
  - Herramienta de trazado magnético (snapping) de carreteras con ancho y número de carriles configurable.
  - Herramienta de dibujo poligonal para edificios y obstáculos con soporte para polígonos complejos (triangulación por orejas).
  - Detección de colisiones geométricas entre carreteras y edificios.
  - Herramienta de borrado selectivo (sub-polígonos, lasso de selección o elementos completos) e inspector base de objetos.

## Estructura del proyecto

El proyecto está diseñado bajo un monolito modular con separación estricta de responsabilidades:

```text
src/
├── app/          # Inicialización (bootstrap), runtime de egui y reloj lógico
├── generation/   # Generación procedural de escenarios (Builders, Fixtures)
├── integration/  # Contratos compartidos, snapshots, deltas de estado y eventos
├── model/        # Datos puros de dominio (grafo, vehículos, semáforos) sin I/O ni UI
├── persistence/  # Serialización y guardado de escenarios en archivos JSON
├── simulation/   # Motor de simulación determinista (routing, tick engine, métricas)
└── ui/           # Interfaz gráfica de usuario en egui
    └── screens/
        └── simulator/
            ├── bars/        # Barra superior de menú y barra inferior de estado
            ├── canvas/      # Rejilla infinita y manejo de viewport
            ├── components/  # Paneles laterales y menús de herramientas
            ├── geom/        # Lógica geométrica (triangulación, colisiones de carriles)
            ├── state/       # Persistencia de la ventana y estados de GUI
            └── tools/       # Herramientas de dibujo interactivo (Road, Building, Inspect, Delete)
```

## Próximo progreso

El siguiente hito del proyecto es la **integración interactiva del motor en la interfaz gráfica**:
1. Conectar `SimulationEngine` dentro del estado del visualizador (`SimuladorApp`).
2. Implementar los controles de reproducción visuales (Play, Pausa, Reset, Avance por tick) en la barra de menú o lateral.
3. Renderizar dinámicamente los vehículos e interpolar sus movimientos basándose en los snapshots y deltas emitidos por el motor.
4. Mostrar visualmente los estados de los semáforos lógicos sobre el lienzo.
5. Permitir la inspección en tiempo real de vehículos y semáforos, mostrando sus estadísticas de viaje y tiempos de espera.

## Tecnologías

- Rust 2021
- `eframe`
- `egui`
- `serde`
- `serde_json`
- `toml`

## Desarrollo

Para ejecutar el simulador gráfico interactivo:
```bash
cargo run
```

Para correr las pruebas unitarias y de integración del motor de simulación:
```bash
cargo test
```

Para dar formato y validar la calidad del código:
```bash
cargo fmt
cargo clippy
```

## Licencia

MIT
