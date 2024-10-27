set shell := ["sh", "-c"]
set windows-shell := ["powershell.exe", "-c"]

build:
    wasm-pack build --target web

serve:
    python -m http.server 8000

test:
    wasm-pack test --headless --chrome

watch:
    cargo watch -s "wasm-pack build --target web"

run: build serve

open:
    python -c "import webbrowser; webbrowser.open('http://localhost:8080')"
    python -m http.server 8080

default:
    @just --list