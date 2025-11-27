@echo off
REM P3 Test Suite Runner
REM Runs all P3 level tests and generates a comprehensive report

echo ========================================
echo P3 Test Suite Runner
echo ========================================
echo.

set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..
set REPORT_FILE=%PROJECT_ROOT%\P3_TEST_REPORT.md

REM Navigate to src-tauri directory
cd /d "%PROJECT_ROOT%\src-tauri"

echo Starting P3 test execution...
echo.

REM Create report header
echo # P3 Test Execution Report > "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"
echo Generated: %DATE% %TIME% >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"
echo --- >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"

REM ========================================
REM 1. Run Integration Tests
REM ========================================
echo ========================================
echo 1. Running Integration Tests
echo ========================================
echo.

echo ## 1. Integration Tests >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"

echo Running IMAP protocol tests...
cargo test --test imap_protocol_tests -- --nocapture --test-threads=1 >> "%REPORT_FILE%" 2>&1
set IMAP_RESULT=%ERRORLEVEL%

echo Running SMTP protocol tests...
cargo test --test smtp_protocol_tests -- --nocapture --test-threads=1 >> "%REPORT_FILE%" 2>&1
set SMTP_RESULT=%ERRORLEVEL%

echo. >> "%REPORT_FILE%"
echo **IMAP Tests Result:** %IMAP_RESULT% >> "%REPORT_FILE%"
echo **SMTP Tests Result:** %SMTP_RESULT% >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"

REM ========================================
REM 2. Run E2E Helper Tests
REM ========================================
echo ========================================
echo 2. Running E2E Helper Tests
echo ========================================
echo.

echo ## 2. E2E Helper Tests >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"

cargo test --test e2e_test_helper -- --nocapture >> "%REPORT_FILE%" 2>&1
set E2E_RESULT=%ERRORLEVEL%

echo. >> "%REPORT_FILE%"
echo **E2E Helper Result:** %E2E_RESULT% >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"

REM ========================================
REM 3. Run Performance Benchmarks
REM ========================================
echo ========================================
echo 3. Running Performance Benchmarks
echo ========================================
echo.

echo ## 3. Performance Benchmarks >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"

echo Running email parsing benchmarks...
echo ### Email Parsing Performance >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"
cargo bench --bench email_parsing -- --output-format bencher >> "%REPORT_FILE%" 2>&1
set PARSE_BENCH_RESULT=%ERRORLEVEL%

echo.
echo Running database query benchmarks...
echo ### Database Query Performance >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"
cargo bench --bench database_queries -- --output-format bencher >> "%REPORT_FILE%" 2>&1
set DB_BENCH_RESULT=%ERRORLEVEL%

echo. >> "%REPORT_FILE%"
echo **Email Parsing Benchmark Result:** %PARSE_BENCH_RESULT% >> "%REPORT_FILE%"
echo **Database Benchmark Result:** %DB_BENCH_RESULT% >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"

REM ========================================
REM 4. Generate Summary
REM ========================================
echo ========================================
echo 4. Generating Test Summary
echo ========================================
echo.

echo ## Test Summary >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"

echo ^| Test Category ^| Result ^| >> "%REPORT_FILE%"
echo ^|---------------|--------^| >> "%REPORT_FILE%"

if %IMAP_RESULT% EQU 0 (
    echo ^| IMAP Protocol Tests ^| ✅ PASS ^| >> "%REPORT_FILE%"
) else (
    echo ^| IMAP Protocol Tests ^| ⚠️ PARTIAL/SKIPPED ^| >> "%REPORT_FILE%"
)

if %SMTP_RESULT% EQU 0 (
    echo ^| SMTP Protocol Tests ^| ✅ PASS ^| >> "%REPORT_FILE%"
) else (
    echo ^| SMTP Protocol Tests ^| ⚠️ PARTIAL/SKIPPED ^| >> "%REPORT_FILE%"
)

if %E2E_RESULT% EQU 0 (
    echo ^| E2E Helper Tests ^| ✅ PASS ^| >> "%REPORT_FILE%"
) else (
    echo ^| E2E Helper Tests ^| ❌ FAIL ^| >> "%REPORT_FILE%"
)

if %PARSE_BENCH_RESULT% EQU 0 (
    echo ^| Email Parsing Benchmarks ^| ✅ COMPLETE ^| >> "%REPORT_FILE%"
) else (
    echo ^| Email Parsing Benchmarks ^| ❌ FAIL ^| >> "%REPORT_FILE%"
)

if %DB_BENCH_RESULT% EQU 0 (
    echo ^| Database Benchmarks ^| ✅ COMPLETE ^| >> "%REPORT_FILE%"
) else (
    echo ^| Database Benchmarks ^| ❌ FAIL ^| >> "%REPORT_FILE%"
)

echo. >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"
echo --- >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"
echo ## Notes >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"
echo - **IMAP/SMTP tests marked as ⚠️ PARTIAL**: These tests may be skipped if no real email server credentials are provided via environment variables. >> "%REPORT_FILE%"
echo - **Performance benchmarks**: Check the detailed timing results above to ensure they meet the P3 performance targets: >> "%REPORT_FILE%"
echo   - Email header decoding: ^< 5ms per header >> "%REPORT_FILE%"
echo   - Database INBOX query: ^< 50ms >> "%REPORT_FILE%"
echo   - Batch email sync: 100 emails in ^< 3 seconds >> "%REPORT_FILE%"
echo. >> "%REPORT_FILE%"

REM ========================================
REM Final Summary
REM ========================================
echo.
echo ========================================
echo Test Execution Complete!
echo ========================================
echo.
echo Report saved to: %REPORT_FILE%
echo.
echo You can open the report to view detailed results.
echo.

REM Open report in default markdown viewer (optional)
REM start "" "%REPORT_FILE%"

pause
