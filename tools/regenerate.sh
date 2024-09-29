#!/bin/sh

cargo r --bin cargo-diagram
plantuml overview.puml -tsvg
plantuml overview.puml
