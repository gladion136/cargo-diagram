# Use the official Rust DevContainer as the base image
FROM mcr.microsoft.com/vscode/devcontainers/rust:latest

# Install dependencies
RUN apt-get update && apt-get install -y \
    plantuml \
    openjdk-11-jre-headless \
    graphviz \
    && apt-get clean

# Set up PlantUML by downloading the latest jar version
RUN mkdir -p /usr/local/bin/plantuml \
    && curl -L https://github.com/plantuml/plantuml/releases/download/v1.2023.7/plantuml-1.2023.7.jar -o /usr/local/bin/plantuml/plantuml.jar

# Create an easy command to run PlantUML
RUN echo '#!/bin/bash\njava -jar /usr/local/bin/plantuml/plantuml.jar "$@"' > /usr/local/bin/plantuml && chmod +x /usr/local/bin/plantuml

# Set user to vscode
USER vscode
