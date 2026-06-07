## Idea del motor de simulación de tráfico

El objetivo principal es definir con precisión cómo debe funcionar el motor por dentro. La red vial real será solo una entrada de datos para alimentar el sistema; primero se debe dejar sólida la lógica de funcionamiento.

Esta es la idea del motor: todo lo que debe contemplar, administrar, calcular, validar, registrar, persistir y reproducir para que la simulación funcione de forma correcta y determinista.

## 1. Objetivo del sistema

Construir un simulador de tráfico en Rust que permita:

- representar una red vial como un grafo dirigido,
- introducir vehículos con rutas y horarios de aparición,
- modelar colas, capacidad de tramos, semáforos y congestión,
- recalcular rutas cuando cambien las condiciones del tráfico,
- medir tiempos de espera, flujo y saturación,
- reproducir escenarios de prueba de forma determinista.

El foco está en la lógica del motor y en el tratamiento de sus datos de entrada y salida, no en ninguna capa externa de consumo.

## 2. Principios de diseño

El simulador debe seguir estas decisiones base:

- **Determinista por defecto:** con la misma semilla y los mismos datos, el resultado debe ser idéntico.
- **Paso temporal discreto:** el sistema avanza por ticks, no por tiempo continuo.
- **Separación de responsabilidades:** red vial, motor de simulación, rutas, métricas y observabilidad deben estar separados.
- **Grafo por índices o handles:** las entidades del grafo deben referenciarse mediante IDs estables, `Vec` centralizados o estructuras tipo `SlotMap`, evitando punteros compartidos y ciclos de préstamo.
- **Unidades métricas abstractas:** el motor calcula distancias, longitudes y progresos en metros lógicos; la UI solo visualiza y no aporta geometría en tiempo de ejecución.
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

La red se modela como un grafo dirigido con nodos y tramos.

- **Nodo:** punto de decisión o control.
- **Tramo:** conexión entre dos nodos.
- **Carril:** subunidad del tramo donde pueden ubicarse vehículos.

La red debe permitir vías de una sola dirección y bidireccionales representadas como dos tramos separados.

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
- posición 2D para referencia geométrica y cálculo de recorridos,
- tipo de nodo,
- lista de tramos entrantes,
- lista de tramos salientes,
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
- prioridad si corresponde.

Tipos sugeridos:

- automóvil,
- bus,
- camión,
- motocicleta,
- ambulancia o emergencia.

### 4.6 Semáforo

El semáforo se tratará como una máquina de estados por fases.

- fase actual,
- duración de cada fase,
- movimientos permitidos,
- tiempo acumulado en la fase,
- modo fijo o adaptativo,
- verde, amarillo o rojo,
- fase todo rojo para limpieza del cruce.

### 4.7 Evento

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
- degradado por incidencia,
- en recuperación.

### 5.3 Estados del nodo

Un nodo puede funcionar como:

- nodo de paso,
- nodo de decisión,
- nodo semaforizado,
- nodo con prioridad,
- nodo de entrada,
- nodo de salida.

### 5.4 Estados globales

La simulación completa debe poder distinguir entre:

- en preparación,
- corriendo,
- en pausa,
- terminada,
- reiniciándose.

## 6. Ciclo de simulación

Cada tick debe seguir un orden fijo para evitar ambigüedades.

### 6.1 Fase de lectura

1. Tomar una instantánea del estado al inicio del tick.
2. Revisar los vehículos que deben entrar.
3. Calcular decisiones de ruta.
4. Detectar conflictos de prioridad.
5. Calcular cambios de fase semafórica.
6. Identificar bloqueos, cierres o cambios de capacidad.

### 6.2 Fase de aplicación

1. Actualizar semáforos.
2. Mover vehículos con permiso.
3. Liberar vehículos que terminan un tramo.
4. Insertar vehículos en sus nuevas colas.
5. Actualizar contadores de espera y ocupación.
6. Registrar eventos y métricas.

### 6.3 Reglas obligatorias

- las decisiones del tick se toman sobre el estado anterior al movimiento,
- un vehículo que entra a una cola en este tick no puede salir de ella hasta el siguiente tick,
- un vehículo que entra a un tramo en este tick no puede abandonarlo en el mismo tick.

## 7. Reglas de movimiento y prioridad

### 7.1 Condiciones de avance

- Un vehículo solo avanza si el tramo siguiente tiene capacidad.
- Si el tramo está lleno, el vehículo espera en el nodo actual.
- Si el semáforo no habilita el movimiento, el vehículo permanece detenido.
- Si el tramo tiene bloqueo o cierre, no puede entrar.
- Si el destino ya fue alcanzado, el vehículo sale del sistema.

### 7.2 Orden de desempate

Si dos vehículos compiten por la misma capacidad, gana el de mayor prioridad o el que llegó antes.

Orden sugerido:

1. vehículo de emergencia,
2. transporte público,
3. vehículo con más tiempo esperando,
4. vehículo que llegó primero a la cola,
5. identificador más bajo como último criterio determinista.

### 7.3 Prevención de bloqueos lógicos

- si un movimiento no cabe, el vehículo permanece en cola,
- si la fase semafórica no autoriza el giro, el vehículo no avanza,
- si el tramo de destino está saturado, el movimiento se pospone,
- si hay varias salidas compatibles, el motor debe decidir de forma reproducible y no aleatoria.

### 7.4 Recuperación de espera prolongada

- mantener un umbral configurable de espera por nodo,
- intentar una reruta de emergencia cuando exista alternativa,
- registrar el caso si el bloqueo persiste,
- evitar que el sistema quede congelado por saturación permanente.

## 8. Congestión y rutas

### 8.1 Variables de congestión

- ocupación del tramo,
- longitud de la cola de entrada,
- tiempo promedio de espera,
- bloqueos en nodos adyacentes,
- saturación por carril,
- historial reciente de flujo.

### 8.2 Costo dinámico

El costo de una arista debe combinar tiempo base y penalizaciones por ocupación, espera, bloqueo e influencia semafórica.

Forma objetivo:

`costo = tiempo_base + penalización_por_ocupación + penalización_por_espera + penalización_por_incidente`

El costo debe:

- crecer cuando el tramo se acerque a la saturación,
- subir si la cola aumenta,
- reaccionar a bloqueos o cierres,
- considerar el tiempo útil del semáforo,
- evitar oscilaciones absurdas con suavizado temporal y límites máximos.

### 8.3 Replanificación

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

### 8.4 Política de rutas

La política de rutas debe ser estable y reproducible, aunque cambien los pesos dinámicos.

- el motor debe poder usar pesos por tiempo, no solo por distancia,
- los empates deben resolverse con criterio determinista,
- la ruta elegida debe quedar registrada,
- el cálculo debe poder repetirse con la misma semilla.

### 8.5 Resolución de paso de tiempo

Como la simulación avanza por ticks discretos, el motor debe evitar el efecto de salto entre posiciones.

- si un vehículo avanza varios metros en un tick, su desplazamiento debe evaluarse por barrido o por sub-steps internos,
- cualquier semáforo, bloqueo o vehículo ocupado en el intervalo debe poder impedir el avance,
- la posición final no puede ignorar un cambio de estado ocurrido dentro del mismo tramo temporal,
- la lógica de colisión o bloqueo debe resolverse en el backend, nunca en la UI.

## 9. Semáforos e intersecciones

### 9.1 Máquina de estados

El semáforo debe comportarse como una máquina de estados con:

- fase actual,
- duración de cada fase,
- movimientos permitidos,
- tiempo acumulado en la fase,
- modo fijo o adaptativo,
- verde, amarillo o rojo,
- fase todo rojo para limpiar la intersección.

### 9.2 Movimientos permitidos

Un nodo puede permitir uno o varios movimientos por fase. Una fase puede desbloquear una sola arista o varias compatibles. El amarillo debe funcionar como transición, no como permiso libre total.

### 9.3 Operación por intersección

La lógica de intersección debe soportar:

- cruces simples,
- cruces con prioridad,
- rotondas,
- uniones y bifurcaciones,
- intersecciones con varias fases semafóricas.

### 9.4 Control semafórico

- debe haber modos fijo y adaptativo,
- las fases deben poder configurarse por nodo,
- el estado semafórico debe quedar registrado por tick,
- la lógica del semáforo no debe depender de la capa externa.

## 10. Incidentes y fallos

### 10.1 Incidentes posibles

- cierre total de un tramo,
- reducción de carriles,
- semáforo desincronizado,
- demora excesiva en una fase,
- aumento temporal de demanda,
- accidente con bloqueo parcial,
- caída de capacidad por mantenimiento,
- ruta desviada por restricción.

### 10.2 Efecto esperado

- formación de cola,
- reasignación de rutas,
- propagación de congestión,
- aumento del tiempo medio de viaje,
- posible estabilización tras varios ticks.

### 10.3 Tratamiento interno

- cada incidente debe tener inicio, duración y resolución,
- el motor debe registrar el impacto sobre capacidad y tiempos,
- los vehículos afectados deben poder rerutarse,
- el escenario debe seguir siendo reproducible.

## 11. Demanda y escenarios

### 11.1 Formas de entrada

- lista fija de vehículos y horarios,
- generador aleatorio con semilla,
- perfiles por hora pico,
- escenarios de carga progresiva,
- orígenes y destinos ponderados.

### 11.2 Contenido de cada escenario

Cada escenario debería incluir:

- semilla aleatoria,
- duración total,
- lista de nodos,
- lista de tramos,
- lista de semáforos,
- calendario de demanda,
- incidentes programados,
- parámetros globales del motor.

### 11.3 Validación de entrada

- los identificadores deben ser únicos,
- todo tramo debe conectar nodos existentes,
- toda ruta predefinida debe ser válida,
- toda fase semafórica debe referirse a movimientos reales,
- todos los valores numéricos deben estar en rangos positivos o válidos.

## 12. Arquitectura interna en Rust

### 12.1 Módulos principales

- `generation`: calendarios de vehículos,
- `model`: nodos, tramos, vehículos, semáforos, red y rutas,
- `simulation`: orquestación del motor,
- `simulation/runtime.rs`: runtime de tramos y semáforos,
- `simulation/spawn.rs`: creación y calendario de vehículos,
- `simulation/queues.rs`: resolución de colas y prioridades,
- `simulation/movement.rs`: avance por tramos y entrada/salida,
- `simulation/routing.rs`: cálculo de rutas y pesos dinámicos,
- `simulation/signals.rs`: actualización de semáforos,
- `simulation/timing.rs`: contadores de tiempo,
- `simulation/metrics.rs`: acumulación y exportación de resultados,
- `simulation/events.rs`: registro de eventos,
- `scenario`: escenarios de prueba,
- `main`: arranque mínimo de la demo.

### 12.2 Estructuras recomendadas

- `HashMap` para acceso por identificador,
- `VecDeque` para colas FIFO,
- `BinaryHeap` para búsqueda de rutas y candidatos,
- `Vec` para listas ordenadas de fases, eventos o recorridos,
- `BTreeMap` cuando se requiera orden determinista por tiempo,
- `Option` y `Result` para estados incompletos y fallos controlados.

### 12.3 Responsabilidades internas

- el modelo debe contener solo datos y validaciones simples,
- la simulación debe concentrar la lógica de ticks,
- los eventos y métricas deben ser acumulados por módulos dedicados,
- la exportación y la carga de escenarios deben ser independientes del motor de decisión.

## 13. Base técnica de ejecución

### 13.1 Reglas de ejecución

- el núcleo de simulación debe ser monohilo para mantener determinismo,
- las tareas auxiliares no deben alterar el orden del cálculo,
- ningún estado lógico debe depender del orden accidental de `HashMap`,
- las estructuras internas que afecten al resultado deben recorrer su contenido de forma estable.

### 13.2 Criterios de estabilidad

- usar `BTreeMap` cuando el orden de evaluación importe,
- no depender de azar para resolver conflictos internos,
- mantener identificadores estables para nodos, tramos y vehículos,
- registrar errores de validación sin corromper el estado de la corrida.

### 13.3 Contrato de reproducibilidad

- misma semilla,
- misma red,
- misma demanda,
- mismos parámetros de congestión,
- mismo orden de actualización por tick,
- mismas políticas de desempate.

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
- CSV para demanda y resultados,
- formatos binarios o compactos si se busca rendimiento.

### Validación de carga

- los identificadores deben ser únicos,
- toda referencia debe apuntar a una entidad existente,
- los valores numéricos deben estar dentro de rangos válidos,
- los datos incompletos deben rechazarse antes de iniciar la corrida.

## 15. Métricas que debe producir

Las métricas deben servir tanto para depurar como para comparar escenarios.

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

El motor debe poder registrar lo que hace sin depender de una capa externa.

### Formas de registro

- log de eventos por consola o canal de depuración,
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

## 18. Persistencia y exportación

El motor debe poder guardar y recuperar escenarios sin perder consistencia.

### Datos que deben persistir

- geometría de la red,
- tipos de nodo y tramo,
- parametrización de carriles,
- matrices de giro,
- ciclos semafóricos,
- eventos temporales,
- perfiles de demanda,
- semilla aleatoria,
- parámetros globales,
- resultados de ejecución si se decide almacenarlos.

### Formatos posibles

- TOML,
- JSON,
- CSV para demanda y resultados,
- formatos binarios o compactos si se busca rendimiento.

### Exportación del motor

- eventos por tick,
- métricas globales,
- métricas por nodo,
- métricas por tramo,
- historial de vehículos,
- trazas de replanificación,
- resumen de congestión,
- estados semafóricos acumulados.

## 19. Ruta de desarrollo recomendada

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

### Fase 4: exportación, observabilidad y endurecimiento

- exportación de resultados,
- comparación entre escenarios,
- trazas de depuración,
- validación de consistencia,
- endurecimiento del motor frente a casos límite.

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

## 22. Salidas y observabilidad del motor

El motor debe poder entregar salidas coherentes para depuración, análisis y consumo por otros componentes del sistema, sin depender de ninguna capa externa.

### 22.1 Instantáneas del estado

El motor debe producir instantáneas coherentes del sistema para consultar el estado en cualquier tick.

Cada instantánea debería incluir:

- red vial completa,
- estado de nodos, tramos y carriles,
- estado de vehículos,
- estados semafóricos,
- colas activas,
- incidentes activos,
- eventos recientes,
- métricas acumuladas,
- tick actual y contadores auxiliares.

### 22.2 Canales de observabilidad

El motor debe poder emitir:

- eventos estructurados por tick,
- trazas de decisión,
- alertas de consistencia,
- resúmenes de congestión,
- estados intermedios de replanificación,
- diagnósticos de bloqueo o saturación.

### 22.3 Uso de la observabilidad

Estas salidas sirven para:

- depurar el comportamiento interno,
- comparar escenarios,
- validar invariantes,
- alimentar reportes o análisis externos,
- conservar historial reproducible.

## 23. Geometría y datos espaciales internos

El motor debe manejar la geometría del escenario como parte de su propio modelo, sin pensar en una capa de consumo externa.

### 23.1 Sistemas de coordenadas

El sistema debe poder manejar al menos estos formatos espaciales:

- coordenadas del mundo del escenario,
- coordenadas métricas,
- coordenadas geográficas si se usan mapas reales.

La traducción entre estos sistemas debe ser estable y, cuando aplique, reversible dentro del error tolerable de la proyección usada.

### 23.2 Geometría de tramos

Un tramo lógico puede representarse con:

- una línea recta,
- una polilínea,
- una curva Bézier,
- una spline,
- una combinación de segmentos con puntos de control.

La longitud física usada por la simulación debe calcularse a partir de esa geometría, no al revés.

### 23.3 Offsets y separación geométrica

El motor debe admitir que un mismo eje lógico tenga desplazamientos distintos según:

- el sentido de circulación,
- la cantidad de carriles,
- la existencia de mediana,
- la necesidad de representar una separación física entre aristas.

### 23.4 Cruces y pasos a desnivel

Cuando dos elementos se crucen geométricamente, el motor debe distinguir entre:

- cruce lógico real,
- cruce geométrico sin conexión,
- intersección confirmada como nodal,
- paso a desnivel sin interacción.

### 23.5 Alineación geográfica

Si el simulador usa mapas reales, el motor debe poder recibir una referencia geográfica para alinear la red con el fondo. Esa referencia no debe alterar la lógica de cálculo, solo la interpretación espacial de la red.

## 24. Reproducibilidad y ejecución masiva

El motor debe poder guardarse, volver a cargarse y repetirse sin perder consistencia. Esto es clave para comparar escenarios, depurar errores y validar decisiones.

### 24.1 Escenario versionado

Cada escenario debe guardarse con una estructura versionada que incluya:

- red completa,
- geometría,
- demanda,
- semáforos,
- eventos temporales,
- semilla aleatoria,
- parámetros globales,
- configuración de congestión,
- metadatos de ejecución si se desea.

### 24.2 Reproducción determinista

Con la misma semilla, la misma red y la misma demanda, el motor debe repetir exactamente la misma corrida. Si el resultado cambia, la causa debe ser observable y justificable.

### 24.3 Exportación de resultados

El motor debe poder exportar:

- eventos por tick,
- métricas globales,
- métricas por nodo,
- métricas por tramo,
- historial de vehículos,
- trazas de replanificación,
- resumen de congestión,
- estados semafóricos acumulados.

### 24.4 Ejecución por lotes

Debe ser posible correr múltiples escenarios o múltiples semillas sobre el mismo escenario para:

- comparar políticas de control,
- medir sensibilidad,
- obtener promedios estadísticos,
- detectar casos límite,
- validar estabilidad del motor.

## 25. Extensibilidad avanzada y capacidades extra

La idea del motor no debe quedar limitada al conjunto mínimo del simulador actual. Debe poder crecer sin romper su núcleo lógico.

### 25.1 Control adaptativo

El motor debe estar preparado para semáforos adaptativos, prioridades dinámicas y reglas que cambien según demanda, hora o congestión.

### 25.2 Modelos de demanda más ricos

Más adelante el sistema debería poder soportar:

- matrices origen-destino,
- perfiles horarios complejos,
- composición variable del parque automotor,
- demandas estocásticas con distribuciones configurables,
- escenarios de horas pico y valle.

### 25.3 Calibración con datos reales

Si en el futuro se dispone de observaciones reales, el motor debería aceptar parámetros calibrables para:

- velocidades observadas,
- tiempos de espera medidos,
- flujos horarios,
- saturación por tramo,
- tiempos de fase semafórica.

### 25.4 Reglas y políticas intercambiables

Conviene que la lógica pueda evolucionar hacia un sistema donde ciertas decisiones sean sustituibles:

- política de ruta,
- política de prioridad,
- política de bloqueo,
- política de reruta,
- política de control semafórico.

### 25.5 Integración con análisis y optimización

El motor debería poder alimentar procesos externos de:

- optimización de tiempos de viaje,
- búsqueda de configuraciones de semáforos,
- comparación de diseños de red,
- análisis de cuellos de botella,
- evaluación de escenarios extremos.

### 25.6 Modo de simulación avanzada

Más adelante el motor podría soportar, sin cambiar su arquitectura base:

- simulaciones por múltiples réplicas,
- escenarios de sensibilidad,
- estrés con cierres simultáneos,
- políticas de emergencia,
- exportación a herramientas externas de análisis.

## 26. Criterio ampliado de éxito

La idea del motor quedará realmente madura cuando no solo resuelva tráfico, sino que también sirva como plataforma estable para diseñar, probar y comparar redes viales completas.

El motor debe permitir:

- una lógica sólida y determinista,
- observabilidad completa,
- una geometría fiel a la red real,
- persistencia reproducible,
- métricas útiles,
- extensión futura sin reescritura total,
- soporte para escenarios simples y complejos por igual.

Si este plan se cumple, el motor no será únicamente una implementación básica, sino una base seria para experimentación vial, análisis de congestión, diseño de escenarios y evolución hacia capacidades más avanzadas.

## 27. Estado de la implementación del motor

El motor de simulación de tráfico está completamente implementado bajo los principios y especificaciones descritas en este documento:
- **Núcleo lógico determinista**: Desarrollado en [src/simulation/engine.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/simulation/engine.rs). Avanza por ticks y emite snapshots/deltas estructurados en cada tick de forma determinista y monohilo.
- **Ruteo dinámico por Dijkstra**: Ubicado en [src/simulation/routing.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/simulation/routing.rs). Evalúa los costos de viaje en base a congestión física en tiempo real y permite replanificar rutas a vehículos atascados tras 3 ticks de espera.
- **Validaciones e Invariantes**: Implementadas en [src/model/invariants.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/model/invariants.rs) y [src/simulation/validation.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/simulation/validation.rs), asegurando la validez del escenario antes de la corrida.
- **Pruebas y Verificación**: Validadas a través de suites de pruebas automáticas en [tests/smoke.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/tests/smoke.rs), comprobando viajes exitosos, congestión de vehículos y roundtrip de serialización en archivos.


