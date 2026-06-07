# Plan de integración del simulador

Este documento define la capa que conecta el motor y el visualizador. Su foco está en los contratos, el intercambio de datos y la estabilidad entre versiones.

## 1. Objetivo

Establecer una integración clara y estable entre backend y frontend para que ambos puedan evolucionar sin romper el sistema completo.

## 2. Principios

- Las estructuras compartidas deben ser mínimas y explícitas.
- Los mensajes deben ser fáciles de serializar y validar.
- El flujo de comandos y eventos debe ser predecible.
- La compatibilidad entre versiones debe gestionarse desde el diseño.
- La persistencia debe poder reconstruir escenarios completos.
- Las instantáneas deben priorizar actualizaciones delta; los snapshots completos quedan para arranque, reinicio o resincronización.
- Para corridas grandes, la persistencia y la traza deben poder usar formatos compactos o binarios además de formatos legibles.

## 3. Alcance funcional

La integración debe cubrir:

- identificadores compartidos,
- snapshots de estado,
- comandos de edición y control,
- eventos de simulación,
- formatos de persistencia,
- versionado de contratos,
- validación de compatibilidad.

## 4. Fases de implementación

### Fase 0: modelo compartido [COMPLETADA]

- fijar tipos base comunes (en `src/model`),
- definir identificadores estables (`VehicleId`, `NodeId`, `SegmentId`, `SignalId`),
- separar dominio, vista y transporte (estructura limpia e independiente).

### Fase 1: snapshots y estado [COMPLETADA]

- definir la estructura de instantáneas completas y delta (implementado en [src/integration/snapshots.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/integration/snapshots.rs) y [src/integration/delta.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/integration/delta.rs)),
- acordar qué entidades y campos cambian por tick (progreso de vehículos, fases de semáforos, cola),
- permitir actualizaciones parciales y asegurar lecturas consistentes por tick.

### Fase 2: comandos y eventos [PARCIALMENTE COMPLETADA]

- definir comandos de edición (`src/integration/commands.rs`),
- definir eventos emitidos por el motor (`src/integration/events.rs` - implementado),
- [PENDIENTE] establecer el envío de comandos desde el visualizador interactivo y la reacción a los eventos del motor.

### Fase 3: persistencia y serialización [COMPLETADA]

- escoger formatos de guardado y exportación (JSON legible implementado),
- garantizar reconstrucción determinista (tests unitarios y de integración de carga de escenarios aprobados),
- versionar el esquema de datos (`CONTRACT_VERSION = 1`).

### Fase 4: compatibilidad y pruebas [PARCIALMENTE COMPLETADA]

- comprobar que frontend y backend interpretan lo mismo,
- validar casos límite de sincronización (test de viaje completo en `tests/smoke.rs`),
- [PENDIENTE] probar la carga, guardado y reproducción interactiva extremo a extremo desde el visualizador.


## 5. Criterios de éxito

La integración será correcta cuando:

- el motor pueda exponer su estado sin acoplarse a la UI,
- el frontend pueda reconstruir el estado inicial y aplicar deltas sin pedir el estado completo en cada tick,
- los escenarios se puedan guardar y recuperar de forma reproducible,
- la separación entre capas permanezca limpia.
