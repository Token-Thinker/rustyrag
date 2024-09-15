#!/bin/bash

# Find the directory where the script is located and go one level up
SCRIPT_DIR=$(dirname "$(realpath "$0")")
PROJECTS_ROOT=$(realpath "$SCRIPT_DIR/..")

# Set the default value for embedding (true or false)
EMBEDDING=true

# Parse command-line options
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --no-embedding) EMBEDDING=false ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

# List available projects
echo "Available projects:"
select project in "$PROJECTS_ROOT"/*; do
    if [ -n "$project" ]; then
        # Extract project name from the selected path
        project_name=$(basename "$project")

        echo "Selected project: $project"
        # Set the COLLECTION environment variable and run the shuttle app
        COLLECTION="$project_name" PROJECT_DIR="$project" EMBEDDING="$EMBEDDING" cargo shuttle run
        break
    else
        echo "Invalid selection"
    fi
done