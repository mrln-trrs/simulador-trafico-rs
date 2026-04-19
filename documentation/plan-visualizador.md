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

### Fase 0: base de interfaz

- definir arquitectura de pantallas y paneles,
- establecer el flujo de datos con el backend,
- preparar el lienzo y el sistema de selección,
- dejar claros los estados de modo edición y modo ejecución.

### Fase 1: lienzo y representación

- renderizar la red vial básica,
- dibujar nodos, tramos y carriles,
- mapear el progreso normalizado a coordenadas visuales con interpolación,
- soportar zoom, desplazamiento y selección,
- mantener una jerarquía visual limpia.

### Fase 2: edición geométrica

- crear herramientas de trazado y ajuste,
- permitir mover, rotar y redimensionar elementos,
- activar snapping a rejilla, ángulos y ortogonalidad cuando corresponda,
- mostrar guías, referencias y validaciones visuales,
- facilitar deshacer y rehacer.

### Fase 3: observación de simulación

- mostrar vehículos en movimiento,
- representar colas, estados semafóricos y rutas,
- añadir paneles de inspección,
- permitir seguimiento de entidades durante la ejecución.

### Fase 4: análisis y depuración

- incorporar capas de congestión y métricas visuales,
- habilitar filtros y resaltados,
- mejorar la lectura de escenarios complejos,
- preparar herramientas de diagnóstico.

### Fase 5: pulido y escala

- revisar rendimiento visual,
- ajustar ergonomía y consistencia,
- validar escenarios grandes,
- confirmar compatibilidad con el contrato de integración.

## 5. Criterios de éxito

El frontend estará bien resuelto cuando:

- permita construir y leer escenarios con precisión,
- se comunique con el motor sin duplicar lógica,
- soporte edición y ejecución sin ambigüedad,
- interpole vehículos sobre curvas sin saltos visibles,
- mantenga una experiencia clara incluso en escenarios densos.
