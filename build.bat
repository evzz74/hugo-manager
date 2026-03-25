@echo off
echo ========================================
echo Hugo Blog Manager - Build Script
echo ========================================
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Rust/Cargo not found!
    echo Please install Rust from https://rustup.rs/
    echo.
    pause
    exit /b 1
)

echo Rust found. Building project...
echo.

REM Build the project
cargo build --release

if %errorlevel% neq 0 (
    echo.
    echo Build failed! Please check the error messages above.
    pause
    exit /b 1
)

echo.
echo ========================================
echo Build successful!
echo ========================================
echo.
echo The executable is located at:
echo target\release\hugo-manager.exe
echo.
echo To run the program, you can use:
echo   cargo run --release
echo.
pause
