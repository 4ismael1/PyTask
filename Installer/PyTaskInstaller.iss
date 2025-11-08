[Setup]
AppId={{C7ABA812-6E6B-4E67-8612-0F741D62C9FD}
AppName=PyTaskAvalonia
AppVersion=2.0.0
AppPublisher=PyTask
AppPublisherURL=https://example.com
DefaultDirName={autopf}\PyTask
DisableProgramGroupPage=yes
OutputDir=Output
OutputBaseFilename=PyTaskInstaller
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
SetupIconFile=..\Assets\Icons\portapapeles.ico

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Files]
Source: "..\bin\Release\net9.0\win-x64\publish\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\PyTask"; Filename: "{app}\PyTaskAvalonia.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\PyTask"; Filename: "{app}\PyTaskAvalonia.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Crear un icono en el escritorio"; GroupDescription: "Opciones adicionales:"

[Run]
Filename: "{app}\PyTaskAvalonia.exe"; Description: "Iniciar PyTask"; Flags: nowait postinstall skipifsilent
