# PyTask v2.0.0 (C# + Avalonia UI)

> Port oficial del proyecto original escrito en Python/PyQt6. El c�digo Python completo se mantiene en la rama [`v1.0.0-python-legacy`](https://github.com/4ismael1/PyTask/tree/v1.0.0-python-legacy). La rama `main` contiene la nueva implementaci�n en C# y Avalonia UI, optimizada para Windows.

## �Por qu� la migraci�n?

- **Inicio inmediato**: aplicaci�n nativa, sin dependencias de Python ni int�rprete externo.
- **Modo Juegos (SendInput)** integrado para compatibilidad con apps y juegos que bloquean la entrada sint�tica tradicional.
- **Hooks globales m�s seguros** y base de datos SQLite persistente en `%APPDATA%\PyTask`.
- **Instalaci�n opcional** mediante Inno Setup o ejecutable portable single-file.

## Funcionamiento del ejecutable

Publicamos usando `dotnet publish -c Release -r win-x64 /p:PublishSingleFile=true /p:PublishTrimmed=false /p:PublishReadyToRun=true --self-contained true`.

- La **primera ejecuci�n** tarda ~1�2 segundos porque .NET extrae las bibliotecas nativas en la cach� local.
- A partir de la **segunda ejecuci�n**, el arranque es pr�cticamente instant�neo porque reutiliza la cach�.
- El ejecutable crea autom�ticamente `%APPDATA%\PyTask\pytask.db` para guardar ajustes.

## Caracter�sticas

- Grabaci�n y reproducci�n de mouse/teclado con m�ltiples velocidades.
- Hotkeys globales configurables (por defecto F9 grabar / F10 reproducir).
- Modos de reproducci�n: una sola vez, infinito o con intervalos.
- Interfaz compacta (350x110 px) con sesi�n de estado y opciones r�pidas.
- Exporta/importa macros en formato `.macro` (JSON compatible con la versi�n Python).

## Requisitos de desarrollo

- Windows 10/11 x64
- .NET SDK 9.0
- Visual Studio 2022 (o VS Code + extensiones C#)

## Compilaci�n / Publicaci�n

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
3. Obtendr�s `Installer/Output/PyTaskInstaller.exe` listo para distribuci�n.

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

## Compatibilidad con la versi�n Python

- El formato `.macro` es el mismo; puedes crear archivos en cualquiera de las versiones y reutilizarlos.
- La rama `v1.0.0-python-legacy` conserva el c�digo PyQt6 original por motivos hist�ricos o si necesitas portabilidad Python.

## Licencia

MIT License � 2025 4ismael1


