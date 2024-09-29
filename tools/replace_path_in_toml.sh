#!/bin/bash

# Ensure that an argument is provided
if [ -z "$1" ]; then
  echo "Usage: $0 <version|reset>"
  exit 1
fi

# The version to replace the path with (only used when setting version)
VERSION=$1

# Function to update dependencies to use version
update_to_version() {
  local file="$1"
  sed -i.bak -E "s|path *= *\"[^\"]+\"|version = \"$VERSION\"|g" "$file"
  echo "Updated $file to use version $VERSION"
}

# Function to update dependencies to use path
update_to_path() {
  local file="$1"
  
  # Read the package name from the file to construct the path
  local package_name
  package_name=$(grep '^name = "' "$file" | sed -E 's/name = "(.*)"/\1/')

  # Construct the path based on the package name
  local path="../$package_name"

  # Update the version to path
  sed -i.bak -E "s|version *= *\"[0-9]+\.[0-9]+\.[0-9]+\"|path = \"$path\"|g" "$file"
  echo "Updated $file to use path $path"
}

# Find all Cargo.toml files and process each one
find . -name "Cargo.toml" | while read -r file; do
  if grep -q 'path *= *"' "$file"; then
    # If path exists, replace with version
    update_to_version "$file"
  elif grep -q 'version *= *"' "$file"; then
    # If version exists, replace with path
    update_to_path "$file"
  else
    echo "No path or version found in $file"
  fi

  # Optionally, remove the backup files created by sed
  rm "$file.bak"
done

echo "Replacement complete."
