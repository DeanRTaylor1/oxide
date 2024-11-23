#!/bin/bash

# Simple load test
concurrent_requests() {
    local num_requests=$1
    local endpoint=$2

    echo "🔨 Running $num_requests concurrent requests to $endpoint"

    for i in $(seq 1 $num_requests); do
        curl -s "http://localhost:8080$endpoint" > /dev/null &
    done
    wait

    echo "✅ Completed $num_requests requests"
}

# Run 100 concurrent requests to root
concurrent_requests 100 "/"

# Measure response time distribution
ab -n 1000 -c 10 http://localhost:8080/ > ab_results.txt
