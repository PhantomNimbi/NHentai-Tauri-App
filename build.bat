@echo off
setlocal

set "root_dir=%~dp0"
set "log_dir=%root_dir%logs"
if not exist "%log_dir%" mkdir "%log_dir%"
for /f %%i in ('powershell -NoProfile -Command "Get-Date -Format yyyyMMdd-HHmmss"') do set "TIMESTAMP=%%i"
set "platform=windows"
set "LOG_FILE=%log_dir%\build-%platform%-%TIMESTAMP%.log"

cd /d "%~dp0\src-tauri"

if "%~1"=="" goto run
if /I "%~1"=="help" goto usage
if /I "%~1"=="-h" goto usage
if /I "%~1"=="--help" goto usage

if /I "%~1"=="windows" goto platform_windows
if /I "%~1"=="linux" goto platform_linux
if /I "%~1"=="macos" goto platform_macos
if /I "%~1"=="android" goto platform_android
if /I "%~1"=="ios" goto platform_ios

goto run

:platform_windows
shift
if "%~1"=="" goto run
if /I "%~1"=="help" goto usage
if /I "%~1"=="-h" goto usage
if /I "%~1"=="--help" goto usage
set "args=%*"
goto run

:platform_linux
echo The command "build linux" is supported by %~nx0.
echo Use a Linux host and run "./build.sh linux --release" to build Linux.
echo On Windows, this command only shows guidance, it does not build Linux.
goto end

:platform_macos
echo The command "build macos" is supported by %~nx0.
echo Use a macOS host and run "./build.sh macos --release" to build macOS.
echo On Windows, this command only shows guidance, it does not build macOS.
goto end

:platform_android
echo The command "build android" is supported by %~nx0.
echo Use a host with Android SDK installed and run "./build.sh android --release" to build Android.
echo On Windows, this command only shows guidance unless the Android toolchain is installed.
goto end

:platform_ios
echo The command "build ios" is supported by %~nx0.
echo Use a macOS host and run "./build.sh ios --release" to build iOS.
echo On Windows, this command only shows guidance, it does not build iOS.
goto end

:run
echo Building Windows desktop app...
echo Logging output to %LOG_FILE%
powershell -NoProfile -Command "Set-Location '%cd%'; & cargo tauri build %args% 2>&1 | Tee-Object -FilePath '%LOG_FILE%'"
if errorlevel 1 (
  echo Build failed. See %LOG_FILE%
  goto end
)
echo Build succeeded. Log: %LOG_FILE%
goto end

:usage
echo Usage: build [platform] [cargo-tauri-args]
echo.
echo Platform support:
echo   windows   Build Windows desktop app
echo   linux     Print instructions for Linux build host
echo   macos     Print instructions for macOS build host
echo   android   Print instructions for Android build host
echo   ios       Print instructions for iOS build host
echo.
echo Examples:
echo   build --release
echo   build windows --release
echo   build help
goto end

:end
endlocal
