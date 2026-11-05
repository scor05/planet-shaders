# Planetas y estrellas con shaders procedurales

En este repositorio está mi implementación de un renderer de modelos .obj con shaders, el cual implementa shaders para los siguientes 3 cuerpos celestiales:

- Estrella de neutrones: una estrella azul con "solar flares" que salen de ella.
- Planeta oceánico: mi imitación del planeta Tierra usando fbm para los continentes y nubes que se mueven con el paso del tiempo
- Gigante gaseoso: un planeta similar a Júpiter con bandas implementadas usando ruido con una función sin.

## Capturas de los cuerpos celestes

### 1. Estrella de neutrones
<img width="788" height="594" alt="image" src="https://github.com/user-attachments/assets/e045e858-aa25-4128-a92e-6991df9de27a" />


### 2. Planeta oceánico
<img width="798" height="594" alt="image" src="https://github.com/user-attachments/assets/ab3688d1-cecb-4765-bebc-43f3a234436a" />


### 3. Gigante gaseoso
<img width="793" height="591" alt="image" src="https://github.com/user-attachments/assets/e1e693b8-6136-47e8-bd24-a254b4328cf6" />


## Demostración de las animaciones
Como para los tres cuerpos utilicé animaciones que dependen del tiempo, decidí grabar un solo video que demuestra los tres shaders, el cual se puede encontrar [en este enlace de YouTube](https://youtu.be/c2uHly26wDA)


## Estructura del proyecto

```bash
.
├── assets
│   └── obj
│       └── sphere.obj
├── Cargo.lock
├── Cargo.toml
├── README.md
└── src
    ├── algorithms
    │   ├── barycentric.rs
    │   ├── mod.rs
    │   ├── scanline.rs
    │   ├── triangle.rs
    │   └── wireframe.rs
    ├── framebuffer.rs
    ├── line.rs
    ├── main.rs
    ├── model.rs
    ├── procedural.rs
    └── shaders.rs

5 directories, 15 files
```

Es de notar que los archivos `scanline.rs` y `wireframe.rs` son simplemente leftovers que dejé en el proyecto por si en algún momento los fuera a utilizar, pues el renderer es el mismo que utilicé para el laboratorio #4 y #5 de la clase.

## Ejecución 

Para ejecutar el proyecto, únicamente se necesitan Rust y Cargo para poder correr el comando desde la raíz del repositorio:

```bash
cargo run --release
```

## Controles

| Tecla | Control |
|---|---|
| `1` | Seleccionar la estrella de neutrones |
| `2` | Seleccionar el planeta oceánico |
| `3` | Seleccionar el gigante gaseoso |
| Flechas | Trasladar la esfera en X/Y |
| `W` / `S` | Rotar alrededor del eje X |
| `A` / `D` | Rotar alrededor del eje Y |
| `Z` / `X` | Rotar alrededor del eje Z |
| `Q` / `E` | Aumentar o reducir la escala |


## Funcionamiento del renderer

El programa carga `assets/obj/sphere.obj`, triangula sus caras y aplica transformaciones de traslación, rotación y escala. Los triángulos se rasterizan con coordenadas baricéntricas y a cada fragmento se le interpola su profundidad, color y posición 3D original. Para aplicar los shaders, se utilizó la famosa fórmula del combiner de la n64:

```text
res = (A - B) × C + D
```

También cabe mencionar que los algoritmos procedimentales implementados están en `src/procedural.rs` y son los siguientes: 

- Value noise 3D: interpola valores pseudoaleatorios en una cuadrícula tridimensional.
- FBM: combina varias octavas de value noise para producir estructuras a distintas escalas.
- Cellular/Worley 3D: calcula la distancia al punto característico más cercano y genera patrones similares a células.
- Smoothstep: suaviza máscaras y evita transiciones abruptas entre regiones.
- Hash 3D determinista: permite repetir los mismos mundos en cada ejecución sin texturas externas.

## Diseño de los shaders

### Estrella de neutrones

Implementación principal: `src/shaders.rs` en las funciones `star` y `flare_field`.

La superficie parte de una paleta azul oscuro, azul eléctrico, cyan y un poco de blanco. Cellular genera la granulación energética para darle el patrón de una estrella y luego se aplica FBM para deformar las células y agregar turbulencia animada. Luego con una siguiente pasada de FBM se generan las regiones más oscuras de la estrella. 

Las solar flares se calculan en coordenadas esféricas. Cada llamarada tiene un ciclo independiente de formación y decaimiento, durante el cual un arco elíptico crece sobre la superficie, mientras FBM rompe su contorno para evitar que parezca un círculo perfecto. Finalmente se agregan dos desenfoques gaussianos en CPU para producir un efecto de bloom.

Los parámetros que afectan a la estrella son:

| Parámetro en el código | Efecto |
|---|---|
| `position  8.0` | Frecuencia y tamaño de las células |
| `time  0.12` | Velocidad del movimiento celular |
| `time  1.7` | Velocidad del parpadeo de la superficie |
| `time  0.105` en `flare_field` | Frecuencia del ciclo de las llamaradas |
| `0.10 + cycle  0.72` | Tamaño inicial y expansión de los arcos |
| `blur_gaussian(31)` y `blur_gaussian(11)` | Extensión del resplandor exterior |

### Planeta oceánico

Implementación principal: `src/shaders.rs` en la función `rocky_planet`.

Los continentes se obtienen deformando FBM con otra señal de ruido y aplicando una máscara para separar tierra y océano. Un segundo ruido controla la humedad: las regiones húmedas se vuelven verdes y las secas conservan tonos marrones. La latitud y el ruido de detalle generan casquetes polares irregulares.

La capa de nubes utiliza coordenadas que rotan con el tiempo, por lo que se mueve independientemente de los continentes. La superficie se ilumina con luz difusa y los océanos reciben además un reflejo especular, mientras la tierra conserva una respuesta matte.

Parámetros importantes:

| Parámetro en el código | Efecto |
|---|---|
| `position  3.6` | Escala de los continentes |
| `smoothstep(0.49, 0.59, terrain)` | Cantidad y suavidad de la costa |
| `position  13.0` | Detalle pequeño de la superficie |
| `position.y.abs()` | Distribución del hielo por latitud |
| `cloud_point  7.5` | Tamaño de las formaciones de nubes |
| `time  0.045` | Velocidad de rotación de las nubes |
| `smoothstep(0.60, 0.72, clouds)` | Cobertura de nubes |
| `light_direction  0.075` | Separación aparente de las sombras de nubes |
| `powf(48.0)` | Tamaño y dureza del reflejo oceánico |

### Gigante gaseoso

Implementación principal: `src/shaders.rs` en `gas_giant`.

Las bandas se forman con una onda sinusoidal que depende de la latitud, la cual se deforma con FBM para que las transiciones parezcan corrientes de líquido y no líneas perfectamente rectas. Una segunda capa agrega filamentos atmosféricos pequeños. La tormenta utiliza una máscara elíptica localizada, en la cual se modifica la latitud usada por las bandas mediante el ángulo alrededor del centro de la tormenta.

Parámetros importantes:

| Parámetro en el código | Efecto |
|---|---|
| `flowing  4.0` | Escala de la distorsión atmosférica |
| `warped_latitude  29.0` | Número y grosor de las bandas |
| `(broad_noise - 0.5)  0.13` | Intensidad de la deformación de las bandas |
| `smoothstep(0.08, 0.48, storm_distance)` | Tamaño y suavidad del vórtice |
| `time  0.65` | Velocidad de la distorsión de la tormenta |
| `storm_distance  46.0` | Cantidad de detalles concéntricos de la tormenta |
| `flowing  15.0` | Escala de los filamentos finos |
