# Arquitectura propuesta del simulador de tráfico

Este documento convierte la visión y los planes del repositorio en una estructura tecnica concreta para implementar el proyecto en Rust sin mezclar responsabilidades. Hoy el repositorio contiene principalmente documentacion, asi que esta propuesta funciona como plano base para cuando empiece el codigo.

## 1. Lectura del proyecto actual

- `idea-simulador` define el sistema completo como una plataforma de simulacion de trafico con backend, frontend y contratos compartidos.
- `idea-motor` exige un motor determinista, por ticks, con modelo estable y sin dependencias de interfaz.
- `idea-visualizador` deja claro que la UI solo representa y controla; no decide la logica del trafico.
- `plan-integracion` pide snapshots, deltas, comandos y versionado de contratos.
- `plan-motor` y `plan-visualizador` ya separan el trabajo por capas, asi que la arquitectura debe respetar esa division y hacerla visible en el arbol de archivos.

## 2. Decision arquitectonica

La mejor opcion para este proyecto es un **monolito modular** con **Clean Architecture / Ports and Adapters** dentro del mismo repositorio. Eso significa:

- un solo punto de arranque para la aplicacion,
- un nucleo de dominio puro y estable,
- un motor de simulacion que concentra la mutacion,
- una capa de contratos compartidos para comunicacion y persistencia,
- una capa de presentacion que consume snapshots y emite comandos,
- una composicion explicita en `main.rs` para conectar todo.

La regla principal es simple: **el modelo no depende de nada, la simulacion depende del modelo, la presentacion depende de contratos y la aplicacion solo ensambla**.

## 3. Flujo de dependencias

La direccion de dependencias recomendada es esta:

| Capa | Puede depender de |
| --- | --- |
| `app` | todas |
| `presentation` | `integration` y, si hace falta, tipos de lectura de `model` |
| `integration` | `model` |
| `simulation` | `model` e `integration` |
| `generation` | `model` e `integration` |
| `model` | ninguna capa de alto nivel |

El flujo en tiempo de ejecucion queda asi:

`presentation -> integration -> app -> simulation -> model`

Y para contenido de escenarios:

`generation -> model -> integration -> simulation`

## 4. Estructura de archivos propuesta

La siguiente estructura es la que mejor encaja con la documentacion actual y con el tipo de proyecto que se quiere construir:

```text
.
├─ Cargo.toml
├─ README.md
├─ documentation/
│  ├─ architecture.md
│  ├─ idea-simulador.md
│  ├─ idea-motor.md
│  ├─ idea-visualizador.md
│  ├─ plan-simulador.md
│  ├─ plan-motor.md
│  ├─ plan-visualizador.md
│  └─ plan-integracion.md
├─ assets/
│  ├─ icons/
│  ├─ maps/
│  └─ fixtures/
├─ src/
│  ├─ main.rs
│  ├─ lib.rs
│  ├─ app/
│  │  ├─ mod.rs
│  │  ├─ bootstrap.rs
│  │  ├─ composition.rs
│  │  └─ runtime.rs
│  ├─ model/
│  │  ├─ mod.rs
│  │  ├─ ids.rs
│  │  ├─ scenario.rs
│  │  ├─ graph.rs
│  │  ├─ road.rs
│  │  ├─ lane.rs
│  │  ├─ vehicle.rs
│  │  ├─ signal.rs
│  │  ├─ state.rs
│  │  └─ invariants.rs
│  ├─ simulation/
│  │  ├─ mod.rs
│  │  ├─ engine.rs
│  │  ├─ tick.rs
│  │  ├─ routing.rs
│  │  ├─ movement.rs
│  │  ├─ conflicts.rs
│  │  ├─ metrics.rs
│  │  ├─ events.rs
│  │  └─ validation.rs
│  ├─ generation/
│  │  ├─ mod.rs
│  │  ├─ builders.rs
│  │  ├─ loaders.rs
│  │  ├─ fixtures.rs
│  │  └─ scenario_factory.rs
│  ├─ integration/
│  │  ├─ mod.rs
│  │  ├─ commands.rs
│  │  ├─ events.rs
│  │  ├─ snapshots.rs
│  │  ├─ delta.rs
│  │  ├─ protocol.rs
│  │  └─ codec.rs
│  ├─ presentation/
│  │  ├─ mod.rs
│  │  ├─ app_shell.rs
│  │  ├─ view_model.rs
│  │  ├─ canvas.rs
│  │  ├─ panels/
│  │  ├─ tools/
│  │  └─ render/
│  └─ persistence/
│     ├─ mod.rs
│     ├─ file_store.rs
│     ├─ serializer.rs
│     └─ migrations.rs
└─ tests/
   ├─ unit/
   ├─ integration/
   ├─ property/
   └─ fixtures/
```

## 5. Responsabilidad de cada capa

### `src/main.rs`

Solo arranca la aplicacion. No debe contener logica de negocio ni de UI. Su trabajo es llamar al bootstrap y terminar ahi.

### `src/lib.rs`

Es la fachada publica del proyecto. Reexporta lo que el resto de la aplicacion necesita y mantiene ocultos los detalles internos.

### `src/app`

Es la raiz de composicion. Aqui se conectan motor, presentacion, persistencia y contratos. Tambien se decide si la aplicacion corre en modo editor, modo simulacion o modo analisis.

### `src/model`

Contiene datos puros y tipos estables. Aqui viven IDs, entidades, estados, relaciones de la red vial e invariantes. No debe haber I/O, renderizado ni logica de ejecucion.

### `src/simulation`

Contiene la parte que muta el estado. Aqui viven el tick engine, el movimiento, el ruteo, las prioridades, la gestion de colas, las metricas y la emision de eventos.

### `src/generation`

Contiene la construccion y carga de escenarios. Sirve para generar casos de prueba, leer archivos externos, validar entrada y preparar datos para el motor.

### `src/integration`

Contiene los contratos compartidos entre capas: comandos, eventos, snapshots, deltas, versionado y codificacion. Es la capa que evita que UI y motor se acoplen por structs internos.

### `src/presentation`

Contiene la interfaz visual. Renderiza, inspecciona y permite editar, pero no calcula rutas ni resuelve colas. La UI recibe snapshots y envia comandos.

### `src/persistence`

Contiene almacenamiento, serializacion y migraciones de formato. Si mas adelante crece mucho, esta capa puede extraerse o subdividirse, pero al inicio conviene mantenerla cerca de los contratos.

## 6. Patrones de diseño recomendados

- **Composition Root**: toda la inyeccion de dependencias vive en `src/app`.
- **Command**: las acciones del usuario y de la UI se modelan como comandos explicitos.
- **Strategy**: rutas, prioridades, costos y politicas de control pueden intercambiarse sin tocar el motor completo.
- **State**: vehiculos, semaforos, tramos y la simulacion general tienen estados bien definidos.
- **Event-driven**: el motor emite eventos y snapshots; la presentacion reacciona a ellos.
- **Builder**: la construccion de escenarios debe ser explicita y facil de testear.
- **Facade**: `lib.rs` expone una interfaz pequena y clara para el resto del proyecto.

## 7. Reglas para no confundirse

- Un modulo, una responsabilidad.
- Ninguna capa de presentacion debe mutar el modelo directamente.
- Ningun tipo de `model` debe depender de la UI.
- Toda mutacion del estado del trafico pasa por `simulation`.
- Toda comunicacion entre UI y motor pasa por `integration`.
- Todo arranque de la aplicacion pasa por `app`.
- Los identificadores deben ser estables y explicitamente tipados.
- Las colecciones internas del motor deben favorecer indices o handles estables, no punteros compartidos como base del diseño.
- El motor debe seguir siendo determinista por defecto.

## 8. Orden de implementacion recomendado

1. Crear `model` con IDs, estados y entidades basicas.
2. Implementar `simulation` con ticks, colas, movimiento y validacion.
3. Definir `integration` con comandos, eventos y snapshots.
4. Montar `app` como punto unico de composicion.
5. Construir `presentation` como consumidora de snapshots y generadora de comandos.
6. Agregar `generation` y `persistence` para escenarios, pruebas y guardado.
7. Cubrir cada modulo con tests unitarios, de integracion y de propiedades.

## 9. Por que esta estructura funciona para este proyecto

- Reduce la confusion porque cada carpeta tiene una sola tarea.
- Escala bien porque el motor no depende de la interfaz.
- Permite trabajar en paralelo en backend, frontend e integracion.
- Facilita las pruebas porque el modelo y la simulacion son aislables.
- Mantiene la documentacion alineada con la implementacion real.
- Da una base limpia para crecer sin reescribir todo cuando el simulador pase de una red pequena a escenarios complejos.

## 10. Criterio final

Si el codigo futuro respeta esta estructura, el proyecto quedara organizado de esta forma: el modelo define la verdad del dominio, la simulacion decide lo que pasa en cada tick, la integracion traduce entre capas y la presentacion solo muestra y controla. Esa separacion es la que mejor encaja con la vision actual del repositorio.