# 📋 PyTask - Automatización de Macros (v2.0.0 C#)

<div align="center">

![PyTask Banner](https://img.shields.io/badge/PyTask-v2.0.0-blue?style=for-the-badge)
![CSharp](https://img.shields.io/badge/C%23-Avalonia-purple?style=for-the-badge&logo=dotnet)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Windows-lightgrey?style=for-the-badge&logo=windows)
![Games](https://img.shields.io/badge/Modo-SendInput-red?style=for-the-badge&logo=windows)

**Port oficial en C# + Avalonia UI del proyecto original en Python/PyQt6**  
**🎮 Compatible con aplicaciones y juegos que requieren entrada de bajo nivel**

[![Descargar](https://img.shields.io/badge/Descargar-PyTask%20v2.0.0-brightgreen?style=for-the-badge&logo=windows)](https://github.com/4ismael1/PyTask/releases/tag/v2.0.0)

[Características](#-características) • [Uso](#-uso) • [Instalación](#-instalación) • [Changelog](#-changelog)

</div>

---

## 📸 Interfaz

<div align="center">

![Interfaz Principal](https://github.com/4ismael1/PyTask/releases/download/v1.1.0/PyTask.png)

*Diseño compacto (350x110px) con 5 botones: Open, Save, Rec, Play, Prefs*

</div>

---

## 🔄 Migración de Python a C#

- El código Python/PyQt6 se mantiene en la rama [`v1.0.0-python-legacy`](https://github.com/4ismael1/PyTask/tree/v1.0.0-python-legacy).
- La rama `main` contiene PyTask **v2.0.0** escrito en **C# + Avalonia UI**:
  - Ejecutable nativo sin dependencias de Python.
  - Hooks globales endurecidos + entrada SendInput de bajo nivel.
  - Configuración persistente vía SQLite en `%APPDATA%\PyTask`.

### ⏱ Sobre el ejecutable
- Generado con `dotnet publish -c Release -r win-x64 /p:PublishSingleFile=true /p:PublishTrimmed=false /p:PublishReadyToRun=true --self-contained true`.
- **Primera ejecución**: tarda ~1–2 s (extrae las DLL nativas a la caché interna).
- **A partir de la segunda**: arranque casi instantáneo (reutiliza caché).
- No necesita instalar nada adicional; basta con `PyTask-v2.0.0.exe`.

---

## ✨ Características

### 🎮 Modo de Entrada de Bajo Nivel (SendInput)
- Funciona en aplicaciones/juegos que bloquean entrada sintética.
- Activado por defecto (Prefs → “Modo Juegos (SendInput)”).
- Eventos indistinguibles del hardware real.

### ⚡ Interfaz optimizada
- UI de 350x110 px con 5 botones e iconos HD.
- Menús y estados instantáneos.
- Hotkeys mostrados en tiempo real en la barra de estado y menú.

### 🎬 Grabación/Reproducción
- Captura precisa de mouse/teclado.
- Velocidades: ½x, 1x, 2x, 100x, personalizada.
- Modos: una vez, N veces, infinito o intervalo cada X segundos.

### ⌨️ Hotkeys Globales
- Predeterminado: **F9** grabar, **F10** reproducir.
- Configurables (F5–F12).
- Funcionan desde cualquier app.

### 💾 Almacenamiento
- Formato `.macro` (JSON) compatible con la versión Python.
- Configuración en SQLite bajo `%APPDATA%\PyTask`.
- Soporta ejecución portable o instalación.

---

## 🚀 Instalación

### Versión Portable (recomendada)
Descarga el release v2.0.0 y ejecuta `PyTask-v2.0.0.exe`. La primera vez puede tardar ~2 s; después es instantáneo.

### Instalador (opcional)
1. Ejecuta el publish anterior para poblar `bin/Release/.../publish`.
2. Abre `Installer/PyTaskInstaller.iss` con Inno Setup.
3. Compila y obtendrás `Installer/Output/PyTaskInstaller.exe`.

### Desde el código fuente
```bash
git clone https://github.com/4ismael1/PyTask.git
cd PyTask

# Restaurar y compilar
dotnet restore
dotnet build
dotnet run
```
**Requisitos**  
- Windows 10/11 x64  
- .NET SDK 9.0  
- Visual Studio 2022 (o VS Code + C#)

---

## 📖 Uso

### 🎮 Con apps exigentes
1. Prefs (⚙) → “Modo Juegos (SendInput)” (ya viene ON).
2. Graba con F9, reproduce con F10.
3. Funciona incluso con anti-macros básicos.

### Grabar
1. F9 o botón “Rec”.
2. Ejecuta la secuencia.
3. F9 para detener.
4. Guarda con “Save”.

### Reproducir
1. Abre una macro o graba una nueva.
2. F10 o “Play”.
3. Ajusta velocidad/modo desde el menú.
4. F10 para detener.

---

## 🛠️ Tecnologías

- Avalonia UI 11.3.8
- CommunityToolkit.MVVM
- Microsoft.Data.SQLite
- Windows API (SendInput + hooks low-level)

---

## 📊 Changelog

### [v2.0.0] - 2025-11-08
- Migración completa a C# + Avalonia UI (la versión Python queda en la rama `v1.0.0-python-legacy`).
- Ejecutable single-file self-contained; la primera ejecución tarda ~1–2 s (extrae dependencias) y luego arranca al instante.
- Hooks globales reforzados, hotkeys dinámicos en la UI y reproducción con soporte multi-monitor.
- Configuración persistente mediante SQLite en `%APPDATA%\PyTask`.
- Script de instalador oficial (Inno Setup) y documentación actualizada con instrucciones de publicación portable.

### [v1.1.0] - 2025-10-29 (Python/PyQt6)
- Modo SendInput para mejor compatibilidad con juegos.
- UI optimizada y mejoras de rendimiento.
- Hotkeys dinámicos en mensajes y mejoras en textos.

### [v1.0.0] - 2025-10-28
- Primera versión pública en Python/PyQt6.

---

## 📝 Notas

- Ejecuta como administrador si los hotkeys no responden.
- El modo SendInput está activo por defecto para máxima compatibilidad.
- Coordenadas absolutas: soporta setups multi-monitor.
- Usa la herramienta de forma ética.

---

## 🎨 Créditos

- Iconos: [Kiranshastry - Flaticon](https://www.flaticon.com/)
- Desarrollador: [@4ismael1](https://github.com/4ismael1)

---

## 📄 Licencia

MIT License © 2025 4ismael1

---

<div align="center">

**Hecho con ❤️ por [@4ismael1](https://github.com/4ismael1)**  
⭐ Dale una estrella si este proyecto te fue útil

</div>



