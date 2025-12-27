#!/usr/bin/env zsh
# Test script for WASM modules
# Run this after starting the server with: cargo run

BASE_URL="http://localhost:3000"

# Colors
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

submit_job() {
    local module_id=$1
    shift
    local capabilities=("$@")
    
    # Build capabilities array JSON
    local cap_json="[]"
    if [ ${#capabilities[@]} -gt 0 ]; then
        cap_json=$(printf '%s\n' "${capabilities[@]}" | jq -R . | jq -s .)
    fi
    
    # Build request body
    local body=$(jq -n \
        --arg tenant_id "test-tenant" \
        --arg module_id "$module_id" \
        --argjson capabilities "$cap_json" \
        '{tenant_id: $tenant_id, module_id: $module_id, payload: {}, capabilities: $capabilities}')
    
    # Submit job and extract job_id
    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$body" \
        "$BASE_URL/jobs")
    
    echo "$response" | jq -r '.job_id'
}

get_job_result() {
    local job_id=$1
    sleep 1
    curl -s -X GET "$BASE_URL/jobs/$job_id"
}

get_output_string() {
    local result=$1
    
    # Check if result has output field
    local output_json=$(echo "$result" | jq -r '.result.output // empty')
    
    if [ -n "$output_json" ] && [ "$output_json" != "null" ]; then
        # Convert byte array to string
        # Use process substitution to avoid subshell issues
        local output=""
        while IFS= read -r byte; do
            if [ -n "$byte" ] && [ "$byte" != "" ]; then
                output="${output}$(printf "\\$(printf "%03o" "$byte")")"
            fi
        done < <(echo "$result" | jq -r '.result.output[]')
        echo "$output"
    else
        echo "N/A"
    fi
}

echo -e "${CYAN}=== Testing WASM Modules ===${NC}"
echo ""

# Test 1: Simple compute (no capabilities) - should return 60
echo -e "${YELLOW}1. Testing simple-compute (no capabilities)...${NC}"
if job_id=$(submit_job "simple-compute"); then
    echo -e "${GRAY}   Job ID: $job_id${NC}"
    result=$(get_job_result "$job_id")
    finished=$(echo "$result" | jq -r '.status.finished // empty')
    if [ -n "$finished" ] && [ "$finished" != "null" ]; then
        output=$(get_output_string "$result")
        echo -e "${GREEN}   Result: $output (expected: 60)${NC}"
    else
        failed=$(echo "$result" | jq -r '.status.failed // "Unknown error"')
        echo -e "${RED}   FAILED: $failed${NC}"
    fi
else
    echo -e "${RED}   ERROR: Failed to submit job${NC}"
fi
echo ""

# Test 2: GPU compute (with gpu.compute capability) - should return 42
echo -e "${YELLOW}2. Testing gpu-compute (with gpu.compute capability)...${NC}"
if job_id=$(submit_job "gpu-compute" "gpu.compute"); then
    echo -e "${GRAY}   Job ID: $job_id${NC}"
    result=$(get_job_result "$job_id")
    finished=$(echo "$result" | jq -r '.status.finished // empty')
    if [ -n "$finished" ] && [ "$finished" != "null" ]; then
        output=$(get_output_string "$result")
        echo -e "${GREEN}   Result: $output (expected: 42)${NC}"
    else
        failed=$(echo "$result" | jq -r '.status.failed // "Unknown error"')
        echo -e "${RED}   FAILED: $failed${NC}"
    fi
else
    echo -e "${RED}   ERROR: Failed to submit job${NC}"
fi
echo ""

# Test 3: Logging test (with logging capability) - should return 100
echo -e "${YELLOW}3. Testing logging-test (with logging capability)...${NC}"
if job_id=$(submit_job "logging-test" "logging"); then
    echo -e "${GRAY}   Job ID: $job_id${NC}"
    result=$(get_job_result "$job_id")
    finished=$(echo "$result" | jq -r '.status.finished // empty')
    if [ -n "$finished" ] && [ "$finished" != "null" ]; then
        output=$(get_output_string "$result")
        echo -e "${GREEN}   Result: $output (expected: 100)${NC}"
    else
        failed=$(echo "$result" | jq -r '.status.failed // "Unknown error"')
        echo -e "${RED}   FAILED: $failed${NC}"
    fi
else
    echo -e "${RED}   ERROR: Failed to submit job${NC}"
fi
echo ""

# Test 4: GPU compute WITHOUT capability (should fail)
echo -e "${YELLOW}4. Testing gpu-compute WITHOUT capability (should fail)...${NC}"
if job_id=$(submit_job "gpu-compute"); then
    echo -e "${GRAY}   Job ID: $job_id${NC}"
    result=$(get_job_result "$job_id")
    failed=$(echo "$result" | jq -r '.status.failed // empty')
    if [ -n "$failed" ] && [ "$failed" != "null" ]; then
        echo -e "${GREEN}   Correctly failed (expected behavior)${NC}"
    else
        echo -e "${RED}   Unexpected success${NC}"
    fi
else
    echo -e "${RED}   ERROR: Failed to submit job${NC}"
fi
echo ""

# Test 5: Ultra simple (no capabilities) - should return 42
echo -e "${YELLOW}5. Testing ultra-simple (no capabilities)...${NC}"
if job_id=$(submit_job "ultra-simple"); then
    echo -e "${GRAY}   Job ID: $job_id${NC}"
    result=$(get_job_result "$job_id")
    finished=$(echo "$result" | jq -r '.status.finished // empty')
    if [ -n "$finished" ] && [ "$finished" != "null" ]; then
        output=$(get_output_string "$result")
        echo -e "${GREEN}   Result: $output (expected: 42)${NC}"
    else
        failed=$(echo "$result" | jq -r '.status.failed // "Unknown error"')
        echo -e "${RED}   FAILED: $failed${NC}"
    fi
else
    echo -e "${RED}   ERROR: Failed to submit job${NC}"
fi
echo ""

echo -e "${CYAN}=== All tests complete ===${NC}"

