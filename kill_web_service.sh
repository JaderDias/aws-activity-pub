#!/bin/bash

echo "Trying to kill web_service process..."

# Function to check if the process is still running
is_process_running() {
  pgrep -x "web_service" > /dev/null
}

# Function to kill the process
kill_process() {
  pkill -x "web_service"
}

# Try to kill the process
kill_process

# Check if the process is still running, and try to kill it again
while is_process_running; do
  echo "web_service is still running, trying to kill it again..."
  kill_process
  sleep 1
done

echo "web_service process killed successfully."
