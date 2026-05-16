@echo off
title EDCAS - Elite Dangerous Commander Assistant System
edcas-client.exe
if %errorlevel% neq 0 (
    echo.
    echo EDCAS exited with an error. Check that a terminal supporting ANSI colours
    echo is available ^(Windows Terminal or PowerShell 7+ recommended^).
    echo.
    pause
)
