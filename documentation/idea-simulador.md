# Idea general del simulador de tráfico

Este documento es la visión global del proyecto. Define qué es el simulador, qué capacidades debe cubrir y cómo se separa en backend, frontend e integración.

El backend contiene la lógica del motor, el frontend contiene la edición y visualización, y la integración define los contratos compartidos. Todo lo demás son modelos comunes y documentación de apoyo. No existe una tercera aplicación independiente.

## 1. Propósito global

Construir en Rust una plataforma de simulación de tráfico que permita diseñar, ejecutar y analizar redes viales completas con comportamiento determinista, escenarios reproducibles y visualización interactiva.

El sistema debe servir para:

- modelar redes viales urbanas y suburbanas,
- crear escenarios de prueba y escenarios realistas,
- simular vehículos, colas, semáforos, congestión y bloqueos,
- editar la red con herramientas geométricas precisas,
- visualizar el estado del sistema en tiempo real,
- exportar resultados y comparar corridas,
- repetir escenarios con la misma semilla y obtener el mismo resultado.

## 2. División funcional del proyecto

### 2.1 Backend

El backend es responsable de la lógica que decide qué ocurre en cada tick.

Debe encargarse de:

- representar la red vial como grafo,
- validar conectividad y restricciones,
- administrar vehículos, colas y rutas,
- aplicar reglas de prioridad y control,
- actualizar semáforos y eventos,
- calcular congestión y costos dinámicos,
- registrar métricas,
- guardar y recuperar escenarios,
- emitir snapshots y eventos para el frontend.

### 2.2 Frontend

El frontend es la capa visual e interactiva.

Debe encargarse de:

- dibujar el mapa y la red,
- permitir editar geometría y propiedades,
- mostrar el estado de la simulación,
- reproducir, pausar y avanzar escenarios,
- resaltar rutas, colas y congestión,
- inspeccionar nodos, tramos y vehículos,
- presentar paneles de control, propiedades y métricas.

### 2.3 Contratos compartidos

Ambas capas comparten un conjunto de tipos y reglas comunes:

- identificadores de nodos, tramos y vehículos,
- geometría y coordenadas,
- estados y eventos,
- estructuras de escenario,
- formatos de exportación,
- snapshots de visualización,
- parámetros globales del simulador.

## 3. Alcance total de características

El simulador debe cubrir todas las características principales de una plataforma completa de tráfico:

- geometría 2D con mapa base,
- trazado de nodos, tramos, carriles y marcas,
- vías rectas, curvas, desdoblamientos y ramificaciones,
- intersecciones simples y complejas,
- semáforos, prioridades y matrices de giro,
- rotondas y pasos a desnivel,
- fuentes y sumideros de demanda,
- eventos temporales y alteraciones de capacidad,
- simulación discreta por ticks,
- replanificación de rutas,
- telemetría, métricas y exportación,
- modo edición y modo ejecución,
- depuración visual e inspección en tiempo real,
- persistencia de escenarios y reproducibilidad,
- validación automática e integración con pruebas,
- escalabilidad desde escenarios pequeños hasta redes complejas.

## 4. Catálogo de capacidades por área

### 4.1 Geometría y cartografía

La red debe poder representarse sobre un lienzo 2D con coordenadas consistentes. El sistema debe soportar:

- mapa base de referencia,
- proyección y escala,
- conversión entre coordenadas de pantalla y del mundo,
- dibujo de líneas, curvas y segmentos compuestos,
- separación visual entre sentidos de circulación,
- ajuste fino de posición, rotación y longitud.

### 4.2 Red vial y control

El modelo vial debe abarcar:

- nodos de entrada, salida, intersección y unión,
- tramos unidireccionales y bidireccionales,
- carriles con capacidad y restricciones,
- sentidos de circulación,
- giros permitidos y prohibidos,
- rotondas,
- pasos a desnivel,
- nodos semaforizados,
- nodos con prioridad o control especial.

### 4.3 Demanda y escenarios

El simulador debe permitir definir el origen del tráfico con flexibilidad:

- generadores y sumideros,
- tasas de inyección,
- perfiles horarios,
- vehículos con distinto tipo y prioridad,
- escenarios fijos o aleatorios,
- semilla reproducible,
- demanda progresiva o por horas pico.

### 4.4 Ejecución y control temporal

El sistema debe correr por ticks discretos y no por tiempo continuo. Debe incluir:

- iniciar, pausar, reanudar y reiniciar,
- paso a paso,
- velocidad de reproducción ajustable,
- eventos programados en el tiempo,
- cambios temporales de capacidad o velocidad,
- cierre parcial o total de tramos,
- gestión de colas y prioridades.

### 4.5 Simulación y comportamiento

La lógica de tráfico debe cubrir:

- vehículos con estado propio,
- colas y ocupación de carriles,
- saturación y congestión,
- rutas iniciales y replanificación,
- interacción con semáforos,
- bloqueos, incidentes y desvíos,
- reglas de desempate deterministas,
- estados consistentes y mutuamente excluyentes.

### 4.6 Visualización y análisis

La presentación visual debe incluir:

- renderizado de la red,
- renderizado de vehículos,
- resaltado de rutas,
- heatmap de congestión,
- paneles de propiedades,
- inspector de elementos,
- información temporal y estadística,
- ayudas de edición y depuración.

### 4.7 Persistencia y exportación

El proyecto debe guardar y recuperar información de forma estable:

- escenarios completos,
- configuraciones de red,
- demanda y eventos temporales,
- estados semafóricos,
- métricas y resultados,
- trazas de simulación,
- formatos interoperables para análisis externo.

### 4.8 Calidad y escalabilidad

El sistema debe mantenerse:

- determinista,
- reproducible,
- validable con pruebas,
- modular,
- extensible,
- preparado para crecer sin reescritura total.

## 5. Flujo general del producto

1. El usuario crea o carga una red en el frontend.
2. El frontend valida y envía cambios al backend.
3. El backend normaliza la red y prepara el escenario.
4. El usuario define demanda, eventos y parámetros.
5. El backend ejecuta la simulación por ticks.
6. El frontend consume snapshots y eventos para dibujar el estado.
7. El usuario inspecciona, ajusta y vuelve a ejecutar si es necesario.
8. Los resultados se exportan o se comparan con otras corridas.

## 6. Documentos derivados

Este documento es el punto de entrada general del proyecto. Los detalles se separan así:

- idea-motor: visión y reglas del backend de simulación,
- idea-visualizador: visión y reglas del frontend visual e interactivo,
- plan-simulador: plan funcional global de desarrollo,
- plan-motor: plan funcional de implementación del backend,
- plan-visualizador: plan funcional de implementación del frontend,
- plan-integracion: plan de contratos, sincronización y persistencia compartida.

## 7. Criterio de éxito

La idea general quedará bien definida cuando el proyecto pueda:

- modelar una red vial completa,
- ejecutar escenarios reproducibles,
- mostrar el estado del tráfico con claridad,
- permitir edición y análisis sin mezclar responsabilidades,
- crecer en complejidad sin perder orden conceptual,
- mantener backend y frontend separados, pero coordinados.

## 8. Estado de la implementación

Actualmente, el proyecto ha completado de forma sólida la primera iteración de desarrollo:
- **Backend (Motor de Simulación)**: Totalmente implementado a nivel conceptual de tráfico por ticks. Se han integrado vehículos, semáforos, Dijkstra dinámico, y el cálculo de colas de congestión.
- **Frontend (Editor Visual)**: Lienzo interactivo en egui implementado, permitiendo trazar carreteras magnéticas y obstáculos poligonales de forma manual.
- **Próximo Objetivo**: Acoplar ambas partes en tiempo de ejecución. La UI deberá inicializar el motor con la red trazada por el usuario y renderizar los snapshots en tiempo real.

En resumen, este documento engloba la intención total del proyecto: una plataforma de simulación de tráfico en Rust, con motor lógico en backend y visualización interactiva en frontend, ambos unidos por modelos y contratos compartidos.