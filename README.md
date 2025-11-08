# PyTask v2.0.0 (C# + Avalonia UI)

> Port oficial del proyecto original escrito en Python/PyQt6. El código Python completo se mantiene en la rama [`v1.0.0-python-legacy`](https://github.com/4ismael1/PyTask/tree/v1.0.0-python-legacy). La rama `main` contiene la nueva implementación en C# y Avalonia UI, optimizada para Windows.

## ¿Por qué la migración?

- **Inicio inmediato**: aplicación nativa, sin dependencias de Python ni intérprete externo.
- **Modo Juegos (SendInput)** integrado para compatibilidad con apps y juegos que bloquean la entrada sintética tradicional.
- **Hooks globales más seguros** y base de datos SQLite persistente en `%APPDATA%\PyTask`.
- **Instalación opcional** mediante Inno Setup o ejecutable portable single-file.

## Funcionamiento del ejecutable

Publicamos usando `dotnet publish -c Release -r win-x64 /p:PublishSingleFile=true /p:PublishTrimmed=false /p:PublishReadyToRun=true --self-contained true`.

- La **primera ejecución** tarda ~1–2 segundos porque .NET extrae las bibliotecas nativas en la caché local.
- A partir de la **segunda ejecución**, el arranque es prácticamente instantáneo porque reutiliza la caché.
- El ejecutable crea automáticamente `%APPDATA%\PyTask\pytask.db` para guardar ajustes.

## Características

- Grabación y reproducción de mouse/teclado con múltiples velocidades.
- Hotkeys globales configurables (por defecto F9 grabar / F10 reproducir).
- Modos de reproducción: una sola vez, infinito o con intervalos.
- Interfaz compacta (350x110 px) con sesión de estado y opciones rápidas.
- Exporta/importa macros en formato `.macro` (JSON compatible con la versión Python).

## Requisitos de desarrollo

- Windows 10/11 x64
- .NET SDK 9.0
- Visual Studio 2022 (o VS Code + extensiones C#)

## Compilación / Publicación

```powershell
# Restaurar dependencias
 dotnet restore

# Compilar en Debug
 dotnet build

# Ejecutar
dotnet run

# Publicar ejecutable portable (self-contained, single-file, R2R)
dotnet publish -c Release -r win-x64 /p:PublishSingleFile=true /p:PublishTrimmed=false /p:PublishReadyToRun=true --self-contained true
```

El ejecutable resultante se encuentra en `bin/Release/net9.0/win-x64/publish/PyTaskAvalonia.exe`.

## Instalador opcional

Hay un script de Inno Setup en `Installer/PyTaskInstaller.iss`. Para generar el instalador:

1. Ejecuta el publish anterior (para poblar la carpeta `publish`).
2. Abre `Installer/PyTaskInstaller.iss` en Inno Setup y compila (`Build`).
3. Obtendrás `Installer/Output/PyTaskInstaller.exe` listo para distribución.

## Estructura

```
PyTask/
+-- Assets/
+-- Converters/
+-- Models/
+-- Services/
+-- ViewModels/
+-- Views/
+-- Installer/
+-- App.axaml / App.axaml.cs
+-- Program.cs
+-- PyTaskAvalonia.csproj
+-- README.md
+-- LICENSE
```

## Compatibilidad con la versión Python

- El formato `.macro` es el mismo; puedes crear archivos en cualquiera de las versiones y reutilizarlos.
- La rama `v1.0.0-python-legacy` conserva el código PyQt6 original por motivos históricos o si necesitas portabilidad Python.

## Licencia

MIT License © 2025 4ismael1

