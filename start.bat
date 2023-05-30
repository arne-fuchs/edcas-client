@echo off
setlocal
cd /d "%~dp0"
rem Sleep to wait until elite starts
timeout /t 60 /nobreak >nul
cargo run