# Plan general del simulador de tráfico

Este documento coordina el desarrollo completo del simulador, pero no repite el detalle técnico de cada capa. Su función es ordenar el trabajo, fijar dependencias y dejar claro qué documento gobierna cada parte del sistema.

La separación objetivo queda así:

- `idea-simulador`: visión global del producto,
- `idea-motor`: visión conceptual del backend,
- `idea-visualizador`: visión conceptual del frontend,
- `plan-simulador`: hoja de ruta global y coordinación,
- `plan-motor`: implementación del backend,
- `plan-visualizador`: implementación del frontend,
- `plan-integracion`: contratos, sincronización y persistencia compartida.

## 1. Propósito

Construir una plataforma de simulación de tráfico en Rust con un motor determinista, un visualizador interactivo y una capa de integración estable entre ambas partes. El plan global no define cómo se implementa cada detalle, sino en qué orden deben construirse las piezas para que el proyecto avance sin mezclar responsabilidades.

## 2. Principios de organización

- El backend decide el estado lógico de la simulación.
- El frontend construye, muestra e inspecciona escenarios.
- La integración define cómo se comunican ambas capas.
- Los contratos compartidos deben ser explícitos y versionables.
- Cada plan específico debe poder evolucionar sin romper el resto.

## 3. Mapa de desarrollo

### 3.1 Backend

El backend se desarrolla siguiendo `plan-motor`. Ese plan cubre la red vial, el ciclo de ticks, el movimiento, la congestión, las rutas, las métricas y la persistencia lógica.

### 3.2 Frontend

El frontend se desarrolla siguiendo `plan-visualizador`. Ese plan cubre el lienzo, la edición geométrica, los paneles de inspección, la reproducción de la simulación y las ayudas visuales.

### 3.3 Integración

La comunicación entre capas se desarrolla siguiendo `plan-integracion`. Ese plan cubre snapshots completos y delta, comandos, eventos, serialización y compatibilidad entre versiones.

## 4. Fases del proyecto

### Fase 0: definición y límites

- cerrar el alcance de cada capa,
- fijar los tipos compartidos mínimos,
- definir qué datos son de dominio y cuáles son de presentación,
- establecer reglas de determinismo y versionado.

### Fase 1: motor base

- construir la base lógica del simulador,
- validar cargas simples,
- asegurar que el backend puede ejecutarse y producir estado consistente,
- dejar preparada la API de consumo para otras capas.

### Fase 2: interfaz visual básica

- crear el entorno visual inicial,
- permitir cargar y representar escenarios,
- habilitar inspección y edición mínima,
- verificar que la UI consume correctamente el estado del motor.

### Fase 3: contratos e integración completa

- estabilizar snapshots completos, delta y comandos,
- sincronizar edición y ejecución,
- definir el flujo de persistencia compartida,
- resolver compatibilidad entre frontend y backend.

### Fase 4: funciones avanzadas

- añadir congestión, replanificación, eventos y métricas donde corresponda,
- mejorar la edición y la depuración visual,
- cerrar huecos de usabilidad y trazabilidad,
- reforzar validación y pruebas.

### Fase 5: endurecimiento y escala

- revisar rendimiento,
- confirmar reproducibilidad,
- probar escenarios más grandes,
- preparar el proyecto para crecimiento sin reescrituras.

## 5. Criterios de éxito

El plan global se considera bien resuelto cuando:

- cada capa tiene su propio documento y su propia responsabilidad,
- no se repiten reglas de implementación entre planes,
- backend, frontend e integración pueden evolucionar por separado,
- la simulación sigue siendo determinista y reproducible,
- el proyecto puede crecer sin perder claridad documental.
