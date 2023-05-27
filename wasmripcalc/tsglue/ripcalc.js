var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var Ripcalc;
(function (Ripcalc) {
    var maxBufferSize = 1024;
    var exports;
    var output = "";
    var errorOutput = "";
    function getBuffer() {
        var bufferSizeOffset = exports.ripcalc_get_buffer_size_offset();
        var bufferSizeArray = new Uint32Array(exports.memory.buffer, bufferSizeOffset, 1);
        var bufferSize = bufferSizeArray[0];
        var bufferOffset = exports.ripcalc_get_buffer_offset();
        var bufferArray = new Uint16Array(exports.memory.buffer, bufferOffset, bufferSize);
        return bufferArray;
    }
    function getBufferSettingSize(newSize) {
        if (newSize > maxBufferSize) {
            throw new Error("maximum buffer size is ".concat(maxBufferSize, "; cannot extend buffer to ").concat(newSize));
        }
        var bufferSizeOffset = exports.ripcalc_get_buffer_size_offset();
        var bufferSizeArray = new Uint32Array(exports.memory.buffer, bufferSizeOffset, 1);
        bufferSizeArray[0] = newSize;
        var bufferOffset = exports.ripcalc_get_buffer_offset();
        var bufferArray = new Uint16Array(exports.memory.buffer, bufferOffset, newSize);
        return bufferArray;
    }
    function append_output() {
        var array = getBuffer();
        var str = "";
        for (var i = 0; i < array.length; i++) {
            str += String.fromCharCode(array[i]);
        }
        output += str;
    }
    function append_error() {
        var array = getBuffer();
        var str = "";
        for (var i = 0; i < array.length; i++) {
            str += String.fromCharCode(array[i]);
        }
        errorOutput += str;
    }
    function strToU16Buffer(str) {
        var bufferArray = getBufferSettingSize(str.length);
        for (var i = 0; i < str.length; i++) {
            bufferArray[i] = str.charCodeAt(i);
        }
    }
    function isChecked(id) {
        var checkbox = document.getElementById(id);
        return (checkbox === null) ? false : checkbox.checked;
    }
    function run() {
        return __awaiter(this, void 0, void 0, function () {
            var wasmInstance, subnetField, netsField, nets, startField, start, endField, end, networkField, network, prefixField, prefix, networkField, terminal, redSpan;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, WebAssembly.instantiateStreaming(fetch("wasmripcalc.wasm"), {
                            wasm_interop: {
                                append_output: append_output,
                                append_error: append_error,
                            },
                        })];
                    case 1:
                        wasmInstance = _a.sent();
                        exports = wasmInstance.instance.exports;
                        output = "";
                        errorOutput = "";
                        if (isChecked("section-closer-shownet")) {
                            subnetField = document.getElementById("shownet-net");
                            strToU16Buffer(subnetField.value);
                            exports.ripcalc_show_net();
                        }
                        else if (isChecked("section-closer-minimize")) {
                            netsField = document.getElementById("minimize-nets");
                            nets = (netsField.value
                                .split("\n")
                                .map(function (entry) { return entry.trim(); })
                                .filter(function (entry) { return entry.length > 0; })
                                .join("\n"));
                            strToU16Buffer(nets);
                            exports.ripcalc_minimize();
                        }
                        else if (isChecked("section-closer-derange")) {
                            startField = document.getElementById("derange-start");
                            start = startField.value.replace(/ /g, "");
                            endField = document.getElementById("derange-end");
                            end = endField.value.replace(/ /g, "");
                            strToU16Buffer("".concat(start, " ").concat(end));
                            exports.ripcalc_derange();
                        }
                        else if (isChecked("section-closer-resize")) {
                            networkField = document.getElementById("resize-network");
                            network = networkField.value.replace(/ /g, "");
                            prefixField = document.getElementById("resize-prefix");
                            prefix = prefixField.value.replace(/ /g, "");
                            strToU16Buffer("".concat(network, " ").concat(prefix));
                            exports.ripcalc_resize();
                        }
                        else if (isChecked("section-closer-enumerate")) {
                            networkField = document.getElementById("enumerate-network");
                            strToU16Buffer(networkField.value);
                            exports.ripcalc_enumerate();
                        }
                        terminal = document.querySelector("pre.terminal");
                        if (errorOutput.length > 0) {
                            redSpan = document.createElement("span");
                            redSpan.classList.add("color");
                            redSpan.classList.add("color-red");
                            redSpan.classList.add("stderr");
                            redSpan.textContent = errorOutput;
                            while (terminal.firstChild !== null) {
                                terminal.firstChild.remove();
                            }
                            terminal.appendChild(redSpan);
                        }
                        else {
                            terminal.innerHTML = output;
                        }
                        return [2 /*return*/];
                }
            });
        });
    }
    Ripcalc.run = run;
    document.addEventListener("DOMContentLoaded", function () {
        document.getElementById("do-button").addEventListener("click", function () {
            run();
        });
    });
})(Ripcalc || (Ripcalc = {}));
//# sourceMappingURL=ripcalc.js.map