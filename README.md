# 📋 PyTask - Automatización de Macros

<div align="center">

![PyTask Banner](https://img.shields.io/badge/PyTask-v1.0-blue?style=for-the-badge)
![Python](https://img.shields.io/badge/Python-3.8+-yellow?style=for-the-badge&logo=python)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Windows-lightgrey?style=for-the-badge&logo=windows)

**Herramienta profesional de automatización de macros para Windows**

[![Descargar](https://img.shields.io/badge/Descargar-PyTask%20v1.0.0-brightgreen?style=for-the-badge&logo=windows)](https://github.com/4ismael1/PyTask/releases/tag/v1.0.0)

[Características](#-características) • [Uso](#-uso) • [Instalación](#-instalación)

</div>

---

## 📸 Interfaz

<div align="center">

![Interfaz Principal](https://raw.githubusercontent.com/4ismael1/PyTask/main/screenshots/PyTask.png)

*Diseño compacto (350x110px) con 5 botones: Open, Save, Rec, Play, Prefs*

</div>

---

## ✨ Características

### 🎮 Interfaz Compacta
- Diseño minimalista de solo 350x110px
- 5 botones esenciales con iconos profesionales
- Barra de título blanca integrada con Windows 11

### 🎬 Grabación y Reproducción
- Captura precisa de mouse y teclado
- Múltiples velocidades: ½x, 1x, 2x, 100x, personalizada
- Modo intervalo: Ejecuta cada X segundos (5s, 10s, 30s, 60s, personalizado)
- Repeticiones configurables: 1 vez, N veces, o infinito

### ⌨️ Hotkeys Globales
- **F9** - Iniciar/Detener grabación
- **F10** - Reproducir/Detener macro
- Funcionan desde cualquier aplicación

### 💾 Almacenamiento
- Archivos .macro en formato JSON
- Base de datos SQLite para configuración (guardada en AppData)
- Totalmente portable

---

## 🚀 Instalación

### Versión Portable (Recomendada)
Descarga **[PyTask.exe](https://github.com/4ismael1/PyTask/releases/tag/v1.0.0)** y ejecútalo directamente. No requiere instalación.

### Desde el Código Fuente
```bash
git clone https://github.com/4ismael1/PyTask.git
cd PyTask
pip install -r requirements.txt
python main.py
```

---

## 📖 Uso

### Grabar una Macro
1. Presiona **F9** o click en **"Rec"**
2. Realiza las acciones que deseas automatizar
3. Presiona **F9** nuevamente para detener
4. Guarda con el botón **"Save"**

### Reproducir una Macro
1. Abre un archivo con **"Open"** o graba uno nuevo
2. Presiona **F10** o click en **"Play"**
3. Configura velocidad y repeticiones desde el menú desplegable
4. Para detener, presiona **F10** nuevamente

### Menú de Opciones

#### Velocidades
- **½x** - Reproducción lenta
- **1x** - Velocidad normal
- **2x** - Doble velocidad
- **100x** - Súper rápido
- **Personalizada** - Define tu propia velocidad

#### Repeticiones
- **Modo Intervalo**: Reproduce cada X segundos con cantidad configurable
- Opciones: 5s, 10s, 30s, 60s o personalizado
- Cantidad: 1 a 999 veces o infinito

#### Preferencias
- Cambiar hotkeys (F9/F10)
- Ventana siempre visible
- Mostrar/ocultar barra de estado

---

## 🛠️ Tecnologías

- **PyQt6** - Interfaz gráfica
- **pynput** - Captura de eventos
- **keyboard** - Hotkeys globales
- **SQLite** - Persistencia de configuración

---

## 📝 Notas

- **Permisos de administrador**: Ejecuta como administrador si los hotkeys no funcionan
- **Coordenadas absolutas**: Las posiciones del mouse son absolutas
- **Uso responsable**: Usa esta herramienta de forma ética y legal

---

## 🎨 Créditos

### Iconos
<a href="https://www.flaticon.es/iconos-gratis/lista" title="lista iconos">Lista iconos creados por Kiranshastry - Flaticon</a>

### Desarrollador
**GitHub**: [@4ismael1](https://github.com/4ismael1)

---

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo [LICENSE](LICENSE) para más detalles.

---

<div align="center">

**Hecho con ❤️ por [@4ismael1](https://github.com/4ismael1)**

⭐ Dale una estrella si este proyecto te fue útil

</div>
