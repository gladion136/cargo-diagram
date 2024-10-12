#!/bin/sh

cargo r --bin cargo-diagram -- -o ./docs/assets/overview.puml
plantuml ./docs/assets/overview.puml -tsvg
plantuml ./docs/assets/overview.puml

cargo r --bin cargo-diagram -- -r -o ./docs/assets/overview_detailed.puml
plantuml ./docs/assets/overview_detailed.puml -tsvg
plantuml ./docs/assets/overview_detailed.puml
