& cargo build --package=wasmripcalc --target=wasm32-unknown-unknown --release
Copy-Item -Path .\target\wasm32-unknown-unknown\release\wasmripcalc.wasm .\wasmripcalc\tsglue\wasmripcalc.wasm
Write-Output (Get-Item .\wasmripcalc\tsglue\wasmripcalc.wasm).Length
