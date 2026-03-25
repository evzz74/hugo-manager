@echo off
echo Starting Hugo Blog Manager...
echo.

REM Check if release build exists
if exist "target\release\hugo-manager.exe" (
    echo Running release build...
    target\release\hugo-manager.exe
) else (
    echo Release build not found. Running debug build with cargo...
    cargo run
)
