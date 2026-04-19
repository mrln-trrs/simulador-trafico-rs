# Plan del simulador de tráfico

## Objetivo
Definir un plan de desarrollo para un simulador de tráfico en Rust cuyo foco principal sea el motor: red vial, reglas de decisión, colas, semáforos, congestión, replanificación, eventos y métricas. La red real será solo una entrada de datos para probar la lógica interna.

## Principios de diseño

- Determinista por defecto: con la misma semilla, la misma red y la misma demanda, los resultados deben repetirse.
- Paso temporal discreto: la simulación avanza por ticks y no por tiempo continuo.
- Separación de responsabilidades: red, rutas, motor, métricas y visualización deben estar desacoplados.
- Actualización atómica por tick: las decisiones se toman sobre una instantánea previa y luego se aplican.
- Validación incremental: primero una red pequeña, después congestión, bloqueos y escenarios de estrés.

## Alcance inmediato

Antes de intentar un escenario urbano completo, el simulador debe resolver correctamente estos casos:

- un vehículo entra al sistema, calcula ruta y llega a destino,
- varios vehículos compiten por el mismo tramo,
- una intersección con semáforo restringe movimientos,
- una cola se forma cuando la capacidad se agota,
- una ruta alternativa se vuelve preferible por congestión,
- un bloqueo temporal obliga a desviar vehículos,
- el sistema registra métricas y puede reiniciarse sin arrastrar estado anterior.

## Modelo funcional

### Red vial
La red se modelará como un grafo dirigido con nodos, tramos y carriles.

- Un nodo representa un punto de decisión o control.
- Un tramo representa la conexión entre dos nodos.
- Un carril permite representar colas y ocupación con más detalle.

### Nodo
Cada nodo debe poder representar:

- intersección simple,
- semáforo,
- cruce con prioridad,
- rotonda,
- punto de entrada,
- punto de salida,
- unión o división de vías.

Campos mínimos:

- identificador único,
- nombre o etiqueta,
- posición 2D,
- tipo de nodo,
- entradas y salidas,
- reglas de prioridad,
- estado semafórico,
- bloqueo temporal si existe.

### Tramo
Cada tramo debe conservar:

- identificador único,
- nodo origen,
- nodo destino,
- longitud,
- velocidad máxima,
- número de carriles,
- capacidad por carril y total,
- estado operativo,
- factor de congestión,
- tiempo base,
- tiempo dinámico estimado,
- restricciones de giro o maniobra.

### Carril
El carril sirve para modelar la fila real de entrada o circulación.

- Debe respetar orden FIFO.
- Debe tener capacidad máxima.
- Debe conocer sus vehículos ocupantes.
- Debe poder estar libre, ocupado o bloqueado.
- Debe compatibilizarse con giros o salidas específicas si aplica.

### Vehículo
Cada vehículo debe contener:

- identificador único,
- tipo de vehículo,
- nodo de origen,
- nodo de destino,
- ruta actual,
- posición lógica,
- estado actual,
- tiempo total de viaje,
- tiempo total de espera,
- tiempo detenido en semáforos,
- número de cambios de ruta,
- ruta previa para evitar bucles,
- prioridad si corresponde.

Tipos sugeridos:

- automóvil,
- bus,
- camión,
- motocicleta,
- ambulancia o emergencia.

### Semáforo
El semáforo se tratará como una máquina de estados por fases.

- fase actual,
- duración de cada fase,
- movimientos permitidos,
- tiempo acumulado en la fase,
- modo fijo o adaptativo,
- verde, amarillo o rojo,
- fase todo rojo para limpieza del cruce.

### Evento
El motor debe emitir eventos para depuración y trazabilidad.

- vehículo creado,
- vehículo entra a tramo,
- vehículo sale de tramo,
- vehículo espera en nodo,
- vehículo cambia de ruta,
- semáforo cambia de fase,
- tramo se satura,
- tramo se libera,
- incidente se activa,
- incidente se resuelve,
- vehículo llega a destino.

## Ciclo de simulación

Cada tick debe seguir un orden fijo para evitar ambigüedades.

### Fase de lectura

1. Tomar una instantánea del estado al inicio del tick.
2. Revisar los vehículos que deben entrar.
3. Calcular decisiones de ruta.
4. Detectar conflictos de prioridad.
5. Calcular cambios de fase semafórica.
6. Identificar bloqueos, cierres o cambios de capacidad.

### Fase de aplicación

1. Actualizar semáforos.
2. Mover vehículos con permiso.
3. Liberar vehículos que terminan un tramo.
4. Insertar vehículos en sus nuevas colas.
5. Actualizar contadores de espera y ocupación.
6. Registrar eventos y métricas.

Reglas obligatorias:

- las decisiones del tick se toman sobre el estado anterior al movimiento,
- un vehículo que entra a una cola en este tick no puede salir de ella hasta el siguiente tick,
- un vehículo que entra a un tramo en este tick no puede abandonarlo en el mismo tick.

## Reglas de movimiento y prioridad

- Un vehículo solo avanza si el tramo siguiente tiene capacidad.
- Si el tramo está lleno, el vehículo espera en el nodo actual.
- Si el semáforo no habilita el movimiento, el vehículo permanece detenido.
- Si el tramo tiene bloqueo o cierre, no puede entrar.
- Si el destino ya fue alcanzado, el vehículo sale del sistema.
- Si dos vehículos compiten por la misma capacidad, gana el de mayor prioridad o el que llegó antes.

Orden de desempate sugerido:

1. vehículo de emergencia,
2. transporte público,
3. vehículo con más tiempo esperando,
4. vehículo que llegó primero a la cola,
5. identificador más bajo como último criterio determinista.

## Congestión y rutas

La congestión debe afectar tanto la lógica de movimiento como el cálculo de rutas.

### Variables de congestión

- ocupación del tramo,
- longitud de la cola de entrada,
- tiempo promedio de espera,
- bloqueos en nodos adyacentes,
- saturación por carril,
- historial reciente de flujo.

### Costo dinámico
El costo de una arista debe combinar tiempo base y penalizaciones por ocupación, espera, bloqueo e influencia semafórica.

Forma objetivo:

`costo = tiempo_base + penalización_por_ocupación + penalización_por_espera + penalización_por_incidente`

El costo debe:

- crecer cuando el tramo se acerque a la saturación,
- subir si la cola aumenta,
- reaccionar a bloqueos o cierres,
- considerar el tiempo útil del semáforo,
- evitar oscilaciones absurdas con suavizado temporal y límites máximos.

### Replanificación
Un vehículo puede recalcular ruta cuando:

- el siguiente tramo se cierra,
- la congestión supera un umbral,
- aparece un bloqueo prolongado,
- existe una alternativa claramente mejor.

Para evitar bucles:

- limitar la cantidad de replanificaciones por vehículo,
- recordar los últimos nodos o tramos visitados,
- penalizar repetición de segmentos,
- evitar cambios si la mejora es mínima,
- prohibir volver inmediatamente al tramo recién abandonado salvo emergencia.

## Semáforos e intersecciones

Los semáforos deben controlar los movimientos permitidos por fase.

- Un nodo puede permitir uno o varios movimientos por fase.
- Una fase puede desbloquear una sola arista o varias compatibles.
- El amarillo debe funcionar como transición, no como permiso libre total.
- El rojo completo puede usarse para limpiar la intersección.
- Debe existir modo fijo, adaptativo y manual para pruebas.

## Incidentes y fallos

El simulador debe soportar situaciones de estrés.

### Incidentes posibles

- cierre total de un tramo,
- reducción de carriles,
- semáforo desincronizado,
- demora excesiva en una fase,
- aumento temporal de demanda,
- accidente con bloqueo parcial,
- caída de capacidad por mantenimiento,
- ruta desviada por restricción.

### Efecto esperado

- formación de cola,
- reasignación de rutas,
- propagación de congestión,
- aumento del tiempo medio de viaje,
- posible estabilización tras varios ticks.

## Demanda y escenarios

La entrada de vehículos debe poder definirse de varias formas:

- lista fija de vehículos y horarios,
- generador aleatorio con semilla,
- perfiles por hora pico,
- escenarios de carga progresiva,
- orígenes y destinos ponderados.

Cada escenario debería incluir:

- semilla aleatoria,
- duración total,
- lista de nodos,
- lista de tramos,
- lista de semáforos,
- calendario de demanda,
- incidentes programados,
- parámetros globales del motor.

## Arquitectura en Rust

La implementación debe separarse en módulos claros.

- `model`: nodos, tramos, vehículos y semáforos.
- `network`: construcción y consulta del grafo.
- `routing`: cálculo de rutas y pesos dinámicos.
- `simulation`: motor de ticks y resolución de colas.
- `events`: registro de eventos.
- `metrics`: acumulación y exportación de resultados.
- `scenario`: escenarios de prueba.
- `io`: carga y guardado de configuraciones.
- `visualization`: renderizado futuro.

Colecciones recomendadas:

- `HashMap` para acceso por identificador,
- `VecDeque` para colas FIFO,
- `BinaryHeap` para búsqueda de rutas,
- `Vec` para listas ordenadas de fases o eventos,
- `BTreeMap` si se requiere orden por tiempo o evaluación determinista.

Decisión inicial recomendada:

- el núcleo de simulación debe ser de un solo hilo para mantener determinismo,
- la concurrencia puede dejarse para interfaz, renderizado o escritura de logs.

## Métricas y trazabilidad

### Métricas globales

- tiempo total de simulación,
- vehículos creados,
- vehículos completados,
- vehículos activos,
- tiempo promedio de viaje,
- tiempo promedio de espera,
- tiempo promedio detenido,
- flujo por tick,
- saturación media de la red.

### Métricas por nodo

- cola máxima,
- cola promedio,
- demora acumulada,
- número de vehículos atendidos,
- cantidad de bloqueos,
- utilización de cada fase del semáforo.

### Métricas por tramo

- ocupación máxima,
- ocupación media,
- tiempo en saturación,
- frecuencia de uso,
- tiempo promedio de recorrido real,
- número de veces que se bloqueó.

### Registro

El motor debe poder registrar:

- log de eventos por pantalla,
- archivo CSV por tick,
- archivo CSV por vehículo,
- archivo JSON de depuración completa,
- modo silencioso para pruebas masivas.

## Validación

Antes de considerar estable el motor, debe pasar pruebas automáticas.

### Pruebas unitarias

- creación de nodos y tramos,
- cálculo de rutas,
- actualización de semáforos,
- avance de vehículos,
- formación y liberación de colas,
- cálculo de pesos dinámicos.

### Pruebas de integración

- un vehículo recorre una ruta completa,
- varios vehículos compiten por la misma salida,
- un semáforo restringe correctamente el paso,
- un cierre obliga a desviar vehículos,
- una cola se vacía después de varios ticks.

### Pruebas de estrés

- alta demanda desde un mismo origen,
- cierre de un tramo principal,
- semáforos desincronizados,
- rutas con bucles potenciales,
- red saturada por varios ticks consecutivos.

### Invariantes

- un vehículo no puede estar en dos tramos al mismo tiempo,
- la cola de un carril no puede superar su capacidad,
- un vehículo finalizado no vuelve al sistema,
- una ruta debe ser consistente con la conectividad de la red,
- los tiempos acumulados no deben ser negativos,
- los estados deben ser mutuamente excluyentes.

## Ruta de desarrollo

### Fase 0: definición funcional

- fijar estados y reglas,
- definir métricas,
- decidir el orden de actualización por tick,
- determinar cómo se calculan los pesos dinámicos.

### Fase 1: motor mínimo

- red pequeña,
- vehículos con rutas fijas,
- colas básicas,
- semáforos simples,
- reporte final.

### Fase 2: congestión y rutas dinámicas

- pesos variables,
- replanificación,
- bloqueos temporales,
- prioridades de paso.

### Fase 3: validación y estrés

- escenarios límite,
- casos de bloqueo,
- pruebas de estabilidad,
- medición sistemática.

### Fase 4: visualización y análisis

- interfaz simple,
- colores por saturación,
- exportación de resultados,
- comparación entre escenarios.

## Definición de éxito

El simulador estará bien planteado cuando pueda:

- mover vehículos sin inconsistencias,
- respetar colas y prioridades,
- reaccionar a congestión y bloqueos,
- evitar bucles infinitos de rutas,
- registrar métricas útiles,
- reproducir escenarios con resultados comparables,
- escalar desde un caso pequeño hasta una red más grande sin cambiar la lógica central.

En resumen, el plan debe priorizar primero el comportamiento del motor y después la representación del mapa. Si la lógica interna queda bien definida, cualquier escenario vial será solo un conjunto de datos de entrada.
