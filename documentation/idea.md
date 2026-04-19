## Plan funcional detallado del simulador

El objetivo principal ya no es describir un escenario real concreto, sino definir con precisión cómo debe funcionar el simulador por dentro. El escenario vial real será solo una entrada de datos para alimentar el motor; primero se debe dejar sólida la lógica de funcionamiento.

## 1. Objetivo del sistema

Construir un simulador de tráfico en Rust que permita:

- representar una red vial como un grafo dirigido,
- introducir vehículos con rutas y horarios de aparición,
- modelar colas, capacidad de tramos, semáforos y congestión,
- recalcular rutas cuando cambien las condiciones del tráfico,
- medir tiempos de espera, flujo y saturación,
- reproducir escenarios de prueba de forma determinista.

El foco está en la lógica del motor, no en la visualización ni en la calibración con un mapa real.

## 2. Principios de diseño

El simulador debe seguir estas decisiones base:

- **Determinista por defecto:** con la misma semilla y los mismos datos, el resultado debe ser idéntico.
- **Paso temporal discreto:** el sistema avanza por ticks, no por tiempo continuo.
- **Separación de responsabilidades:** red vial, motor de simulación, rutas, métricas y visualización deben estar separados.
- **Modelo orientado a eventos internos:** cada tick puede generar eventos de entrada, salida, espera, cambio de semáforo o bloqueo.
- **Validación incremental:** primero debe funcionar una red pequeña y luego escalar a escenarios más grandes.

## 3. Alcance funcional inmediato

Antes de pensar en una ciudad completa, el simulador debe poder resolver correctamente estas situaciones:

- un vehículo entra al sistema, calcula ruta y llega a destino,
- varios vehículos compiten por el mismo tramo,
- una intersección con semáforo deja pasar solo ciertos movimientos,
- una cola se forma cuando la capacidad de un tramo se agota,
- una ruta alternativa se vuelve preferible por congestión,
- un bloqueo temporal obliga a desviar vehículos,
- el sistema registra métricas y puede reiniciarse sin arrastrar estado anterior.

## 4. Entidades del simulador

### 4.1 Red vial

La red se modela como un grafo dirigido con nodos y aristas.

- **Nodo:** punto de decisión o control.
- **Arista o tramo:** conexión entre dos nodos.
- **Carril:** subunidad del tramo donde pueden ubicarse vehículos.

La red debe permitir vías de una sola dirección y bidireccionales representadas como dos aristas separadas.

### 4.2 Nodo

Cada nodo debe poder representar:

- intersección simple,
- semáforo,
- cruce con prioridad,
- rotonda,
- punto de entrada,
- punto de salida,
- unión o división de vías.

Campos recomendados:

- identificador único,
- nombre o etiqueta,
- posición 2D para visualización,
- tipo de nodo,
- lista de aristas entrantes,
- lista de aristas salientes,
- reglas de prioridad,
- estado semafórico actual,
- ciclo semafórico si aplica,
- estado de bloqueo temporal si existe.

### 4.3 Tramo

Cada tramo debe guardar:

- identificador único,
- nodo origen,
- nodo destino,
- longitud,
- velocidad máxima,
- número de carriles,
- capacidad por carril,
- capacidad total,
- estado operativo,
- factor de congestión,
- tiempo base de recorrido,
- tiempo estimado dinámico,
- restricción de giro o maniobra si existe.

### 4.4 Carril

El carril es importante si se quiere representar filas reales con más detalle.

Debe incluir:

- orden FIFO,
- capacidad máxima,
- vehículos ocupantes,
- longitud efectiva,
- estado libre, ocupado o bloqueado,
- compatibilidad con giros o salidas específicas.

### 4.5 Vehículo

Cada vehículo debe tener:

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
- prioridad si aplica.

Tipos de vehículo sugeridos:

- automóvil,
- bus,
- camión,
- motocicleta,
- ambulancia o emergencia.

### 4.6 Semáforo

Debe modelarse como una máquina de estados con fases.

Campos sugeridos:

- fase actual,
- duración de cada fase,
- conjunto de movimientos permitidos,
- tiempo acumulado en la fase,
- modo fijo o adaptativo,
- estado verde, amarillo o rojo,
- opción de todo rojo para limpieza de cruce.

### 4.7 Evento

El motor debe generar eventos para poder depurar y registrar el sistema.

Ejemplos:

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

## 5. Estados y transiciones

### 5.1 Estados del vehículo

Un vehículo puede estar en uno de estos estados:

- esperando para entrar,
- esperando en nodo,
- entrando a un tramo,
- circulando por un tramo,
- detenido por semáforo,
- detenido por saturación,
- replanificando ruta,
- finalizado.

### 5.2 Estados del tramo

Un tramo puede estar:

- libre,
- cargado,
- saturado,
- bloqueado temporalmente,
- cerrado,
- con capacidad parcial,
- en recuperación después de un incidente.

### 5.3 Estados del nodo

Un nodo puede estar:

- operativo,
- controlado por semáforo,
- en prioridad libre,
- bloqueado,
- en mantenimiento,
- con colas activas.

## 6. Ciclo de simulación

Cada tick debe seguir un orden fijo para evitar ambigüedades.

### Orden recomendado por tick

1. Ingresar vehículos programados para ese tick.
2. Actualizar semáforos y eventos de control.
3. Actualizar incidentes, cierres o cambios de capacidad.
4. Calcular o refrescar pesos dinámicos de rutas.
5. Avanzar vehículos dentro de tramos.
6. Liberar vehículos que llegan a nodos.
7. Intentar mover vehículos desde nodos hacia tramos salientes.
8. Resolver colas con reglas de prioridad.
9. Registrar métricas del tick.
10. Evaluar condiciones de término.

### Regla importante

Un vehículo que entra a un tramo en un tick no debe abandonar ese mismo tramo en el mismo tick. Esto evita saltos imposibles y simplifica la consistencia del modelo.

## 7. Reglas de movimiento

El movimiento debe obedecer estas reglas:

- un vehículo solo avanza si el tramo siguiente tiene capacidad,
- si el tramo está lleno, el vehículo espera en el nodo actual,
- si el semáforo no habilita su movimiento, permanece detenido,
- si el tramo tiene bloqueo o cierre, no puede entrar,
- si el destino final ya fue alcanzado, el vehículo sale del sistema,
- si dos vehículos compiten por la misma capacidad, gana el que tenga mayor prioridad o el que llegó antes.

### Reglas de prioridad sugeridas

El orden de desempate puede ser:

1. vehículo de emergencia,
2. transporte público,
3. vehículo con más tiempo esperando,
4. vehículo que llegue primero a la cola,
5. identificador como último criterio determinista.

## 8. Modelo de congestión

La congestión no debe ser solo visual; debe afectar la lógica.

### Variables de congestión

- ocupación del tramo,
- longitud de la cola de entrada,
- tiempo promedio de espera,
- bloqueos en nodos adyacentes,
- saturación por carril,
- historial reciente de flujo.

### Peso dinámico de una arista

El costo de una ruta puede calcularse con una fórmula del tipo:

`costo = tiempo_base + penalización_por_ocupación + penalización_por_espera + penalización_por_incidente`

La penalización puede crecer si:

- el tramo se acerca a la saturación,
- la cola de entrada aumenta,
- un nodo vecino está bloqueado,
- existe un semáforo con poco tiempo útil,
- el tramo se usa repetidamente en desvíos recientes.

### Importante

La congestión debe propagarse de forma realista hacia rutas vecinas, pero sin generar oscilaciones absurdas. Para eso conviene usar suavizado temporal en los pesos y límites máximos de penalización.

## 9. Rutas y replanificación

### Ruta inicial

Cada vehículo necesita una ruta inicial calculada desde origen a destino.

### Criterio de búsqueda

Puede usarse Dijkstra o A*.

### Criterio de costo

La ruta no debe depender solo de la distancia; debe considerar:

- longitud física,
- velocidad máxima,
- congestión actual,
- bloqueos activos,
- tiempo semafórico estimado,
- penalización por giros prohibidos.

### Replanificación

Un vehículo puede recalcular ruta cuando:

- el tramo siguiente queda cerrado,
- la congestión supera un umbral,
- se detecta un bloqueo prolongado,
- una ruta alternativa mejora claramente el tiempo estimado.

### Prevención de bucles

Para evitar rutas circulares o cambios interminables:

- limitar la cantidad de replanificaciones por vehículo,
- recordar los últimos nodos o tramos visitados,
- aplicar una penalización fuerte a segmentos repetidos,
- evitar cambiar ruta si la mejora es demasiado pequeña,
- prohibir volver inmediatamente al tramo recién abandonado salvo emergencia.

## 10. Semáforos y control de intersecciones

Los semáforos deben controlar qué movimientos están permitidos en cada fase.

### Elementos mínimos

- fase verde,
- fase amarilla,
- fase roja,
- tiempo de limpieza entre fases si es necesario.

### Tipos de control

- ciclo fijo,
- ciclo adaptativo,
- prioridad manual para pruebas,
- modo de emergencia con paso restringido.

### Reglas en cruces

- un nodo puede permitir uno o varios movimientos por fase,
- una fase puede desbloquear una sola arista o varias compatibles,
- el amarillo debe servir como transición y no como paso libre total,
- el rojo completo puede usarse para limpiar la intersección antes del siguiente flujo.

## 11. Incidentes y fallos

El simulador debe poder reproducir situaciones de estrés.

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
- aumento de tiempo medio de viaje,
- posible estabilización después de cierto número de ticks.

## 12. Generación de demanda

La entrada de vehículos debe poder definirse de varias maneras.

### Opciones

- lista fija de vehículos y horarios,
- generador aleatorio con semilla,
- perfiles por hora pico,
- escenarios de carga progresiva,
- orígenes y destinos ponderados.

### Variables de demanda

- frecuencia de llegada,
- distribución por origen,
- distribución por destino,
- tipo de vehículo,
- intensidad horaria,
- picos y valles.

## 13. Arquitectura en Rust

La implementación debe dividirse en módulos.

### Módulos sugeridos

- `model`: tipos base de nodos, tramos, vehículos y semáforos,
- `network`: construcción y consulta del grafo,
- `routing`: cálculo de rutas y pesos dinámicos,
- `simulation`: motor de ticks y resolución de colas,
- `events`: registro de eventos del sistema,
- `metrics`: acumulación y exportación de resultados,
- `scenario`: escenarios de prueba,
- `io`: carga y guardado de configuraciones,
- `visualization`: renderizado futuro.

### Colecciones útiles

- `HashMap` para acceso por identificador,
- `VecDeque` para colas FIFO,
- `BinaryHeap` para búsqueda de rutas,
- `Vec` para listas ordenadas de fases o eventos,
- `BTreeMap` si se requiere ordenar por tiempo.

### Decisión recomendada

El núcleo de simulación debería ser de un solo hilo al inicio para mantener determinismo. La concurrencia puede dejarse para la interfaz, el renderizado o la escritura de logs.

## 14. Configuración y archivos de entrada

Conviene que el simulador pueda cargarse desde configuración externa.

### Datos configurables

- nodos,
- tramos,
- carriles,
- semáforos,
- horarios de demanda,
- bloqueos temporales,
- tipos de vehículo,
- pesos de rutas,
- parámetros de congestión.

### Formatos posibles

- TOML,
- JSON,
- CSV para demanda y resultados.

## 15. Métricas que debe producir

Las métricas deben servir tanto para depurar como para presentar resultados.

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

## 16. Registro y trazabilidad

El motor debe poder registrar lo que hace.

### Formas de registro

- log de eventos por pantalla,
- archivo CSV por tick,
- archivo CSV por vehículo,
- archivo JSON para depuración completa,
- modo silencioso para pruebas masivas.

### Datos recomendados en el log

- tick,
- evento,
- vehículo,
- nodo,
- tramo,
- semáforo,
- tiempo de espera,
- tiempo de viaje,
- ruta actual,
- estado de congestión.

## 17. Validación lógica

Antes de presentar resultados, el simulador debe pasar pruebas automáticas.

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

### Invariantes que nunca deben romperse

- un vehículo no puede estar en dos tramos al mismo tiempo,
- la cola de un carril no puede superar su capacidad,
- un vehículo finalizado no vuelve al sistema,
- una ruta debe ser consistente con la conectividad de la red,
- los tiempos acumulados no deben ser negativos,
- los estados deben ser mutuamente excluyentes.

## 18. Visualización futura

La visualización puede añadirse después de estabilizar la lógica.

### Opciones

- terminal con texto y colores,
- `egui` para paneles interactivos,
- `macroquad` para una vista 2D,
- exportación de datos para gráficos externos.

### Qué mostrar

- nodos,
- tramos,
- vehículos en movimiento,
- colas,
- semáforos,
- saturación por color,
- métricas en tiempo real.

## 19. Ruta de desarrollo recomendada

### Fase 0: definición funcional

- fijar estados y reglas,
- definir métricas,
- escoger el orden de actualización por tick,
- decidir cómo se calculan los pesos dinámicos.

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

## 20. Definición del éxito

El simulador estará bien planteado cuando pueda:

- mover vehículos sin inconsistencias,
- respetar colas y prioridades,
- reaccionar a congestión y bloqueos,
- evitar bucles infinitos de rutas,
- registrar métricas útiles,
- reproducir escenarios con resultados comparables,
- escalar desde un caso pequeño hasta una red más grande sin cambiar la lógica central.

En resumen, el proyecto no debe empezar por el mapa, sino por el comportamiento. Si el motor está bien definido, luego cualquier delimitación geográfica solo será un conjunto de datos de entrada.

## 21. Especificación técnica del motor

Esta sección deja definido lo que el motor necesita para funcionar sin ambigüedades. Si alguna de estas reglas no se fija desde el inicio, el simulador puede producir resultados distintos según el orden de recorrido de las estructuras internas.

### 21.1 Contrato de ejecución

El motor debe exponer, como mínimo, estas operaciones:

- inicializar una simulación con red, demanda, parámetros y semilla,
- avanzar un tick de simulación,
- ejecutar la simulación hasta que se alcance un límite o no queden vehículos activos,
- reiniciar el estado para volver a correr el mismo escenario,
- consultar el estado actual de nodos, tramos, vehículos, semáforos y métricas,
- exportar eventos y resultados finales.

### 21.2 Orden exacto de cada tick

Cada tick debe dividirse en dos fases lógicas:

**Fase de lectura**

- se toma una instantánea del estado al inicio del tick,
- se revisan los vehículos que deben entrar,
- se calculan decisiones de ruta,
- se detectan conflictos de prioridad,
- se calculan cambios de fase semafórica,
- se identifican bloqueos, cierres o cambios de capacidad.

**Fase de aplicación**

- se actualizan los semáforos,
- se mueven los vehículos que sí obtuvieron permiso,
- se liberan los vehículos que terminan un tramo,
- se insertan los vehículos en sus nuevas colas,
- se actualizan contadores de espera y ocupación,
- se registran eventos y métricas.

Regla obligatoria: las decisiones de un tick deben tomarse sobre el estado anterior al movimiento, no sobre un estado parcialmente modificado dentro del mismo tick.

Regla adicional: un vehículo que entra a una cola en este tick no puede salir de ella hasta el siguiente tick.

### 21.3 Resolución de conflictos

Cuando varios vehículos quieran usar el mismo recurso, el motor debe resolver el conflicto con una política fija y determinista.

#### Conflictos posibles

- varios vehículos quieren salir del mismo nodo,
- varios vehículos quieren entrar al mismo tramo,
- dos movimientos compiten por un cruce incompatible,
- una fase semafórica habilita más movimientos de los que puede absorber el tramo de destino,
- una cola supera la capacidad disponible.

#### Orden de prioridad

El desempate debe seguir este orden:

1. vehículo de emergencia,
2. transporte público,
3. vehículo con mayor tiempo de espera,
4. vehículo que llegó primero a la cola,
5. identificador más bajo como criterio final estable.

#### Reglas adicionales

- si un movimiento no cabe, el vehículo permanece en cola,
- si una fase semafórica no autoriza el giro, el vehículo no avanza,
- si el tramo de destino está saturado, el movimiento se pospone,
- si hay varias salidas compatibles, el motor debe decidir de forma reproducible y no aleatoria.

### 21.4 Modelo exacto de congestión

La congestión debe influir en el cálculo de rutas y en la decisión de mover vehículos.

#### Variables base

- `ocupacion`: cantidad de vehículos presentes en el tramo,
- `capacidad`: cantidad máxima permitida en el tramo,
- `cola`: vehículos esperando entrar,
- `estado_semaforo`: verde, amarillo o rojo,
- `bloqueo`: cierre parcial o total,
- `incidente`: presencia de accidente, obra o restricción temporal.

#### Fórmula sugerida

El costo dinámico de una arista puede calcularse así:

`costo = tiempo_base * (1 + alfa * ocupacion_relativa + beta * cola_relativa + gamma * bloqueo + delta * penalizacion_semaforo)`

Donde:

- `ocupacion_relativa` = vehículos en el tramo / capacidad del tramo,
- `cola_relativa` = vehículos esperando / capacidad de entrada,
- `bloqueo` = 0 si no hay bloqueo, 1 si existe bloqueo parcial, 2 si existe bloqueo total,
- `penalizacion_semaforo` = estimación del tiempo perdido por la fase actual del nodo de salida.

#### Reglas de estabilidad

- el costo nunca debe ser negativo,
- el costo debe tener un máximo acotado para evitar explosiones numéricas,
- la congestión debe suavizarse con una media móvil o un factor de inercia,
- una congestión muy alta debe aumentar el costo, pero no provocar oscilaciones inestables entre rutas.

### 21.4.1 Colecciones deterministas en Rust

Para que la simulación sea reproducible, las estructuras internas que se recorren durante la lógica del motor no deben depender del orden aleatorio de `HashMap`.

#### Regla de implementación

- usar `BTreeMap` en las colecciones que afecten al orden de evaluación del motor,
- si en el futuro se necesita una tabla hash, usar un *hasher* determinista,
- no depender del orden de inserción accidental para resolver conflictos o producir eventos.

### 21.5 Contrato de entrada de demanda

La demanda debe describirse de forma explícita para que el escenario sea reproducible.

#### Cada vehículo de entrada debe poder definir:

- identificador opcional,
- tipo de vehículo,
- nodo de origen,
- nodo de destino,
- instante de aparición,
- prioridad,
- ruta preferida opcional,
- comportamiento especial opcional.

#### Cada escenario debe poder definir:

- semilla aleatoria,
- duración total de la corrida,
- lista de nodos,
- lista de tramos,
- lista de semáforos,
- calendario de demanda,
- incidentes programados,
- parámetros globales del motor.

#### Validaciones de entrada

- los identificadores deben ser únicos,
- todo tramo debe conectar nodos existentes,
- toda ruta predefinida debe ser válida,
- toda fase semafórica debe referirse a movimientos reales,
- todos los valores numéricos deben estar en rangos positivos o válidos.

### 21.6 Replanificación de rutas

La replanificación debe ser controlada, no continua.

#### Cuándo recalcular

- el siguiente tramo se cerró,
- la ruta fue bloqueada por un incidente,
- el tiempo estimado empeoró por encima de un umbral,
- el vehículo lleva demasiado tiempo detenido,
- existe una alternativa claramente mejor.

#### Cuándo no recalcular

- si la mejora es mínima,
- si el vehículo acaba de replanificar,
- si recalcular lo enviaría de vuelta a un tramo recién abandonado,
- si el vehículo ya excedió su número máximo de cambios de ruta.

#### Prevención de bucles

- mantener un historial corto de tramos visitados,
- penalizar severamente repetir la misma secuencia,
- limitar la cantidad total de replanificaciones por vehículo,
- aplicar enfriamiento entre cambios de ruta.

### 21.6.1 Recuperación de deadlocks

Cuando un vehículo permanece detenido demasiados ticks por saturación, el motor debe activar una excepción controlada para evitar que la simulación quede congelada permanentemente.

#### Política recomendada

- definir un umbral configurable de espera por nodo,
- intentar primero una reruta de emergencia evitando el tramo bloqueado,
- si no existe ruta alternativa, forzar una liberación controlada del cuello de botella,
- registrar siempre el evento para que el caso sea visible en los resultados.

### 21.7 Semáforos y movimientos permitidos

Cada semáforo debe declarar explícitamente qué movimientos habilita.

#### Un movimiento puede describirse como:

- tramo de entrada,
- nodo de cruce,
- tramo de salida,
- tipo de giro o maniobra.

#### El motor debe guardar:

- movimientos compatibles por fase,
- duración de cada fase,
- tiempo restante de la fase,
- transición amarillo o todo rojo si aplica,
- modo fijo o adaptativo.

### 21.8 Métricas internas del motor

Además de las métricas finales, el motor debe mantener contadores durante la corrida.

#### Métricas globales mínimas

- vehículos creados,
- vehículos finalizados,
- vehículos activos,
- tiempo medio de viaje,
- tiempo medio de espera,
- tiempo medio detenido,
- número de replanificaciones,
- número de bloqueos,
- número de eventos registrados.

#### Métricas por nodo

- cola máxima,
- cola promedio,
- tiempo de espera acumulado,
- número de pasos verdes y rojos,
- vehículos atendidos,
- frecuencia de saturación.

#### Métricas por tramo

- ocupación máxima,
- ocupación promedio,
- tiempo en saturación,
- frecuencia de uso,
- veces que fue bloqueado,
- tiempo medio de recorrido real.

### 21.9 Registros y trazabilidad

El motor debe producir un historial que permita depurar el comportamiento.

#### Eventos mínimos

- creación de vehículo,
- entrada a cola,
- avance a tramo,
- espera por semáforo,
- espera por saturación,
- cambio de ruta,
- cambio de fase semafórica,
- activación o resolución de incidente,
- llegada a destino.

#### Formatos de salida

- consola para depuración rápida,
- CSV para análisis,
- JSON para inspección completa,
- resumen final agregado.

### 21.10 Reglas de reproducibilidad

Para que dos corridas sean comparables, el motor debe fijar estas condiciones:

- misma semilla aleatoria,
- misma red,
- misma demanda,
- mismos parámetros de congestión,
- mismo orden de actualización por tick,
- mismas políticas de desempate.

### 21.11 Invariantes del motor

Estas condiciones no deben romperse nunca:

- un vehículo no puede existir simultáneamente en dos tramos,
- una cola no puede superar la capacidad definida,
- un vehículo finalizado no vuelve al sistema,
- un tramo cerrado no debe aceptar entradas,
- un movimiento prohibido no debe ser autorizado,
- un costo de ruta no debe ser negativo,
- un estado del sistema no debe depender del azar si la semilla es la misma.

### 21.12 Orden recomendado de implementación

1. Definir estructuras y enums base.
2. Construir carga de red y validación de datos.
3. Implementar rutas iniciales.
4. Implementar tick atómico con colas y semáforos.
5. Agregar congestión dinámica y replanificación.
6. Incorporar incidencias y bloqueos.
7. Registrar métricas y eventos.
8. Validar reproducibilidad y escenarios de estrés.

Con esta especificación el motor ya queda definido de forma operativa: qué recibe, cómo avanza, cómo decide, cómo resuelve conflictos, qué mide y cómo evita resultados inconsistentes.