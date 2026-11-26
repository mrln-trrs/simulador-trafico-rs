# Registro de cambios

Todos los cambios relevantes de este proyecto se documentan en este archivo.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [Versionado Semántico](https://semver.org/lang/es/).

## [0.1.0] - 2026-11-26

### Añadido
- Editor gráfico con egui/eframe: rejilla infinita, cámara con zoom, herramientas de vía, edificio, inspección, borrado y edición (base), detección de colisiones y ventana de configuración.
- Reloj de paso fijo y persistencia del estado de la ventana.
- Motor de simulación anterior (grafo vial, vehículos, rutas, semáforos, métricas y persistencia JSON) conservado en `legacy/`, fuera de la compilación.
- Documentación de ideas, planes y arquitectura en `documentation/`.
- README actualizado al estado real, licencia MIT, política de seguridad y configuración de Dependabot.

### Eliminado
- Dependencia `instant` (sin uso y sin mantenimiento, RUSTSEC-2024-0384).

### Cambiado
- Dependencias actualizadas con `cargo update` dentro de las versiones declaradas.

[0.1.0]: https://github.com/mrln-trrs/simulador-trafico-rs/releases/tag/v0.1.0
