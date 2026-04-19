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

### Fase 0: modelo compartido

- fijar tipos base comunes,
- definir identificadores estables,
- separar dominio, vista y transporte,
- decidir qué se comparte y qué no.

### Fase 1: snapshots y estado

- definir la estructura de instantáneas completas y delta,
- acordar qué entidades y campos cambian por tick,
- permitir actualizaciones parciales de posiciones, colas y estados,
- asegurar lecturas consistentes por tick.

### Fase 2: comandos y eventos

- definir comandos de edición,
- definir eventos emitidos por el motor,
- establecer respuesta a cambios de escenario,
- hacer explícito el flujo de ida y vuelta.

### Fase 3: persistencia y serialización

- escoger formatos de guardado y exportación,
- usar formatos legibles para depuración y compactos o binarios para corridas masivas,
- garantizar reconstrucción determinista,
- versionar el esquema de datos,
- preparar migraciones si cambia el modelo.

### Fase 4: compatibilidad y pruebas

- comprobar que frontend y backend interpretan lo mismo,
- validar casos límite de sincronización,
- probar carga, guardado y reproducción,
- revisar compatibilidad entre versiones.

## 5. Criterios de éxito

La integración será correcta cuando:

- el motor pueda exponer su estado sin acoplarse a la UI,
- el frontend pueda reconstruir el estado inicial y aplicar deltas sin pedir el estado completo en cada tick,
- los escenarios se puedan guardar y recuperar de forma reproducible,
- la separación entre capas permanezca limpia.
