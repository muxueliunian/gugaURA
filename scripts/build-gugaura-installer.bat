@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
for %%I in ("%SCRIPT_DIR%..") do set "REPO_ROOT=%%~fI"
set "TAURI_DIR=%REPO_ROOT%\guga_ura_config_tauri"
set "INSTALLER_PATH=%REPO_ROOT%\target\release\bundle\nsis\gugaURA_installer.exe"

where pnpm >nul 2>nul
if errorlevel 1 (
    echo [ERROR] pnpm not found in PATH.
    exit /b 1
)

where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERROR] cargo not found in PATH.
    exit /b 1
)

echo [STEP] Entering %TAURI_DIR%
pushd "%TAURI_DIR%" || exit /b 1

echo [STEP] Syncing frontend dependencies...
call pnpm install
if errorlevel 1 goto :fail

echo [STEP] Building frontend, backend and NSIS installer...
call pnpm build:windows-installer
if errorlevel 1 goto :fail

popd

if not exist "%INSTALLER_PATH%" (
    echo [ERROR] Installer not found: %INSTALLER_PATH%
    exit /b 1
)

echo [DONE] Installer ready:
echo %INSTALLER_PATH%
exit /b 0

:fail
set "EXIT_CODE=%ERRORLEVEL%"
popd
echo [ERROR] Build failed with exit code %EXIT_CODE%.
exit /b %EXIT_CODE%
