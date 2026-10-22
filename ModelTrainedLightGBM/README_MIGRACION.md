# 🛡️ Tigo NDR Suite - Paquete de Migración

Este paquete contiene **todo el código fuente**, **los modelos de Machine Learning ya entrenados**, **activos de interfaz**, **reportes XAI** y **configuraciones de compilación** necesarios para ejecutar o compilar la plataforma en cualquier otro entorno (Windows, Linux, Docker o la nube).

---

## 📦 Contenido del Paquete

### 1. core_engine/ (Motor Principal en Rust)
* **src/main.rs**: Código fuente del servidor NDR, motor de inferencia con LightGBM, WebSocket en tiempo real y simulador de tráfico resiliente.
* **models/**:
  * mejor_modelo_kfold.txt: **Modelo LightGBM entrenado** (23 features) seleccionado por validación cruzada K-Fold.
  * 	raining_stats.json: Estadísticas de rendimiento (Accuracy, Recall, Precision, matriz de confusión) e importancia de características.
* **Cargo.toml & Cargo.lock**: Definición de dependencias exactas (polars, lightgbm3, xum, 	okio, etc.).
* **dashboard_live.html**: Tablero interactivo de operaciones en tiempo real con WebSocket, KPIs carrier-grade y control de mitigaciones (Shadow Routing / Throttling).
* **eporte_interactivo.html**: Reporte interactivo con análisis de IA Explicable (XAI).
* **Dockerfile**: Configuración multicapa optimizada para compilar en Linux/Render con Rust, CMake 3.31 y OpenMP.
* **onts/**: Tipografías Liberation Sans requeridas para renderizado.
* **
dr_db.json**: Registro histórico de incidentes detectados.
* **EXPLICACION_METRICAS_NDR.txt**: Guía técnica de métricas operativas y defensa del modelo.
* **eporte_kfold.pdf & olds_chart.png**: Gráfica y reporte formal de validación cruzada K-Fold.

### 2. rain-training/
* **models/**:
  * modelo_ndr_v1.txt y modelo_ndr_v2_high_prec.txt: Variantes de modelos entrenados de alta precisión.
  * inal_metrics_ndr.json y metrics.json: Métricas de evaluación y benchmarks.

---

## ❓ ¿Por qué NO necesitas la "Data" ni los "Compilados"?

1. **La Data cruda (shared-data/raw/ ~9.3 GB)**:
   * **Innecesaria para inferencia/producción**: Los modelos ya están 100% entrenados (mejor_modelo_kfold.txt).
   * **Modo Cloud / Resiliente automático**: El código de src/main.rs detecta automáticamente si los CSVs están presentes. Si no lo están, activa el **generador sintético basado en perfiles de tráfico Tigo**, permitiendo que el servidor, la inferencia de IA y el dashboard funcionen sin requerir los gigabytes de datos crudos. Solo se requeriría la data si se quisiera reentrenar desde cero (cargo run -- train).
2. **Los compilados (	arget/ ~3.15 GB)**:
   * Los compilados son temporales y específicos de la máquina y sistema operativo de origen (Windows x86_64).
   * Al transferir a otro entorno, Rust compila de forma nativa, limpia y óptima con cargo build.

---

## 🚀 Instrucciones de Despliegue en el Nuevo Entorno

### Opción A: Compilación Nativa con Cargo (Rust)

#### Requisitos Previos:
* **Rust**: ustup update (versión 1.80+)
* **CMake**: >= 3.20 (para compilar la librería nativa de LightGBM)
* **OpenMP**:
  * En Ubuntu/Debian: sudo apt-get update && sudo apt-get install -y libomp-dev clang cmake
  * En Windows: Incluido al tener instalado Visual Studio C++ Build Tools.

#### Pasos:
`ash
cd core_engine

# Compilar para producción
cargo build --release

# Ejecutar el servidor NDR
cargo run --release -- server
`
El dashboard estará accesible en: **http://localhost:3000**

---

### Opción B: Despliegue en Contenedor (Docker / Render / Cloud)

`ash
cd core_engine

# Construir la imagen Docker
docker build -t tigo-ndr:latest .

# Ejecutar el contenedor exponiendo el puerto 3000
docker run -d -p 3000:3000 --name tigo-ndr-server tigo-ndr:latest
`

---

## ⚙️ Variables de Entorno
* PORT: Puerto TCP donde escuchará el servidor web/WebSocket (por defecto: 3000).
