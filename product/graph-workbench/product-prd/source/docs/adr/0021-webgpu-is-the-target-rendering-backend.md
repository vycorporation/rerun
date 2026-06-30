# WebGPU is the target rendering backend

Houdini Clone targets WebGPU for high-performance viewport rendering across web and Tauri desktop builds, while keeping a small internal rendering abstraction above raw WebGPU APIs. This preserves the ability to optimize for massive spatial datasets without coupling layers, node outputs, styles, and prepared representations directly to backend-specific GPU details.
