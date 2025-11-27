@echo off
REM P3 E2E Test Database Fixture Setup Script
REM This script creates a test database fixture for E2E testing

echo ========================================
echo P3 E2E Test Fixture Setup
echo ========================================
echo.

REM Set paths
set SCRIPT_DIR=%~dp0
set FIXTURE_DB=%SCRIPT_DIR%test-fixture.db
set SQL_SCRIPT=%SCRIPT_DIR%create_test_fixture.sql

REM Check if sqlite3 is available
where sqlite3 >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: sqlite3 not found in PATH
    echo Please install SQLite3 command-line tools
    echo Download from: https://www.sqlite.org/download.html
    exit /b 1
)

REM Remove old fixture if exists
if exist "%FIXTURE_DB%" (
    echo Removing old test fixture...
    del "%FIXTURE_DB%"
)

REM Create new fixture
echo Creating new test fixture database...
sqlite3 "%FIXTURE_DB%" < "%SQL_SCRIPT%"

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ========================================
    echo SUCCESS: Test fixture created!
    echo ========================================
    echo Location: %FIXTURE_DB%
    echo.
    echo Summary:
    sqlite3 "%FIXTURE_DB%" "SELECT COUNT(*) || ' accounts' FROM accounts; SELECT COUNT(*) || ' folders' FROM folders; SELECT COUNT(*) || ' emails' FROM emails; SELECT COUNT(*) || ' attachments' FROM attachments;"
    echo.
    echo You can now use this fixture for E2E tests
    echo ========================================
) else (
    echo.
    echo ERROR: Failed to create test fixture
    exit /b 1
)
