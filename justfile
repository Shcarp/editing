build:
    wasm-pack build --target web

serve:
    python -m http.server 8000

test:
    wasm-pack test --headless --chrome

watch:
    cargo watch -s "wasm-pack build --target web"

run: build serve

open: build
    python -c "import webbrowser; webbrowser.open('http://localhost:8000')"
    python -m http.server 8000

default:
    @just --list