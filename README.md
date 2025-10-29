# 📋 PyTask - Automatización de Macros

<div align="center">

![PyTask Banner](https://img.shields.io/badge/PyTask-v1.1.0-blue?style=for-the-badge)
![Python](https://img.shields.io/badge/Python-3.8+-yellow?style=for-the-badge&logo=python)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)
![Platform](https://img.shields.io/badge/Platform-Windows-lightgrey?style=for-the-badge&logo=windows)
![Games](https://img.shields.io/badge/Modo-Bajo%20Nivel-red?style=for-the-badge&logo=windows)

**Herramienta profesional de automatización de macros para Windows**  
**🎮 Compatible con aplicaciones y juegos que requieren entrada de bajo nivel**

[![Descargar](https://img.shields.io/badge/Descargar-PyTask%20v1.1.0-brightgreen?style=for-the-badge&logo=windows)](https://github.com/4ismael1/PyTask/releases/tag/v1.1.0)

[Características](#-características) • [Uso](#-uso) • [Instalación](#-instalación) • [Changelog](#-changelog)

</div>

---

## 📸 Interfaz

<div align="center">

![Interfaz Principal](https://github.com/4ismael1/PyTask/blob/main/screenshots/PyTask.png)

*Diseño compacto (350x110px) con 5 botones: Open, Save, Rec, Play, Prefs*

</div>

---

## ✨ Características

### 🎮 **NUEVO en v1.1**: Modo de Entrada de Bajo Nivel (SendInput)
- ✅ **Mayor compatibilidad** - Funciona en aplicaciones y juegos que filtran entrada sintética de alto nivel
- ✅ **Sin pre-interacción requerida** - Funciona inmediatamente sin configuración adicional
- ✅ **Tecnología de bajo nivel** (Windows SendInput API)
- ✅ **Entrada directa al sistema** - Indistinguible de dispositivos de hardware reales
- 🔧 Activable en Preferencias → "Modo Juegos (SendInput)" (activo por defecto)

### 🎮 Interfaz Compacta y Optimizada
- Diseño minimalista de solo 350x110px
- 5 botones esenciales con iconos profesionales
- **97.5% más rápido** que la versión original
- Apertura de menús instantánea (50ms vs 2000ms)
- Barra de título blanca integrada con Windows 11

### 🎬 Grabación y Reproducción
- Captura precisa de mouse y teclado
- Múltiples velocidades: ½x, 1x, 2x, 100x, personalizada
- Modo intervalo: Ejecuta cada X segundos (5s, 10s, 30s, 60s, personalizado)
- Repeticiones configurables: 1 vez, N veces, o infinito

### ⌨️ Hotkeys Globales Configurables
- **F9** (predeterminado) - Iniciar/Detener grabación
- **F10** (predeterminado) - Reproducir/Detener macro
- **Personalizables** de F5 a F12
- Funcionan desde cualquier aplicación

### 💾 Almacenamiento
- Archivos .macro en formato JSON
- Base de datos SQLite para configuración (guardada en AppData)
- Totalmente portable

---

## 🚀 Instalación

### Versión Portable (Recomendada)
Descarga **[PyTask.exe v1.1.0](https://github.com/4ismael1/PyTask/releases/tag/v1.1.0)** y ejecútalo directamente. No requiere instalación.

### Desde el Código Fuente
```bash
git clone https://github.com/4ismael1/PyTask.git
cd PyTask
pip install -r requirements.txt
python main.py
```

**Requisitos**:
- Python 3.8+
- Windows 10/11
- PyQt6 6.6.1+
- pynput 1.7.6+
- keyboard 0.13.5+

---

## 📖 Uso

### 🎮 Usar con Aplicaciones Exigentes
1. Ve a **Preferencias** (⚙️)
2. Activa **"Modo Juegos (SendInput)"** (debería estar activo por defecto)
3. Graba tu macro normalmente
4. ¡Funciona inmediatamente en aplicaciones que requieren entrada de bajo nivel!

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

- **PyQt6** - Interfaz gráfica moderna
- **Windows SendInput API** - Compatibilidad con juegos (v1.1+)
- **ctypes** - Integración con Windows API
- **pynput** - Captura de eventos y fallback
- **keyboard** - Hotkeys globales
- **SQLite** - Persistencia de configuración

---

## 📊 Changelog

### [v1.1.0] - 2025-10-29 🎮🚀
**Actualización Mayor: Compatibilidad con Entrada de Bajo Nivel + Optimización de Rendimiento**

- ✅ **Modo de Entrada de Bajo Nivel** - Windows SendInput API para máxima compatibilidad
- ✅ **Mayor compatibilidad** - Funciona en aplicaciones y juegos que requieren entrada directa al sistema
- ✅ **Sin pre-interacción** - Las macros funcionan inmediatamente
- ⚡ **97.5% más rápido** - Apertura de menús optimizada (2000ms → 50ms)
- 💾 **Caché inteligente** - Iconos y menús en memoria
- 🎨 **Textos visibles** - Corregido problema de texto blanco sobre blanco
- ⌨️ **Hotkeys dinámicos** - Los mensajes reflejan las teclas configuradas actuales
- 📦 **Lazy imports** - Startup 25% más rápido
- 🔧 **Checkbox configurable** - Modo de bajo nivel activable en Preferencias
- 📚 **Documentación completa** - Guías técnicas y de uso

### [v1.0.0] - 2025-10-28 🎉
- 🎬 Grabación y reproducción de macros
- ⌨️ Hotkeys globales F9/F10
- 💾 Formato .macro y SQLite
- 🎨 Interfaz compacta 350x110px

---

## 📝 Notas

- **Permisos de administrador**: Ejecuta como administrador si los hotkeys no funcionan
- **Modo de Bajo Nivel**: Activo por defecto para máxima compatibilidad con aplicaciones exigentes
- **Coordenadas absolutas**: Las posiciones del mouse son absolutas (compatible con multi-monitor)
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
