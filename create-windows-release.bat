@echo off

setlocal

set "folder_name=edcas-client-windows"

rem Clean build artifacts
cargo clean

rem Update dependencies
cargo update

rem Build release version
cargo build --release

rem Check if the folder exists
if exist "%folder_name%" (
    echo Folder '%folder_name%' found. Removing...
    rem Remove the folder
    rmdir /s /q "%folder_name%"
    echo Folder removed.
)

rem Create the folder and subdirectories
mkdir "%folder_name%"
mkdir "%folder_name%\logs"

rem Copy files and directories
copy settings-example.json "%folder_name%\settings-example.json"
copy settings-example.json "%folder_name%\settings.json"
copy materials.json "%folder_name%\materials.json"
copy start.bat "%folder_name%\"
copy target\release\edcas-client.exe "%folder_name%\"

rem Create ZIP archive
"%ProgramFiles%\7-Zip\7z.exe" a -tzip "%folder_name%_windows.zip" "%folder_name%\*"

endlocal