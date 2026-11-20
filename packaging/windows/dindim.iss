; Inno Setup script for the Windows installer.
; Built in CI with: iscc /DAppVersion=0.1.0-alpha.1 /DSourceDir=<folder> /DOutputDir=<folder> dindim.iss

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{55DA0FD7-9A9C-4828-A664-DBD43BB9AE91}
AppName=dindim
AppVersion={#AppVersion}
AppVerName=dindim {#AppVersion}
AppPublisher=dindim contributors
AppPublisherURL=https://github.com/thdtr/dindim
AppSupportURL=https://github.com/thdtr/dindim/issues
AppUpdatesURL=https://github.com/thdtr/dindim/releases
DefaultDirName={localappdata}\Programs\dindim
DefaultGroupName=dindim
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir={#OutputDir}
OutputBaseFilename=dindim-v{#AppVersion}-windows-x86_64-setup
SetupIconFile=..\..\assets\icons\dindim.ico
UninstallDisplayIcon={app}\dindim.exe
LicenseFile=..\..\LICENSE
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
CloseApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{group}\dindim"; Filename: "{app}\dindim.exe"
Name: "{autodesktop}\dindim"; Filename: "{app}\dindim.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\dindim.exe"; Description: "{cm:LaunchProgram,dindim}"; Flags: nowait postinstall skipifsilent
