#  Mandala Weaver VCS

> **El Tiempo Circular del Software.**
> Un Sistema de Control de Versiones (VCS) espacial, semántico y radial.

Mandala Weaver abraza el **Espacio de Versionado Radial**. En este sistema, el código no avanza en una línea recta de texto plano; emana como unidades de lógica pura desde un centro inmutable hacia anillos concéntricos de evolución.

##  Conceptos Fundamentales

*  **Mónadas (No líneas de texto):** Utiliza análisis de árboles de sintaxis abstracta (`ast-grep`) para versionar lógica pura (funciones, structs), ignorando formato o espacios en blanco.
*  **Vectores y Anillos:** El software se organiza espacialmente. Los dominios lógicos (ej. UI, Red, Core) son Ángulos ($\theta$), y las iteraciones temporales son Radios ($r$).
*  **El Fin de los Merge Conflicts:** Las ramas no chocan. Las características coexisten en el espacio.
*  **La Destilación (Spatial Checkout):** Compila una versión ejecutable dibujando un polígono visual que conecta características estables de diferentes anillos y dominios.

## Stack Tecnológico

El "Telar" está construido sobre tres pilares de ingeniería de alto rendimiento:

* ** El Motor (Core):** `Rust` + `nalgebra` (Cálculo orbital y espacial) + `ast-grep` (Parsing semántico).
* ** El Registro Akáshico:** `SurrealDB` embebido (Base de datos de grafos para trazar el linaje evolutivo del código).
* ** El Lienzo (UI):** `Tauri` + `Astro` + `React` + `D3.js` (Renderizado interactivo masivo acelerado por hardware).


*** *El software se cultiva desde el centro hacia afuera.*
