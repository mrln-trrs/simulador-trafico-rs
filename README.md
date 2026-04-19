# Simulador de Tráfico 🚗

Simulador modular de tráfico urbano en Rust con arquitectura limpia y frontend moderno.

## ✨ Características

### Backend
- Modelo de dominio tipado y estable
- Motor determinista por ticks
- Contratos compartidos (snapshots, comandos)
- Persistencia JSON/TOML
- Simulación realista de flujo vehicular

### Frontend - Traffix Pro / Interfaz Fluent Design
- **Diseño moderno** inspirado en Fluent Design de Windows
- **Canvas isométrico central** con red vial, carriles, pasos peatonales y capa de calor
- **Panel izquierdo** para trazado, señalización, escenarios y tipos de vehículos
- **Panel derecho** para propiedades contextuales, visualización y KPI
- **Barra superior e inferior** para control temporal, línea de tiempo y análisis
- **Scheduler determinista**: Los ticks avanzan por tiempo real con pasos fijos
- **Interpolación visual**: El render suaviza el movimiento entre pasos discretos
- **Control de velocidad**: 0.25x a 4.0x
- **Métricas en vivo**: Visualización de estadísticas en tiempo real
- **Grid y snap**: Herramientas de edición precisas
- **Tema oscuro/claro**: Soporte completo

## 🚀 Inicio Rápido

### Ejecutar
```bash
cargo run --release
```

### Compilación Debug (más rápida)
```bash
cargo run
```

### Pruebas
```bash
cargo test
```

## 🎮 Cómo Usar

### Observar Simulación
1. Ejecuta `cargo run`
2. La interfaz muestra un escenario de demostración
3. Haz clic en **Play** para iniciar
4. Ajusta **Speed** y **Ticks/Segundo** para controlar la tasa de avance
5. Observa los vehículos moviéndose en la red con paso determinista
6. Usa la barra superior para reproducir, pausar, avanzar o rebobinar, y la franja inferior para seguir eventos y métricas

### Panel Izquierdo - Herramientas de Diseño de Red
- **🔍 Seleccionar**: Selecciona elementos
- **● Crear Nodo**: Agrega puntos de intersección
- **⟶ Crear Tramo**: Conecta nodos
- **✤ Mover**: Reposiciona elementos
- **⌫ Eliminar**: Quita elementos
- **Snap a grid**: Alineación precisa
- **Velocidad**: Controla la rapidez de simulación

### Panel Central - Canvas de Simulación
- **Zoom**: Rueda del mouse
- **Pan**: Botón derecho + arrastrar
- **Seleccionar**: Clic izquierdo en elemento
- **Grid**: Guía de referencia espacial
- **Mapa de calor**: Superposición para detectar congestión y puntos críticos

### Panel Derecho - Propiedades y Visualización
- Estado actual de la simulación
- Estadísticas: Ticks, vehículos, viaje promedio
- Lista de vehículos activos con progreso
- Ajustes visuales y KPIs en tiempo real

## 🖥️ Traffix Pro

La interfaz está pensada como una estación de control completa para diseño, observación y análisis.

### 1. Canvas de Simulación Isométrica

El panel central prioriza una lectura clara del escenario con perspectiva isométrica, capas visuales y superposiciones térmicas para congestión.

### 2. Herramientas de Diseño de Red

La barra lateral izquierda agrupa edición de trazado, creación de intersecciones complejas, señalización, carga de escenarios y definición de tipos de vehículos.

### 3. Panel de Propiedades y Visualización

La barra lateral derecha concentra propiedades contextuales, modo de vista, opacidad del mapa de calor, iluminación y KPIs operativos.

### 4. Controles de Simulación y Análisis

La parte superior e inferior de la UI se reserva para reproducción, control temporal, línea de tiempo, gráficos comparativos y seguimiento de eventos.

## 🎨 Tema Fluent Design

### Colores
| Color | Uso |
|-------|-----|
| 🔵 Azul (#0078D4) | Elementos primarios, selección |
| 🟢 Verde (#107C10) | Vehículos, estado positivo |
| 🟡 Amarillo (#FFB800) | Advertencias, congestión |
| 🔴 Rojo (#FF0000) | Errores, bloqueos |

### Espaciado
- **Pequeño**: 4px - 8px
- **Normal**: 12px
- **Grande**: 16px - 24px
- **XL**: 32px

## 📁 Estructura del Código

```
src/
├── app/              # Composición e inicio
├── model/            # Dominio (tipado, puro)
├── simulation/       # Motor de simulación
├── generation/       # Creación de escenarios
├── integration/      # Contratos compartidos
├── presentation/     # Interfaz visual
│   ├── theme.rs      # Sistema Fluent Design
│   ├── components.rs # Widgets reutilizables
│   ├── controls.rs   # Controles de simulación
│   ├── canvas.rs     # Renderizado 2D
│   └── app_shell.rs  # Layout principal
└── persistence/      # Guardado/carga
```

## 🔧 Tecnologías

- **Rust 2021 Edition**
- **egui 0.27** - Framework GUI inmediato
- **eframe 0.27** - Backend multiplataforma
- **Serde** - Serialización
- **TOML** - Configuración

## 💡 Características Especiales

### Scheduler Determinista por Tiempo Real ⭐
```
RELOJ REAL DE LA PC:
El bucle mide el tiempo transcurrido con `Instant`
→ Un acumulador decide cuántos pasos consumir

PASO FIJO:
Cada tick representa un paso discreto del motor
→ La tasa visual depende de `Ticks/Segundo` y `Speed`

RENDER SUAVE:
La UI interpola entre snapshots consecutivos
→ La simulación sigue siendo discreta, pero se ve continua
```

Útil para que la simulación sea reproducible y no dependa del FPS ni del mouse.

### Interfaz Responsiva
- Paneles redimensionables
- Canvas que usa espacio disponible
- Elementos escalables con zoom

## 🧪 Desarrollo

### Compilar en Debug (rápido)
```bash
cargo build
```

### Compilar Optimizado (lento pero rápido en runtime)
```bash
cargo build --release
```

### Ejecutar Tests
```bash
cargo test
```

### Format
```bash
cargo fmt
```

### Lint
```bash
cargo clippy
```

## 📊 Próximas Mejoras

- ✅ Interfaz Fluent Design moderna
- ✅ Control por movimiento del mouse
- ⏳ Herramientas de edición interactivas
- ⏳ Análisis y mapas de calor
- ⏳ Exportación de reportes
- ⏳ Modo depuración avanzado
- ⏳ Compatibilidad con escenarios OpenStreetMap

## 📝 Notas de Desarrollo

### Filosofía de Código
- **Claridad**: El código es el mejor comentario
- **Modularidad**: Cada capa una responsabilidad
- **Testabilidad**: El modelo es independiente
- **Escalabilidad**: Preparado para crecer

### Principios Fluent
- Minimalismo
- Profundidad visual sutil
- Espaciado generoso
- Tipografía clara
- Accesibilidad

## 📄 Licencia

MIT

## 👨‍💻 Autor

Proyecto educativo de simulación de tráfico urbano en Rust.

---

**Versión**: 1.0.0 - Fluent Design Edition
**Estado**: En desarrollo activo ✨
