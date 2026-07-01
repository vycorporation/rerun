set shell := ["bash", "-uc"]

root := justfile_directory()
example_data := root / "example_data"

# List available commands.
default:
    @just --list

# Run the synthetic malware byteplot example in the native desktop viewer.
malware-desktop:
    @cd "{{root}}" && \
        PROTOC="$(pixi run -q which protoc)" \
        RERUN_ALLOW_MISSING_BIN=1 \
        uv run --package malware-byteplot python -m malware_byteplot

# Save the synthetic malware byteplot example to an RRD.
malware-save output="/tmp/rerun_malware_byteplot.rrd":
    @cd "{{root}}" && \
        rm -f "{{output}}" && \
        PROTOC="$(pixi run -q which protoc)" \
        RERUN_ALLOW_MISSING_BIN=1 \
        uv run --package malware-byteplot python -m malware_byteplot --save "{{output}}" && \
        ls -lh "{{output}}"

# Build a one-card local examples manifest for the malware byteplot example.
malware-examples-build port="8000":
    @cd "{{root}}" && \
        rm -rf "{{example_data}}" && \
        PROTOC="$(pixi run -q which protoc)" \
        RERUN_ALLOW_MISSING_BIN=1 \
        pixi run build-examples rrd "{{example_data}}/examples" --channel main --examples malware_byteplot --install && \
        pixi run build-examples manifest "{{example_data}}/examples_manifest.json" --channel main --examples malware_byteplot --base-url "http://127.0.0.1:{{port}}"

# Serve the local examples manifest. Run `just malware-examples-build` first.
malware-examples-serve port="8000":
    @cd "{{root}}" && python3 -m http.server "{{port}}" --directory "{{example_data}}"

# Build, serve, and launch Rerun with the malware byteplot card in the Welcome screen.
malware-examples-gui port="8000":
    @cd "{{root}}" && \
        just malware-examples-build "{{port}}" && \
        python3 -m http.server "{{port}}" --directory "{{example_data}}" >/tmp/rerun-malware-example-http.log 2>&1 & \
        server_pid=$!; \
        trap 'kill "$server_pid" 2>/dev/null || true' EXIT; \
        sleep 1; \
        EXAMPLES_MANIFEST_URL="http://127.0.0.1:{{port}}/examples_manifest.json" pixi run rerun
