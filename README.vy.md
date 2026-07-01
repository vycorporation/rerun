# VY Rerun Fork Helpers

This file is for product-fork workflow notes and helper commands. It intentionally
does not replace the upstream-facing `README.md`.

## Malware Byteplot Example

Run the example directly in the native desktop viewer:

```bash
just malware-desktop
```

Save the example to an `.rrd` file:

```bash
just malware-save
```

Build a local one-card examples manifest, serve it, and launch Rerun so the
Welcome screen shows the `Malware byteplot` card:

```bash
just malware-examples-gui
```

If you want to keep the HTTP server and viewer launch separate:

```bash
just malware-examples-build
just malware-examples-serve
```

Then, in another terminal:

```bash
EXAMPLES_MANIFEST_URL=http://127.0.0.1:8000/examples_manifest.json pixi run rerun
```

The Python SDK in this checkout is not compiled with the `server` feature, so
`python -m malware_byteplot --serve` is not expected to work here. The desktop
path uses the SDK's default native viewer spawn behavior instead.
