#!/bin/bash
# P3 E2E Test Database Fixture Setup Script
# This script creates a test database fixture for E2E testing

echo "========================================"
echo "P3 E2E Test Fixture Setup"
echo "========================================"
echo ""

# Set paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURE_DB="$SCRIPT_DIR/test-fixture.db"
SQL_SCRIPT="$SCRIPT_DIR/create_test_fixture.sql"

# Check if sqlite3 is available
if ! command -v sqlite3 &> /dev/null; then
    echo "ERROR: sqlite3 not found"
    echo "Please install SQLite3:"
    echo "  Ubuntu/Debian: sudo apt-get install sqlite3"
    echo "  macOS: brew install sqlite3"
    exit 1
fi

# Remove old fixture if exists
if [ -f "$FIXTURE_DB" ]; then
    echo "Removing old test fixture..."
    rm "$FIXTURE_DB"
fi

# Create new fixture
echo "Creating new test fixture database..."
sqlite3 "$FIXTURE_DB" < "$SQL_SCRIPT"

if [ $? -eq 0 ]; then
    echo ""
    echo "========================================"
    echo "SUCCESS: Test fixture created!"
    echo "========================================"
    echo "Location: $FIXTURE_DB"
    echo ""
    echo "Summary:"
    sqlite3 "$FIXTURE_DB" "SELECT COUNT(*) || ' accounts' FROM accounts; SELECT COUNT(*) || ' folders' FROM folders; SELECT COUNT(*) || ' emails' FROM emails; SELECT COUNT(*) || ' attachments' FROM attachments;"
    echo ""
    echo "You can now use this fixture for E2E tests"
    echo "========================================"
else
    echo ""
    echo "ERROR: Failed to create test fixture"
    exit 1
fi
