; ==============================================================================
; Proteus Sovereign BOS — Native Windows Inno Setup Script
; Self-contained single-click installer with zero SaaS dependencies
; ==============================================================================

#define MyAppName "Proteus Sovereign BOS"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Proteus Sovereign Systems"
#define MyAppURL "https://github.com/Siaftoulis/proteus"
#define MyAppExeName "proteus-client.exe"
#define MyStudioExeName "proteus-design-studio.exe"

[Setup]
AppId={{9B7F162A-5D63-4C90-8C84-18B2446FE157}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} v{#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\Proteus BOS
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\..\dist\installer
OutputBaseFilename=Proteus_BOS_Setup_v{#MyAppVersion}
SetupIconFile=..\..\assets\favicon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\assets\favicon.ico

[Languages]
Name: "greek"; MessagesFile: "compiler:Languages\Greek.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
greek.PosIconDesc=Συντόμευση στην Επιφάνεια Εργασίας: Proteus POS (Ταμείο)
english.PosIconDesc=Desktop shortcut: Proteus POS (Cashier Station)
greek.StudioIconDesc=Συντόμευση στην Επιφάνεια Εργασίας: Proteus Studio (Διαχείριση)
english.StudioIconDesc=Desktop shortcut: Proteus Studio (Backoffice & SSG)
greek.LaunchPos=Εκκίνηση Proteus POS (Ταμείο)
english.LaunchPos=Launch Proteus POS (Cashier Station)
greek.LaunchStudio=Εκκίνηση Proteus Studio (Διαχείριση)
english.LaunchStudio=Launch Proteus Studio
greek.ConfigShortcut=Ρυθμίσεις Συστήματος (proteus.toml)
english.ConfigShortcut=System Configuration (proteus.toml)
greek.KeepDatabasePrompt=Βρέθηκε τοπική βάση δεδομένων πωλήσεων και ρυθμίσεων (store.db). Επιθυμείτε να διατηρηθεί για ασφάλεια;
english.KeepDatabasePrompt=A local sales and settings database (store.db) was found. Do you wish to preserve it?

[Tasks]
Name: "desktopicon_pos"; Description: "{cm:PosIconDesc}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "desktopicon_studio"; Description: "{cm:StudioIconDesc}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
; Core Binaries
Source: "..\..\dist\proteus-desktop\bin\proteus-client.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "..\..\dist\proteus-desktop\bin\proteus-design-studio.exe"; DestDir: "{app}\bin"; Flags: ignoreversion skipifsourcedoesntexist

; Default configuration - Preserved on updates (never overwritten)
Source: "proteus.default.toml"; DestDir: "{app}\config"; DestName: "proteus.toml"; Flags: onlyifdoesntexist uninsneveruninstall

; Documentation and Branding
Source: "..\..\dist\proteus-desktop\README-PORTABLE.txt"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion
Source: "..\..\assets\favicon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion

[Dirs]
Name: "{app}\data"; Flags: uninsneveruninstall
Name: "{app}\config"
Name: "{app}\bin"
Name: "{app}\assets"

[Icons]
; Start Menu Shortcuts
Name: "{autoprograms}\{#MyAppName}\Proteus POS (Ταμείο)"; Filename: "{app}\bin\{#MyAppExeName}"; IconFilename: "{app}\assets\favicon.ico"
Name: "{autoprograms}\{#MyAppName}\Proteus Studio (Διαχείριση)"; Filename: "{app}\bin\{#MyStudioExeName}"; IconFilename: "{app}\assets\favicon.ico"
Name: "{autoprograms}\{#MyAppName}\{cm:ConfigShortcut}"; Filename: "notepad.exe"; Parameters: """{app}\config\proteus.toml"""
Name: "{autoprograms}\{#MyAppName}\Οδηγίες Χρήσης"; Filename: "notepad.exe"; Parameters: """{app}\README.txt"""
Name: "{autoprograms}\{#MyAppName}\{cm:ProgramOnTheWeb,{#MyAppName}}"; Filename: "{#MyAppURL}"
Name: "{autoprograms}\{#MyAppName}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"

; Desktop Shortcuts (Optional)
Name: "{autodesktop}\Proteus POS"; Filename: "{app}\bin\{#MyAppExeName}"; IconFilename: "{app}\assets\favicon.ico"; Tasks: desktopicon_pos
Name: "{autodesktop}\Proteus Studio"; Filename: "{app}\bin\{#MyStudioExeName}"; IconFilename: "{app}\assets\favicon.ico"; Tasks: desktopicon_studio

[Run]
Filename: "{app}\bin\{#MyAppExeName}"; Description: "{cm:LaunchPos}"; Flags: nowait postinstall skipifsilent

[Code]
// Protect local database store.db from accidental deletion during uninstallation
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  DbPath: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    DbPath := ExpandConstant('{app}\data\store.db');
    if FileExists(DbPath) then
    begin
      if MsgBox(CustomMessage('KeepDatabasePrompt'), mbConfirmation, MB_YESNO) = IDYES then
      begin
        Log('User chose to preserve store.db database file.');
      end
      else
      begin
        DeleteFile(DbPath);
        RemoveDir(ExpandConstant('{app}\data'));
      end;
    end;
  end;
end;
