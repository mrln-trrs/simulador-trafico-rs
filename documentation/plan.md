# Plan del simulador de tráfico

## Objetivo
Construir un simulador de tráfico con múltiples caminos, filas de vehículos y reglas de paso en intersecciones, usando una red de nodos y tramos como base del modelo.

## Idea central
El sistema se modelará como un grafo dirigido:

- Los **nodos** representan puntos de decisión o control.
- Los **tramos** representan los caminos entre nodos.
- Los **vehículos** se desplazan por los tramos y esperan cuando existe congestión o prioridad en conflicto.

Esta estructura permite representar calles, cruces, semáforos, rotondas, entradas, salidas y desvíos.

## Elementos del modelo

### 1. Nodo
Cada nodo puede representar:

- Intersección.
- Semáforo.
- Stop o ceda el paso.
- Rotonda.
- Entrada o salida del sistema.
- Punto de fusión o división de carriles.

Propiedades sugeridas:

- Identificador.
- Tipo de nodo.
- Lista de conexiones de entrada y salida.
- Reglas de prioridad.
- Estado de semáforo, si aplica.

### 2. Tramo
Cada tramo conecta dos nodos y debe guardar información como:

- Longitud.
- Número de carriles.
- Capacidad máxima.
- Velocidad máxima.
- Vehículos presentes.
- Tiempo promedio de recorrido.

### 3. Vehículo
Cada vehículo puede tener:

- Origen.
- Destino.
- Ruta actual.
- Velocidad.
- Tiempo acumulado de espera.
- Estado actual: moviéndose, detenido o terminado.

### 4. Simulación
La simulación se encargará de:

- Avanzar el tiempo.
- Mover vehículos por los tramos.
- Resolver conflictos en nodos.
- Aplicar semáforos y prioridades.
- Registrar métricas.

## Funcionamiento general

1. Se define la red de nodos y tramos.
2. Se generan o ingresan vehículos al sistema.
3. Cada vehículo recibe una ruta inicial o dinámica.
4. En cada paso de simulación, los vehículos avanzan según su velocidad y el espacio disponible.
5. Cuando llegan a un nodo, se aplican reglas de prioridad y control.
6. Si un tramo está lleno o un cruce está bloqueado, se forma una fila de espera.
7. El proceso se repite hasta que los vehículos alcanzan su destino.

## Reglas de tránsito

El simulador puede incorporar reglas como:

- Prioridad semafórica por ciclos.
- Prioridad de paso según tipo de vía.
- Respeto de capacidad máxima por tramo.
- Espera en intersecciones cuando hay conflicto.
- Preferencia para transporte público o vehículos de emergencia.

## Estrategia de implementación

### Fase 1: versión mínima viable
- Crear una red simple de nodos y tramos.
- Simular vehículos con rutas fijas.
- Implementar colas básicas en intersecciones.
- Agregar un semáforo simple con ciclos de tiempo.

### Fase 2: mejora del realismo
- Soportar varios carriles por tramo.
- Incluir congestión y bloqueo parcial.
- Recalcular rutas cuando un tramo esté saturado.
- Permitir diferentes tipos de vehículos.

### Fase 3: escenarios avanzados
- Accidentes, obras y cierres temporales.
- Semáforos adaptativos.
- Prioridad por tipo de vehículo.
- Generación de tráfico aleatorio por demanda horaria.

## Métricas útiles

El simulador debería registrar:

- Tiempo total de viaje.
- Tiempo medio de espera.
- Longitud de colas.
- Flujo vehicular por tramo.
- Nivel de congestión.
- Tasa de vehículos completados.

## Estructura conceptual de clases

- `Nodo`
- `Tramo`
- `Vehiculo`
- `Simulacion`
- `ControlSemaforico`
- `GeneradorDeDemanda`
- `CalculadorDeRutas`

## Posibles extensiones

- Visualización del tráfico en tiempo real.
- Exportación de resultados a CSV o gráficos.
- Interfaz para diseñar mapas manualmente.
- Simulación por eventos discretos.
- Modelo por celdas para representar carriles con más detalle.

## Recomendación inicial
Empezar con un modelo sencillo basado en pasos de tiempo, porque permite validar la lógica de nodos, colas y rutas antes de agregar complejidad.

La idea clave es que el simulador no solo represente caminos, sino también decisiones, conflictos y espera. Ahí es donde los nodos y las colas hacen que el modelo sea realmente útil.
