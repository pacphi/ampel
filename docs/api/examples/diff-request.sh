#!/bin/bash
# Git Diff API Request Examples

# Set your JWT token
TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
API_BASE="http://localhost:8080/api/v1"
PR_ID="550e8400-e29b-41d4-a716-446655440000"

echo "=== Example 1: Basic Diff Request ==="
curl -X GET \
  "${API_BASE}/pull-requests/${PR_ID}/diff" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Accept: application/json" \
  | jq '.'

echo -e "\n=== Example 2: Split View with Extended Context ==="
curl -X GET \
  "${API_BASE}/pull-requests/${PR_ID}/diff?format=split&context=5" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Accept: application/json" \
  | jq '.data.files[] | {path: .new_path, status: .status, changes: .changes}'

echo -e "\n=== Example 3: Bypass Cache ==="
curl -X GET \
  "${API_BASE}/pull-requests/${PR_ID}/diff" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Cache-Control: no-cache" \
  -H "Accept: application/json" \
  | jq '.metadata'

echo -e "\n=== Example 4: Check Rate Limits ==="
curl -X GET \
  "${API_BASE}/pull-requests/${PR_ID}/diff" \
  -H "Authorization: Bearer ${TOKEN}" \
  -I | grep -i "x-ratelimit"

echo -e "\n=== Example 5: Error Handling (Invalid Format) ==="
curl -X GET \
  "${API_BASE}/pull-requests/${PR_ID}/diff?format=invalid" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Accept: application/json" \
  | jq '.error'

echo -e "\n=== Example 6: Measure Response Time ==="
time curl -X GET \
  "${API_BASE}/pull-requests/${PR_ID}/diff" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Accept: application/json" \
  -o /dev/null -s -w "HTTP %{http_code} - %{time_total}s\n"
