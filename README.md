# Simulador de tráfico en Rust

Base de un simulador de tráfico discreto, determinista y orientado a eventos. El objetivo inmediato no es modelar una ciudad completa, sino dejar lista la infraestructura para construir el motor: red vial, rutas, semáforos, colas, congestión, métricas y validación.

## Estado actual

- `src/model/`: dominio separado por responsabilidad.
- `src/model/mod.rs`: reexporta el modelo público.
- `src/model/node.rs`: nodos, fases y planes semafóricos.
- `src/model/road.rs`: tramos y capacidades.
- `src/model/vehicle.rs`: vehículos, tipos y estados.
- `src/model/network.rs`: red vial y consulta del grafo.
- `src/model/routing.rs`: cálculo de rutas.
- `src/simulation.rs`: motor por ticks, colas y control de avance.
- `src/simulation/events.rs`: eventos y reporte final.
- `src/scenario.rs`: red de demostración y calendario de vehículos.
- `src/main.rs`: demo en consola para ejecutar el flujo actual.
- `documentation/idea.md`: especificación funcional del motor.
- `documentation/plan.md`: plan de desarrollo alineado con esa especificación.

## Preparación del entorno

- Se usa Rust estable mediante `rust-toolchain.toml`.
- `Cargo.lock` se conserva para mantener la base reproducible.
- `target/` y archivos temporales quedan fuera del control de versiones con `.gitignore`.
- La lógica del motor se desarrollará sobre colecciones deterministas y pasos discretos.

## Requisitos

- Rust instalado con `rustup`.
- Herramientas `cargo`, `rustfmt` y `clippy` disponibles.

## Comandos útiles

```bash
cargo build
cargo test
cargo run
cargo fmt
cargo clippy
```

## Cómo empezar

1. Abrir el proyecto en la carpeta raíz del repositorio.
2. Verificar que la toolchain estable esté activa.
3. Ejecutar `cargo test` para comprobar que la base compila.
4. Usar `cargo run` para ver la demo actual en consola.

## Estructura del proyecto

- `src/lib.rs`: exporta la API pública del proyecto.
- `src/model/`: tipos base del dominio y la red vial.
- `src/simulation.rs`: ciclo de simulación y resolución de colas.
- `src/simulation/events.rs`: eventos y reporte de ejecución.
- `src/scenario.rs`: escenario de prueba listo para ejecutar.
- `src/main.rs`: punto de entrada del binario.
- `documentation/idea.md`: definición funcional detallada.
- `documentation/plan.md`: ruta de desarrollo previa al motor.

## Próximo paso

La siguiente etapa es convertir la base actual en un motor bien definido, con contrato de entrada, tick atómico, prioridades, congestión, replanificación y métricas. La lógica del mapa vendrá después como dato de entrada, no como el centro del diseño.
