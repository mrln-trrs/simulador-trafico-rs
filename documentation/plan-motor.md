# Plan del motor de simulación de tráfico

Este documento convierte la idea del motor en un plan de implementación por fases. Todo lo que sigue pertenece al backend del simulador: modelo, tick engine, rutas, congestión, eventos, métricas, persistencia, validación y extensibilidad.

La prioridad es consolidar primero el comportamiento interno del sistema. La red vial real será solo una entrada de datos; si el motor es sólido, el resto del simulador se construye sobre esa base sin contradicciones.

## 1. Objetivo

Definir un plan de desarrollo para un motor de simulación de tráfico en Rust que sea determinista, reproducible, validable y escalable.

El motor debe ser capaz de:

- representar una red vial como grafo dirigido,
- cargar escenarios con nodos, tramos, carriles, semáforos y demanda,
- ejecutar la simulación por ticks discretos,
- resolver colas, prioridades y bloqueos,
- recalcular rutas cuando cambien las condiciones,
- registrar eventos y métricas,
- persistir y reproducir escenarios,
- soportar estrés, calibración y extensiones futuras.

## 2. Principios de diseño

- Determinista por defecto: con la misma semilla, la misma red y la misma demanda, los resultados deben repetirse.
- Paso temporal discreto: la simulación avanza por ticks y no por tiempo continuo.
- Actualización atómica por tick: las decisiones se toman sobre una instantánea previa y luego se aplican.
- Separación de responsabilidades: red, rutas, motor, métricas y persistencia deben estar desacoplados.
- Contratos de datos explícitos: cada entidad debe tener estados, límites y validaciones claras.
- Validación temprana: los escenarios inválidos deben rechazarse antes de iniciar la corrida.
- Observabilidad completa: el motor debe poder exponer eventos, métricas y snapshots coherentes.
- Representación contigua para las colas y la ocupación: cuando el acceso sea secuencial, las estructuras de carril y vehículos activos deben favorecer almacenamiento contiguo.
- Grafo por índices o handles estables: el dominio debe apoyarse en IDs, `Vec` centralizados o estructuras equivalentes antes que en punteros compartidos.
- Unidades métricas abstractas: el motor opera en metros lógicos y progresos normalizados; las longitudes curvas y sus aproximaciones deben quedar resueltas al cargar el escenario.
- Núcleo monohilo: el orden de ejecución no debe depender del planificador ni de concurrencia accidental.
- Extensibilidad controlada: futuras capacidades deben entrar mediante contratos estables, no mediante atajos.

## 3. Fases de desarrollo

### Fase 0 - Contratos y modelo base [COMPLETADA]

Objetivo: fijar los cimientos conceptuales y técnicos del motor antes de mover un solo vehículo.

Alcance:

- definir estados del vehículo, tramo, nodo, semáforo, escenario y simulación,
- definir la posición lógica del vehículo como progreso normalizado dentro del tramo,
- fijar el orden exacto del tick,
- preparar un analizador topológico previo a la ejecución para detectar componentes sin salida, ciclos imposibles y reglas de giro incompatibles,
- establecer invariantes y criterios de desempate,
- definir semilla, reproducibilidad y políticas deterministas,
- seleccionar colecciones internas estables para la lógica crítica,
- dejar cerrada la API mínima del backend.

Entregables:

- structs y enums base del dominio,
- contrato de ejecución del tick,
- validador de escenarios y parámetros,
- pruebas unitarias mínimas de consistencia,
- documentación técnica del modelo.

Criterio de cierre:

- el proyecto compila con el modelo base,
- los estados están claramente separados,
- los escenarios inválidos fallan antes de ejecutar,
- el motor puede inicializarse de forma repetible.

### Fase 1 - Red vial y carga de escenarios [COMPLETADA]

Objetivo: construir y validar la red lógica y su relación con los datos de entrada.

Alcance:

- nodos, tramos y carriles,
- geometría interna de la red,
- conexiones dirigidas y bidireccionales,
- restricciones por tramo y por nodo,
- formatos de carga y guardado de escenario,
- validación de conectividad y referencias,
- semilla y parámetros globales del escenario.

Entregables:

- cargador de escenarios,
- serialización y deserialización del estado,
- validación de conectividad y topología,
- ensamblado de la red desde configuración,
- pruebas de carga y restauración.

Criterio de cierre:

- un escenario puede cargarse, validarse y guardarse sin pérdida de información,
- la red reconstruida coincide con la original,
- la geometría y la lógica permanecen consistentes.

### Fase 2 - Motor por ticks y movilidad base [COMPLETADA]

Objetivo: hacer que el sistema avance de forma correcta y determinista.

Alcance:

- fases de lectura y aplicación por tick,
- generación y entrada de vehículos,
- colas FIFO,
- ocupación de carriles,
- movimiento entre nodos y tramos,
- semáforos simples,
- estados de vehículo y de tramo,
- registro de eventos básicos.
- resolución de desplazamientos por barrido o sub-stepping para evitar saltos sobre semáforos, vehículos u ocupación intermedia,
- validación del recorrido continuo del vehículo dentro del tick,
- tratamiento de cambios de estado ocurridos a mitad del avance.

Entregables:

- bucle principal de simulación,
- resolución de colas y prioridades,
- avance y salida de vehículos,
- manejo básico de semáforos,
- eventos de entrada, espera, movimiento y llegada.

Criterio de cierre:

- un vehículo puede recorrer una ruta completa,
- no existe doble ocupación del mismo vehículo,
- el orden por tick se mantiene estable,
- semáforos y colas se respetan correctamente.
- un vehículo no puede atravesar un obstáculo lógico por el mero tamaño del tick,
- los cambios de estado intermedios se detectan dentro del avance,
- el motor resuelve el paso temporal sin depender de la UI ni del renderizado.

### Fase 3 - Congestión, rutas e incidencias [COMPLETADA]

Objetivo: introducir comportamiento dinámico realista en el motor.

Alcance:

- coste dinámico de aristas,
- cálculo de congestión,
- replanificación controlada,
- bloqueos y cierres temporales,
- reducción de capacidad,
- propagación de congestión a rutas vecinas,
- recuperación de espera prolongada,
- tratamiento de incidentes y fallos.

Entregables:

- motor de rutas con costes variables,
- política de replanificación limitada,
- scheduler de incidencias,
- penalizaciones por ocupación, cola, semáforo y bloqueo,
- pruebas de reacción ante cierres y congestión.

Criterio de cierre:

- el motor reacciona a cambios de capacidad,
- las rutas alternativas se eligen de forma determinista,
- no aparecen bucles de reruta sin control,
- los incidentes alteran la simulación de forma medible.

### Fase 4 - Métricas, trazabilidad y persistencia [COMPLETADA]

Objetivo: hacer el motor explicable, exportable y reproducible.

Alcance:

- métricas globales, por nodo y por tramo,
- eventos estructurados por tick,
- trazas de decisión,
- snapshots del estado,
- persistencia de escenarios y resultados,
- exportación en formatos útiles para análisis,
- reproducibilidad exacta con semilla y datos iguales.

Entregables:

- módulo de métricas,
- módulo de eventos y trazabilidad,
- exportadores CSV, JSON o TOML,
- snapshot del estado del motor con progreso normalizado por vehículo,
- carga y guardado determinista de corridas.

Criterio de cierre:

- dos corridas iguales producen los mismos resultados,
- el estado puede guardarse y restaurarse,
- las métricas permiten comparar escenarios,
- los eventos permiten depurar decisiones internas.

### Fase 5 - Validación, estrés y escala [COMPLETADA]

Objetivo: demostrar que el motor sigue siendo correcto cuando crece la complejidad.

Alcance:

- pruebas unitarias, de integración, de propiedad y de estrés,
- escenarios de alta demanda,
- Arachne / redes grandes,
- bloqueos simultáneos,
- rutas con bucles potenciales,
- validación de invariantes,
- ejecución por lotes con distintas semillas.

Entregables:

- suite de validación con pruebas de propiedad sobre invariantes clave,
- casos de estrés reproducibles,
- benchmarks básicos,
- verificación de invariantes,
- regresión de comportamiento estable.

Criterio de cierre:

- el motor soporta carga elevada sin romper reglas,
- las invariantes no se violan,
- el comportamiento sigue siendo determinista,
- la escalabilidad no obliga a reescribir el núcleo.

### Fase 6 - Extensibilidad y endurecimiento [PLANIFICADA]

Objetivo: dejar el motor listo para crecer sin perder orden ni reproducibilidad.

Alcance:

- políticas intercambiables para rutas, prioridad y control,
- calibración con datos reales,
- modelos de demanda más ricos,
- integración con optimización externa,
- control adaptativo,
- mantenimiento de la arquitectura modular,
- endurecimiento frente a casos límite.

Entregables:

- interfaces de políticas sustituibles,
- adaptadores para calibración,
- ganchos para optimización y análisis,
- ampliaciones sin romper el contrato base.

Criterio de cierre:

- nuevas capacidades se pueden añadir sin rediseñar el motor,
- la reproducibilidad sigue intacta,
- la arquitectura permanece legible y estable.

## 4. Cobertura del plan respecto a la idea del motor

- Fase 0 cubre contratos, estados, invariantes y estructura base.
- Fase 1 cubre red vial, geometría interna, carga y guardado de escenarios.
- Fase 2 cubre ciclo de tick, colas, movimiento, semáforos y eventos base.
- Fase 3 cubre congestión, rutas dinámicas, semáforos avanzados e incidencias.
- Fase 4 cubre métricas, trazabilidad, snapshots, persistencia y exportación.
- Fase 5 cubre validación lógica, estrés, rendimiento y comportamiento estable.
- Fase 6 cubre extensibilidad avanzada, calibración y endurecimiento.

## 5. Orden técnico recomendado

1. Definir structs, enums y estados.
2. Cerrar el contrato del tick y la reproducibilidad.
3. Implementar carga y validación de escenarios.
4. Construir el motor mínimo de movilidad.
5. Añadir congestión y replanificación.
6. Incorporar incidencias y bloqueo parcial.
7. Registrar métricas y eventos.
8. Persistir y restaurar escenarios.
9. Validar con pruebas de integración y estrés.
10. Extender con políticas y calibración.

## 6. Criterio de éxito final

El plan quedará completo cuando el motor pueda:

- ejecutar escenarios deterministas,
- resolver colas, prioridades y semáforos,
- reaccionar a congestión e incidentes,
- persistir y restaurar corridas,
- exportar métricas y trazas útiles,
- escalar a redes grandes sin perder estabilidad,
- crecer funcionalmente sin romper el núcleo lógico.

Si estas fases se cumplen, el backend del simulador quedará suficientemente definido para sostener cualquier evolución posterior del proyecto.
