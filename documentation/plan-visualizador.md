# Plan del visualizador de tráfico

Este documento define la implementación del frontend del simulador. Su ámbito es la interfaz de edición, observación y control visual. No decide la lógica del tráfico: esa responsabilidad pertenece al motor.

## 1. Objetivo

Construir un visualizador de escritorio en Rust que permita crear, inspeccionar y reproducir escenarios de tráfico con una interfaz clara, precisa y orientada a edición geométrica.

## 2. Principios de diseño

- La UI representa el estado; no lo inventa.
- La edición propone cambios al modelo, pero el backend valida la verdad final.
- El renderizado debe ser legible, estable y preparado para escenarios grandes.
- La interacción debe priorizar precisión sobre ornamentación.
- La reproducción temporal debe ser clara y controlable.
- La posición visual de cada vehículo debe interpolarse a partir de un progreso normalizado sobre el tramo recibido del backend.
- La edición geométrica debe ofrecer snapping, guías y restricciones de ángulo cuando se activen.

## 3. Alcance funcional

El visualizador debe permitir:

- dibujar y editar redes viales,
- seleccionar nodos, tramos, carriles y vehículos,
- inspeccionar propiedades y estados,
- reproducir, pausar y avanzar la simulación,
- mostrar rutas, colas, congestión y semáforos,
- aplicar capas de depuración y análisis,
- guardar y restaurar configuraciones visuales compatibles con el motor.

## 4. Fases de implementación

### Fase 0: base de interfaz [COMPLETADA]

- definir arquitectura de pantallas y paneles (barra de menú, barra de estado y barra de herramientas lateral implementadas),
- establecer el flujo de datos y modelo de la interfaz (`SimuladorApp`),
- preparar el lienzo y el sistema de selección interactivo por clic,
- dejar definidos los estados de edición y modo de borrado selectivo.

### Fase 1: lienzo y representación [COMPLETADA]

- renderizar la red vial básica en asfalto con mezcla de tramos,
- dibujar marcas de carriles segmentadas con recortes en intersecciones,
- soportar zoom y desplazamiento fluidos en el viewport,
- implementar un sistema de coordenadas estable e independiente de la resolución de pantalla.

### Fase 2: edición geométrica [COMPLETADA]

- crear herramientas de trazado interactivo y elástico (`RoadTool` y `BuildingTool`),
- permitir trazar carreteras magnéticas con snapping configurable por zoom y número de carriles variables,
- implementar el trazado de obstáculos/edificios poligonales con triangulación por orejas (ear clipping) para polígonos complejos,
- evitar la colisión física entre calles y edificios en tiempo de diseño,
- habilitar herramientas de borrado avanzado (`DeleteTool`) con modos de sub-polígono, lazo a mano alzada y elemento completo.

### Fase 3: observación de simulación [PLANIFICADA - SIGUIENTE PASO]

- conectar `SimulationEngine` interactivo dentro de la interfaz gráfica,
- mostrar vehículos en movimiento sobre las carreteras lógicas e interpolar su progreso visual,
- representar visualmente los estados de los semáforos (verde/amarillo/rojo) en las intersecciones,
- añadir paneles de control para reproducir, pausar, resetear e ir paso a paso,
- permitir inspeccionar un vehículo o nodo semafórico obteniendo sus detalles acumulados en tiempo real.

### Fase 4: análisis y depuración [PLANIFICADA]

- incorporar capas de color térmicas según la congestión de los tramos devuelta por el motor,
- habilitar filtros y resaltados de rutas seguidas por vehículos específicos,
- depurar visualmente los vectores de colisión e identificadores lógicos mediante un panel de control técnico.

### Fase 5: pulido y escala [PLANIFICADA]

- revisar el rendimiento visual de renderizado continuo en egui con alta densidad de vehículos,
- pulir atajos de teclado y consistencia de menús contextuales,
- confirmar la compatibilidad y estabilidad del contrato de persistencia JSON.


## 5. Criterios de éxito

El frontend estará bien resuelto cuando:

- permita construir y leer escenarios con precisión,
- se comunique con el motor sin duplicar lógica,
- soporte edición y ejecución sin ambigüedad,
- interpole vehículos sobre curvas sin saltos visibles,
- mantenga una experiencia clara incluso en escenarios densos.
