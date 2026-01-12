@echo off
setlocal
echo ==========================================
echo      iSwitch Windows Build Script
echo ==========================================

:: Navigate to script directory
cd /d "%~dp0"

:: Check if Node.js is installed
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Node.js is not installed or not in PATH.
    pause
    exit /b 1
)

:: Check if Rust/Cargo is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Rust/Cargo is not installed or not in PATH.
    pause
    exit /b 1
)

:: Go to tauri project root (parent directory -> iswitch-tauri)
cd ..\iswitch-tauri

echo.
echo [1/2] Installing dependencies...
call npm install
if %errorlevel% neq 0 (
    echo Error: 'npm install' failed.
    pause
    exit /b %errorlevel%
)

echo.
echo [2/2] Building for Windows...
call npm run tauri build
if %errorlevel% neq 0 (
    echo Error: Build command failed.
    pause
    exit /b %errorlevel%
)

echo.
echo ==========================================
echo      Build Completed Successfully!
echo ==========================================
echo.
pause
