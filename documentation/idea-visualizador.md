# Idea del visualizador del simulador de tráfico

Este documento define exclusivamente el componente visual y de interacción del simulador. Todo lo relativo a rutas, semáforos, colas, congestión, decisiones de tráfico y estados internos pertenece al backend y a su documentación específica.

El visualizador es la capa que el usuario ve y manipula. Su función es mostrar la red, editar el escenario, reproducir la simulación, inspeccionar entidades y presentar información de forma clara y fluida.

## 1. Propósito

Construir una interfaz visual de escritorio en Rust que permita:

- dibujar y editar redes viales con precisión,
- mostrar el estado de la simulación en tiempo real,
- visualizar vehículos, colas, rutas y congestión,
- inspeccionar nodos, tramos y vehículos,
- reproducir escenarios por ticks,
- activar capas de depuración y análisis,
- servir como panel de diseño y como panel de observación.

## 2. Qué sí hace el visualizador

El visualizador se encarga de representar y comunicar.

Debe:

- renderizar el mapa base y las capas del escenario,
- dibujar nodos, tramos, carriles y marcas,
- mostrar vehículos en movimiento,
- resaltar rutas planificadas,
- pintar la congestión con colores o gradientes,
- mostrar colas, bloqueos, cierres y zonas de velocidad,
- abrir paneles de propiedades y paneles de estado,
- enviar comandos de edición al backend,
- recibir snapshots, eventos y métricas para dibujar el estado actual.

## 3. Qué no hace el visualizador

Para mantener la separación de responsabilidades, el visualizador no debe:

- calcular rutas,
- decidir prioridades de paso,
- simular movimiento de vehículos,
- resolver colas,
- cambiar semáforos por sí mismo,
- recalcular congestión lógica,
- modificar el modelo interno sin pasar por el backend.

Toda la lógica de tráfico vive fuera de esta capa.

## 4. Modos de uso

### 4.1 Modo diseño

En este modo el usuario construye el escenario.

Debe permitir:

- crear nodos,
- trazar tramos rectos y curvos,
- mover y alinear elementos,
- dividir o unir vías visualmente,
- añadir marcas, etiquetas y restricciones,
- configurar propiedades desde paneles laterales,
- validar visualmente cruces e intersecciones,
- guardar el escenario.

### 4.2 Modo ejecución

En este modo la simulación ya está corriendo.

Debe permitir:

- reproducir, pausar y avanzar,
- seguir la evolución tick por tick,
- ver vehículos desplazarse,
- ver cambios de fase semafórica,
- observar colas y saturación,
- detectar incidentes y bloqueos,
- inspeccionar el estado de cualquier elemento.

### 4.3 Modo depuración

En este modo el sistema muestra información técnica de apoyo.

Debe permitir:

- activar contornos y guías,
- mostrar identificadores,
- ver vectores de dirección,
- mostrar límites de carril,
- resaltar rutas alternativas,
- visualizar ocupación interna,
- mostrar alertas y eventos recientes.

### 4.4 Modo análisis

Este modo sirve para comparar y estudiar resultados.

Debe permitir:

- ver mapas de calor,
- revisar métricas por tramo y por nodo,
- comparar corridas,
- inspeccionar estadísticas acumuladas,
- revisar la evolución temporal de una entidad,
- exportar capturas o reportes visuales.

## 5. Espacio visual principal

### 5.1 Lienzo 2D

El centro de la interfaz debe ser un lienzo 2D navegable.

Debe soportar:

- desplazamiento,
- zoom,
- ajuste a pantalla,
- encaje por rejilla o guía,
- selección por clic,
- selección múltiple,
- arrastre con mouse,
- navegación fluida en escenarios grandes.

### 5.2 Sistema de coordenadas

La interfaz debe traducir entre:

- coordenadas de pantalla,
- coordenadas del mundo del escenario,
- coordenadas métricas o geográficas si el mapa lo requiere.

La conversión debe ser estable para que el dibujo no dependa del tamaño de la ventana o del nivel de zoom.

### 5.3 Capas visuales

El renderizado debe dividirse en capas separadas para evitar confusión:

1. Capa base o mapa de referencia.
2. Capa de geometría vial.
3. Capa de marcas, carriles y restricciones.
4. Capa de control de intersecciones.
5. Capa dinámica de vehículos.
6. Capa termográfica o analítica.
7. Capa de selección, guías y depuración.

## 6. Elementos que se muestran

### 6.1 Nodos

Los nodos deben visualizarse con símbolos claros y consistentes.

Deben mostrar:

- tipo de nodo,
- estado actual,
- conexión con entradas y salidas,
- semáforo o prioridad si aplica,
- información de cola o espera cuando se inspeccionen.

### 6.2 Tramos

Los tramos deben visualizarse según su geometría y sus atributos.

Deben mostrar:

- dirección de circulación,
- número de carriles,
- ancho aparente,
- estado operativo,
- cierre o bloqueo temporal,
- saturación o nivel de uso.

### 6.3 Carriles

Los carriles deben poder representarse cuando el nivel de detalle lo requiera.

Deben permitir ver:

- carriles exclusivos,
- carriles bloqueados,
- carriles de giro,
- carriles de acceso o salida,
- cambios temporales de capacidad.

### 6.4 Vehículos

Los vehículos deben verse como entidades móviles orientadas según la dirección del tramo.

Deben mostrar:

- tipo de vehículo,
- orientación,
- velocidad visual,
- posición interpolada,
- ruta seguida o planificada,
- estado resaltado si están detenidos, bloqueados o llegando a destino.

### 6.5 Rutas y trayectorias

La ruta de un vehículo debe poder resaltarse visualmente.

Debe permitir:

- mostrar el camino completo,
- diferenciar tramo actual y tramos futuros,
- mostrar cambios de ruta,
- visualizar rutas alternativas si se activan,
- ocultar o atenuar rutas cuando no se necesiten.

### 6.6 Congestión y eventos

La interfaz debe poder pintar el estado de la red con colores y avisos.

Debe mostrar:

- flujo libre,
- carga media,
- saturación,
- bloqueo,
- cierre temporal,
- incidentes,
- zonas con reducción de velocidad,
- eventos activos o recientes.

## 7. Interacción del usuario

### 7.1 Edición directa

El usuario debe poder manipular elementos visualmente.

Debe poder:

- crear,
- mover,
- redimensionar,
- rotar,
- dividir,
- unir,
- duplicar,
- eliminar,
- agrupar,
- reordenar capas si aplica.

### 7.2 Paneles de propiedades

Al seleccionar un elemento, debe abrirse un panel lateral o flotante.

Ese panel debe mostrar y permitir editar:

- nombre o etiqueta,
- tipo,
- posición,
- geometría,
- colores,
- estado visual,
- parámetros permitidos por el modelo,
- información de contexto útil para el usuario.

### 7.3 Menús contextuales

El clic derecho o un menú contextual debe ofrecer acciones rápidas como:

- inspeccionar,
- centrar vista,
- aislar elemento,
- ocultar o mostrar,
- duplicar,
- borrar,
- activar guías,
- abrir configuración visual.

### 7.4 Deshacer y rehacer

La interfaz debe soportar historial de edición para que el usuario pueda corregir errores sin perder trabajo.

## 8. Controles de reproducción

La experiencia visual de simulación necesita una barra de tiempo clara.

Debe incluir:

- reproducir,
- pausar,
- detener,
- avanzar un tick,
- retroceder si el sistema lo soporta,
- acelerar,
- desacelerar,
- saltar a un instante concreto si se implementa.

La línea temporal debe ayudar a entender qué ocurrió y cuándo ocurrió.

## 9. Inspector visual

El visualizador debe incluir un inspector para ver el estado de la simulación sin abrir herramientas externas.

### 9.1 Inspector de nodo

Debe mostrar:

- tipo de nodo,
- conexiones,
- estado visual,
- semáforo o prioridad visible,
- cola o espera observada,
- eventos recientes asociados.

### 9.2 Inspector de vehículo

Debe mostrar:

- tipo de vehículo,
- ruta resaltada,
- tramo actual,
- velocidad visual,
- tiempo transcurrido,
- estado actual,
- cambios de ruta observables.

### 9.3 Inspector de tramo

Debe mostrar:

- longitud visual,
- carriles visibles,
- ocupación aparente,
- estado de bloqueo,
- nivel de saturación,
- mensajes o alertas asociados.

## 10. Comunicación con el backend

El visualizador debe consumir datos del backend mediante un contrato claro.

### 10.1 Lo que recibe

Debe recibir:

- snapshots del escenario,
- eventos por tick,
- métricas resumidas,
- cambios de estado,
- confirmaciones de edición,
- resultados de validación.

### 10.2 Lo que envía

Debe enviar:

- comandos de edición,
- solicitudes de reproducción,
- selección de elementos,
- cambios de vista,
- filtros visuales,
- opciones de análisis,
- exportaciones o capturas si se habilitan.

### 10.3 Regla de separación

El visualizador nunca debe modificar la simulación de forma directa. Solo puede emitir comandos; el backend decide si son válidos y cómo se aplican.

## 11. Calidad visual

La interfaz debe ser útil y legible, no solo funcional.

Debe cuidar:

- contraste,
- legibilidad de etiquetas,
- colores consistentes,
- iconografía clara,
- jerarquía visual,
- reducción de ruido en mapas complejos,
- rendimiento estable al mover muchos objetos.

## 12. Rendimiento y escalabilidad visual

El componente visual debe estar preparado para redes grandes y movimiento continuo.

Debe contemplar:

- renderizado eficiente,
- actualización parcial de capas,
- interpolación de movimiento entre ticks,
- ocultación de detalles cuando el zoom sea bajo,
- agrupación visual cuando haya demasiados elementos,
- respuesta fluida al arrastrar y seleccionar.

## 13. Criterio de éxito

El visualizador estará bien definido cuando pueda:

- mostrar la red con claridad,
- editar escenarios sin confusión,
- seguir la simulación en tiempo real,
- inspeccionar cualquier elemento sin romper el flujo,
- visualizar congestión y eventos de forma inmediata,
- trabajar como frontend puro sin contener lógica de tráfico.

## 14. Estado de la implementación de la UI

La interfaz gráfica del simulador se encuentra actualmente estructurada bajo [src/ui](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/ui), construida en base a `egui` y `eframe`:
- **Lienzo e Infinito Grid**: Implementados en [src/ui/screens/simulator/canvas](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/ui/screens/simulator/canvas), permitiendo navegación (zoom y pan) y cacheo de líneas de rejilla.
- **Herramienta de Carreteras (`RoadTool`)**: Desarrollada en [src/ui/screens/simulator/tools/road_tool.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/ui/screens/simulator/tools/road_tool.rs). Permite trazar vías rectas, configurar el número de carriles de forma dinámica y realizar un snapping magnético preciso al lienzo.
- **Herramienta de Edificios (`BuildingTool`)**: Desarrollada en [src/ui/screens/simulator/tools/building_tool.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/ui/screens/simulator/tools/building_tool.rs). Permite dibujar polígonos cerrados de obstáculos e implementa triangulación de orejas para renderizar polígonos complejos sin problemas.
- **Detección de Colisiones**: Integrada geométricamente en [src/ui/screens/simulator/geom/collisions.rs](file:///c:/TRABAJOS%20-%202026/Optimizacion%20y%20Simulacion/LRPD/src/ui/screens/simulator/geom/collisions.rs), impidiendo que se tracen carreteras sobre edificios.
- **Herramientas de Borrado (`DeleteTool`)**: Soporta eliminación selectiva por sub-polígono, lazo interactivo de selección o borrado del elemento completo.

*Nota de progreso*: Los modos de ejecución de la simulación (Play, Pausa, Reset, avance tick por tick) están diseñados pero aún no se han acoplado a la estructura interactiva de `SimuladorApp` para procesar y renderizar los snapshots en tiempo real.

En resumen, este documento cubre solo la experiencia visual e interactiva del simulador. Si una decisión afecta al dibujo, la interacción o la presentación, pertenece aquí. Si afecta al comportamiento del tráfico, pertenece al backend.
