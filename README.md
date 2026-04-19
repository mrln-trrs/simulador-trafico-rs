# Simulador de tráfico en Rust

Este proyecto modela una red de tráfico como un grafo dirigido:

- Los nodos representan intersecciones, accesos, salidas y puntos de control semafórico.
- Los tramos de vía representan las conexiones entre nodos.
- Los vehículos recorren una ruta, esperan en colas y respetan las fases del semáforo en cada nodo.

La implementación actual es pequeña a propósito, pero ya sirve como base para seguir creciendo el simulador.

## Ejecución

```bash
cargo run
```

## Pruebas

```bash
cargo test
```

## Estructura

- `src/model.rs`: modelo de datos principal, grafo de la red y cálculo de rutas más cortas.
- `src/simulation.rs`: motor de simulación por ticks, colas y control semafórico.
- `src/scenario.rs`: red de ejemplo y programación de vehículos.
- `src/main.rs`: demostración en consola.

## Notas

- El simulador usa pasos de tiempo discretos.
- Los tramos se modelan con capacidad por carril y colas.
- El planificador de rutas elige actualmente el camino más rápido según el tiempo de viaje.
- Esto deja una buena base para extenderlo después con desvíos dinámicos, accidentes o comportamiento de carril más detallado.
