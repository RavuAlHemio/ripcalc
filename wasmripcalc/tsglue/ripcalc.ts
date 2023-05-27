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

    export async function run() {
        const wasmInstance = await WebAssembly.instantiateStreaming(
            fetch("wasmripcalc.wasm"),
            {
                wasm_interop: {
                    append_output: append_output,
                    append_error: append_error,
                },
            },
        );
        exports = <WasmRipcalcExports><unknown>wasmInstance.instance.exports;

        output = "";
        errorOutput = "";

        if (isChecked("section-closer-shownet")) {
            // output subnet
            const subnetField = <HTMLInputElement>document.getElementById("shownet-net");
            strToU16Buffer(subnetField.value);
            exports.ripcalc_show_net();
        } else if (isChecked("section-closer-minimize")) {
            // minimize nets
            const netsField = <HTMLInputElement>document.getElementById("minimize-nets");
            const nets = (
                netsField.value 
                    .split("\n")
                    .map(entry => entry.trim())
                    .filter(entry => entry.length > 0)
                    .join("\n")
            );
            strToU16Buffer(nets);
            exports.ripcalc_minimize();
        } else if (isChecked("section-closer-derange")) {
            const startField = <HTMLInputElement>document.getElementById("derange-start");
            const start = startField.value.replace(/ /g, "");
            const endField = <HTMLInputElement>document.getElementById("derange-end");
            const end = endField.value.replace(/ /g, "");
            strToU16Buffer(`${start} ${end}`);
            exports.ripcalc_derange();
        } else if (isChecked("section-closer-resize")) {
            const networkField = <HTMLInputElement>document.getElementById("resize-network");
            const network = networkField.value.replace(/ /g, "");
            const prefixField = <HTMLInputElement>document.getElementById("resize-prefix");
            const prefix = prefixField.value.replace(/ /g, "");
            strToU16Buffer(`${network} ${prefix}`);
            exports.ripcalc_resize();
        } else if (isChecked("section-closer-enumerate")) {
            const networkField = <HTMLInputElement>document.getElementById("enumerate-network");
            strToU16Buffer(networkField.value);
            exports.ripcalc_enumerate();
        }

        const terminal = document.querySelector("pre.terminal");
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

    document.addEventListener("DOMContentLoaded", () => {
        const doButton = document.getElementById("do-button");
        if (doButton !== null) {
            doButton.addEventListener("click", () => {
                run();
            });
        }
    });
}
