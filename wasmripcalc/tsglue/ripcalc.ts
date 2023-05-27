namespace Ripcalc {
    const maxBufferSize = 1024;
    interface WasmRipcalcExports {
        memory: WebAssembly.Memory;
        ripcalc_get_buffer_size_offset: () => number;
        ripcalc_get_buffer_offset: () => number;
        ripcalc_show_net: () => void;
        ripcalc_minimize: () => void;
        ripcalc_derange: () => void;
        ripcalc_resize: () => void;
        ripcalc_enumerate: () => void;
    }
    let wasmInstance: WebAssembly.WebAssemblyInstantiatedSource;
    let exports: WasmRipcalcExports;
    let output: string = "";
    let errorOutput: string = "";

    function getBuffer(): Uint16Array {
        const bufferSizeOffset = exports.ripcalc_get_buffer_size_offset();
        const bufferSizeArray = new Uint32Array(exports.memory.buffer, bufferSizeOffset, 1);
        const bufferSize = bufferSizeArray[0];

        const bufferOffset = exports.ripcalc_get_buffer_offset();
        const bufferArray = new Uint16Array(exports.memory.buffer, bufferOffset, bufferSize);
        return bufferArray;
    }

    function getBufferSettingSize(newSize: number): Uint16Array {
        if (newSize > maxBufferSize) {
            throw new Error(`maximum buffer size is ${maxBufferSize}; cannot extend buffer to ${newSize}`);
        }

        const bufferSizeOffset = exports.ripcalc_get_buffer_size_offset();
        const bufferSizeArray = new Uint32Array(exports.memory.buffer, bufferSizeOffset, 1);
        bufferSizeArray[0] = newSize;

        const bufferOffset = exports.ripcalc_get_buffer_offset();
        const bufferArray = new Uint16Array(exports.memory.buffer, bufferOffset, newSize);
        return bufferArray;
    }

    function append_output() {
        const array = getBuffer();
        let str = "";
        for (let i = 0; i < array.length; i++) {
            str += String.fromCharCode(array[i]!);
        }
        output += str;
    }

    function append_error() {
        const array = getBuffer();
        let str = "";
        for (let i = 0; i < array.length; i++) {
            str += String.fromCharCode(array[i]!);
        }
        errorOutput += str;
    }

    function strToU16Buffer(str: string) {
        const bufferArray = getBufferSettingSize(str.length);
        for (let i = 0; i < str.length; i++) {
            bufferArray[i] = str.charCodeAt(i);
        }
    }

    function isChecked(id: string): boolean {
        const checkbox = <HTMLInputElement|null>document.getElementById(id);
        return (checkbox === null) ? false : checkbox.checked;
    }

    async function obtainWasmInstance() {
        wasmInstance = await WebAssembly.instantiateStreaming(
            fetch("wasmripcalc.wasm"),
            {
                wasm_interop: {
                    append_output: append_output,
                    append_error: append_error,
                },
            },
        );
        exports = <WasmRipcalcExports><unknown>wasmInstance.instance.exports;
    }

    function runRipcalc() {
        output = "";
        errorOutput = "";

        if (isChecked("section-closer-shownet")) {
            // output subnet
            const subnetField = <HTMLInputElement|null>document.getElementById("shownet-net");
            if (subnetField !== null) {
                strToU16Buffer(subnetField.value);
                exports.ripcalc_show_net();
            }
        } else if (isChecked("section-closer-minimize")) {
            // minimize nets
            const netsField = <HTMLInputElement|null>document.getElementById("minimize-nets");
            if (netsField !== null) {
                const nets = (
                    netsField.value
                        .split("\n")
                        .map(entry => entry.trim())
                        .filter(entry => entry.length > 0)
                        .join("\n")
                );
                strToU16Buffer(nets);
                exports.ripcalc_minimize();
            }
        } else if (isChecked("section-closer-derange")) {
            const startField = <HTMLInputElement|null>document.getElementById("derange-start");
            const endField = <HTMLInputElement|null>document.getElementById("derange-end");
            if (startField !== null && endField !== null) {
                const start = startField.value.replace(/ /g, "");
                const end = endField.value.replace(/ /g, "");
                strToU16Buffer(`${start} ${end}`);
                exports.ripcalc_derange();
            }
        } else if (isChecked("section-closer-resize")) {
            const networkField = <HTMLInputElement|null>document.getElementById("resize-network");
            const prefixField = <HTMLInputElement|null>document.getElementById("resize-prefix");
            if (networkField !== null && prefixField !== null) {
                const network = networkField.value.replace(/ /g, "");
                const prefix = prefixField.value.replace(/ /g, "");
                strToU16Buffer(`${network} ${prefix}`);
                exports.ripcalc_resize();
            }
        } else if (isChecked("section-closer-enumerate")) {
            const networkField = <HTMLInputElement|null>document.getElementById("enumerate-network");
            if (networkField !== null) {
                strToU16Buffer(networkField.value);
                exports.ripcalc_enumerate();
            }
        }

        const terminal = <HTMLPreElement|null>document.querySelector("pre.terminal");
        if (terminal !== null) {
            if (errorOutput.length > 0) {
                const redSpan = document.createElement("span");
                redSpan.classList.add("color");
                redSpan.classList.add("color-red");
                redSpan.classList.add("stderr");
                redSpan.textContent = errorOutput;

                while (terminal.firstChild !== null) {
                    terminal.firstChild.remove();
                }
                terminal.appendChild(redSpan);
            } else {
                terminal.innerHTML = output;
            }
        }
    }

    async function setUp() {
        await obtainWasmInstance();
        const form = <HTMLFormElement|null>document.getElementById("ripcalc-form");
        if (form !== null) {
            form.addEventListener("submit", (event) => {
                event.preventDefault();
                runRipcalc();
            });
        }
        const terminal = <HTMLPreElement|null>document.querySelector("pre.terminal");
        if (terminal !== null) {
            terminal.textContent = "ready";
        }
    }

    document.addEventListener("DOMContentLoaded", () => {
        setUp();
    });
}
