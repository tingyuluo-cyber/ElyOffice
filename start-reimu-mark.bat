@echo off
setlocal DisableDelayedExpansion
pushd "%~dp0" || exit /b 1

set "ELY_MARK_REIMU=1"
set "ELY_DATA_ROOT=%LOCALAPPDATA%\ElyMarkReimu"
set "CARGO_PROFILE_DEV_DEBUG=0"
set "CARGO_BUILD_BUILD_DIR=%~dp0target"

rem Reuse the matching worker from the installed release for PDF export.
if not defined ELY_PDF_WORKER if exist "target\debug\ely-pdf-worker.exe" set "ELY_PDF_WORKER=%~dp0target\debug\ely-pdf-worker.exe"
if not defined ELY_PDF_WORKER for /f "tokens=2,*" %%A in ('reg query "HKCU\Software\Ely Office\ElyMark" /v InstallDir 2^>nul') do (
    if "%%A"=="REG_SZ" if exist "%%B\ely-pdf-worker.exe" set "ELY_PDF_WORKER=%%B\ely-pdf-worker.exe"
)

cargo run --locked --package ely-mark --bin ely-mark --target-dir target\reimu -- %*
set "reimu_exit=%ERRORLEVEL%"
popd
if not "%reimu_exit%"=="0" if not defined ELY_REIMU_NO_PAUSE pause
exit /b %reimu_exit%
