#!/usr/bin/env bash
# Smoke test for lopen-memory
# Run from the repo root after building: cargo build --release
# Usage: ./smoke_test.sh

set -e
BIN="./target/release/lopen-memory"
DB="/tmp/lopen-memory-smoke-test.db"
export LOPEN_MEMORY_DB="$DB"

rm -f "$DB"
echo "=== lopen-memory smoke test ==="
echo

# ── Projects ──────────────────────────────────────────────────────────────────
echo "--- project add ---"
$BIN project add my-app /home/user/my-app "Core application rewrite"
$BIN project add other-app /home/user/other

echo "--- project list ---"
$BIN project list

echo "--- project set-description ---"
$BIN project set-description --project my-app "Updated description"

echo "--- project set-path ---"
$BIN project set-path --project my-app /home/user/new-path

echo "--- project rename ---"
$BIN project rename --project other-app renamed-app

echo "--- project complete / reopen ---"
$BIN project complete --project renamed-app
$BIN project list --completed
$BIN project reopen --project renamed-app

echo "--- project show ---"
$BIN project show --project my-app

# ── Modules ───────────────────────────────────────────────────────────────────
echo "--- module add ---"
$BIN module add --project my-app auth "Handles authentication"
$BIN module add --project my-app payments "Handles payments"

echo "--- module list ---"
$BIN module list --project my-app

echo "--- module set-description ---"
$BIN module set-description --module auth --project my-app "Auth and session management"

echo "--- module set-details ---"
$BIN module set-details --module auth --project my-app "JWT with refresh token rotation"

echo "--- module transition ---"
$BIN module transition --module auth --project my-app Planning
$BIN module transition --module auth --project my-app Building

echo "--- invalid transition (should error) ---"
set +e
$BIN module transition --module auth --project my-app Planning
set -e

echo "--- module show ---"
$BIN module show --module auth --project my-app

# ── Features ──────────────────────────────────────────────────────────────────
echo "--- feature add ---"
$BIN feature add --module auth --project my-app login-flow "User login and session creation"
$BIN feature add --module auth --project my-app token-refresh "Token refresh mechanism"

echo "--- feature list ---"
$BIN feature list --module auth --project my-app

echo "--- feature transition ---"
$BIN feature transition --feature login-flow --module auth Planning
$BIN feature show --feature login-flow --module auth

# ── Tasks ──────────────────────────────────────────────────────────────────────
echo "--- task add ---"
$BIN task add --feature login-flow implement-jwt "Implement JWT issuance"
$BIN task add --feature login-flow write-tests "Write integration tests"

echo "--- task list ---"
$BIN task list --feature login-flow

echo "--- task set-details ---"
$BIN task set-details --task implement-jwt "Use HS256 algorithm with 1h expiry"

echo "--- task transition ---"
$BIN task transition --task implement-jwt --feature login-flow Planning
$BIN task show --task implement-jwt --feature login-flow

# ── Research ──────────────────────────────────────────────────────────────────
echo "--- research add ---"
$BIN research add jwt-rfc "IETF JSON Web Token specification"
$BIN research add oauth2-flows "OAuth2 grant type comparison"

echo "--- research list ---"
$BIN research list

echo "--- research set-content ---"
$BIN research set-content --research jwt-rfc "JWTs are compact, URL-safe tokens. They use base64url encoding and can be signed with HMAC or RSA."

echo "--- research set-source ---"
$BIN research set-source --research jwt-rfc "https://datatracker.ietf.org/doc/html/rfc7519"

echo "--- research set-researched-at ---"
$BIN research set-researched-at --research oauth2-flows 2024-06-01

echo "--- research search ---"
$BIN research search jwt
$BIN research search "no-match-xyz"

echo "--- research link ---"
$BIN research link --research jwt-rfc --project my-app
$BIN research link --research jwt-rfc --module auth
$BIN research link --research jwt-rfc --feature login-flow
$BIN research link --research jwt-rfc --task implement-jwt

echo "--- research links ---"
$BIN research links --research jwt-rfc

echo "--- research show ---"
$BIN research show --research jwt-rfc

echo "--- project show with research ---"
$BIN project show --project my-app

echo "--- research unlink ---"
$BIN research unlink --research jwt-rfc --task implement-jwt
$BIN research links --research jwt-rfc

echo "--- remove task (research survives) ---"
$BIN task remove --task write-tests --feature login-flow
$BIN research show --research jwt-rfc

echo "--- cascade remove feature ---"
set +e
$BIN feature remove --feature login-flow --module auth
set -e
$BIN feature remove --feature login-flow --module auth --cascade

echo "--- remove module ---"
$BIN module remove --module payments --project my-app

echo "--- cascade remove project ---"
$BIN project remove --project renamed-app

echo "--- research still exists after project removal ---"
$BIN research list

echo "--- remove research ---"
$BIN research remove --research jwt-rfc
$BIN research list

echo "--- JSON output test ---"
$BIN --json project list
$BIN --json research list

echo
echo "=== All smoke tests passed ==="
rm -f "$DB"
