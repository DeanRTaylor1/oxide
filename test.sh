#!/bin/bash

HOST="localhost"
PORT="8080"
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

test_count=0
pass_count=0

test_request() {
    local method=$1
    local endpoint=$2
    local expected_status=$3
    local data=$4
    local cookies=$5
    local description=$6

    test_count=$((test_count + 1))
    echo -e "\n🔍 Testing: ${description}"

    local headers=""

    if [ -n "$cookies" ]; then
        headers="$headers --cookie '$cookies'"
    fi

    if [ -n "$data" ]; then
        response=$(curl -i -s ${headers} -X $method \
            -H "Content-Type: application/json" \
            -d "$data" \
            "http://$HOST:$PORT$endpoint")
    else
        response=$(curl -i -s ${headers} -X $method \
            "http://$HOST:$PORT$endpoint")
    fi

    http_status=$(echo "$response" | grep "HTTP/" | awk '{print $2}')
    body=$(echo "$response" | awk 'BEGIN{RS="\r\n\r\n";ORS=""}NR==2{print}')

    if [ "$http_status" -eq "$expected_status" ]; then
        echo -e "${GREEN}✓ PASS${NC} - Status: $http_status"
        pass_count=$((pass_count + 1))
    else
        echo -e "${RED}✗ FAIL${NC} - Expected: $expected_status, Got: $http_status"
    fi
    echo "Response: $body"
}

echo "🚀 Starting HTTP Server Tests"
echo "=============================="

# Basic GET request
test_request "GET" "/api" 200 "" "Basic GET request to root"

test_request "GET" "/user/10" 200 "10" "Basic GET request to user endpoint"

# Non-existent endpoint
test_request "GET" "/notfound" 404 "" "GET request to non-existent endpoint"

# POST request with JSON data
test_request "POST" "/api" 201 '{"message":"test"}' "POST request with JSON payload"

# PUT request
test_request "PUT" "/api/data/1" 201 '{"name":"updated"}' "PUT request with JSON payload"

# DELETE request
test_request "DELETE" "/api/data/1" 200 "" "DELETE request"

# Bad request test
test_request "POST" "/api" 400 'invalid-json' "POST request with invalid JSON"

# # Test CORS preflight
# test_request "OPTIONS" "/" 200 "" "" "OPTIONS request for CORS"

test_request "GET" "/cookies" 200 "" "session=abc123" "Test reading cookies"
test_request "GET" "/cookies" 200 "" "session=abc123; user=dean" "Test multiple cookies"

echo -e "\n📊 Test Summary"
echo "================="
echo "Total Tests: $test_count"
echo "Passed: $pass_count"
echo "Failed: $((test_count - pass_count))"

if [ $test_count -eq $pass_count ]; then
    echo -e "\n${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed${NC}"
    exit 1
fi
