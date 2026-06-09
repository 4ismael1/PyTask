# PyTask - Automatizacion de Macros (v3.0.0 Rust)

<div align="center">

![PyTask Banner](https://img.shields.io/badge/PyTask-v3.0.0-blue?style=for-the-badge)
![Rust](https://img.shields.io/badge/Rust-Win32_UI-orange?style=for-the-badge&logo=rust)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Windows-lightgrey?style=for-the-badge&logo=windows)
![Games](https://img.shields.io/badge/Modo-SendInput-red?style=for-the-badge&logo=windows)

**Reescritura nativa en Rust + Win32 UI de PyTask.**  
**Compatible con aplicaciones y juegos que requieren entrada de bajo nivel.**

[![Descargar](https://img.shields.io/badge/Descargar-PyTask%20v3.0.0-brightgreen?style=for-the-badge&logo=windows)](https://github.com/4ismael1/PyTask/releases/tag/v3.0.0)

[Caracteristicas](#caracteristicas) - [Uso](#uso) - [Instalacion](#instalacion) - [Changelog](#changelog)

</div>

---

## Interfaz

<div align="center">

![Interfaz Principal](https://github.com/4ismael1/PyTask/releases/download/v1.1.0/PyTask.png)

*Diseno compacto con 5 botones: Open, Save, Rec, Play, Prefs.*

</div>

---

## Migracion

- El codigo Python/PyQt6 se mantiene en la rama [`v1.0.0-python-legacy`](https://github.com/4ismael1/PyTask/tree/v1.0.0-python-legacy).
- El codigo C# + Avalonia UI se mantiene en la rama [`v2.0.0-csharp-legacy`](https://github.com/4ismael1/PyTask/tree/v2.0.0-csharp-legacy).
- La rama `main` contiene PyTask **v3.0.0** escrito en **Rust + Win32 UI**:
  - Ejecutable portable sin runtime de Python, .NET ni dependencias externas.
  - Interfaz nativa ligera con arranque practicamente instantaneo.
  - Hooks globales de mouse/teclado y reproduccion mediante SendInput.
  - Configuracion persistente via SQLite en `%APPDATA%\PyTask`.

### Sobre el ejecutable

- El binario final es `PyTask.exe`.
- Incluye icono, metadatos de version y manifest de Windows embebidos.
- Esta pensado como portable: descargar, ejecutar y usar.
- No realiza instalacion ni requiere permisos de administrador por defecto.

---

## Caracteristicas

### Modo de Entrada de Bajo Nivel (SendInput)

- Activado por defecto desde Prefs -> `Modo Juegos (SendInput)`.
- Mejora la compatibilidad con aplicaciones exigentes.
- Evita regrabar eventos generados por el propio reproductor.

### Interfaz nativa y rapida

- UI Win32 compacta inspirada en la interfaz original.
- Botones Open, Save, Rec, Play y Prefs.
- Menus con checks para opciones seleccionadas.
- Barra de estado con hotkeys y mensajes temporales.

### Grabacion/Reproduccion

- Captura mouse, clicks, scroll y teclado.
- Soporta combinaciones de teclas como `Shift + W`.
- Velocidades: 0.5x, 1x, 2x, 100x y personalizada.
- Modos: una vez, N veces, infinito e intervalos cada X segundos.
- Detencion rapida de reproduccion desde el boton Play o hotkey.

### Hotkeys Globales

- Predeterminado: **F9** para grabar, **F10** para reproducir/detener.
- Hotkey de grabacion configurable: F6, F7, F8, F9.
- Hotkey de reproduccion configurable: F5, F10, F11, F12.

### Almacenamiento

- Formato `.macro` en JSON compatible con versiones anteriores.
- Preferencias guardadas en `%APPDATA%\PyTask\pytask.db`.
- Funciona como app portable sin instalador.

---

## Instalacion

### Version Portable

Descarga `PyTask.exe` desde Releases y ejecutalo directamente.

### Desde el codigo fuente

```bash
git clone https://github.com/4ismael1/PyTask.git
cd PyTask
cargo run
```

### Compilar release

```bash
cargo build --release
```

El ejecutable queda en:

```text
target/release/PyTask.exe
```

**Requisitos para compilar**

- Windows 10/11 x64.
- Rust estable con toolchain MSVC.
- Visual Studio Build Tools o Windows SDK para recursos de Windows (`rc.exe` y `cvtres.exe`).

---

## Uso

### Grabar

1. Pulsa F9 o el boton `Rec`.
2. Ejecuta la secuencia que quieres capturar.
3. Pulsa F9 otra vez para detener.
4. Guarda la macro con `Save`.

### Reproducir

1. Abre una macro o graba una nueva.
2. Pulsa F10 o `Play`.
3. Ajusta velocidad/modo/intervalo desde `Prefs`.
4. Pulsa F10 o `Play` otra vez para detener.

### Apps exigentes

1. Deja activo `Prefs -> Modo Juegos (SendInput)`.
2. Graba con F9 y reproduce con F10.
3. Si una app se ejecuta como administrador, ejecuta PyTask como administrador tambien.

---

## Tecnologias

- Rust 2024.
- Win32 API via `windows`.
- `SendInput`, low-level keyboard hooks y low-level mouse hooks.
- SQLite via `rusqlite` con SQLite embebido.
- Recursos Windows generados desde `build.rs`.

---

## Changelog

### [v3.0.0] - 2026-06-09

- Reescritura de PyTask en Rust + Win32 UI.
- Ejecutable portable sin runtime externo.
- Interfaz compacta equivalente a la version anterior.
- Reproduccion una vez, infinita, personalizada e intervalos.
- Dialogos nativos para velocidad, repeticiones e intervalo.
- Hotkeys globales configurables y persistentes.
- Manifest, icono y metadatos de version embebidos en el exe.
- Smoke test para validar carga/guardado de macros y preferencias.

### [v2.0.0] - 2025-11-08

- Migracion completa a C# + Avalonia UI.
- Ejecutable single-file self-contained.
- Hooks globales reforzados, hotkeys dinamicos y soporte multi-monitor.
- Configuracion persistente mediante SQLite.

### [v1.1.0] - 2025-10-29

- Modo SendInput para mejor compatibilidad con juegos.
- UI optimizada y mejoras de rendimiento.
- Hotkeys dinamicos en mensajes y mejoras en textos.

### [v1.0.0] - 2025-10-28

- Primera version publica en Python/PyQt6.

---

## Notas

- Ejecuta como administrador si los hotkeys no responden dentro de una app elevada.
- Las coordenadas absolutas soportan configuraciones multi-monitor.
- Usa la herramienta de forma etica y respetando las reglas de cada aplicacion.

---

## Creditos

- Iconos: [Kiranshastry - Flaticon](https://www.flaticon.com/)
- Desarrollador: [@4ismael1](https://github.com/4ismael1)

---

## Licencia

MIT License (c) 2025 4ismael1

---

<div align="center">

**Hecho por [@4ismael1](https://github.com/4ismael1)**  
Estrella el proyecto si te fue util.

</div>
