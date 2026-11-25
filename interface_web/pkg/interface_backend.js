/* @ts-self-types="./interface_backend.d.ts" */

export class JSCommandPacket {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JSCommandPacketFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jscommandpacket_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get fan_speed() {
        const ret = wasm.__wbg_get_jscommandpacket_fan_speed(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get purge_interval() {
        const ret = wasm.__wbg_get_jscommandpacket_purge_interval(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get purge_time() {
        const ret = wasm.__wbg_get_jscommandpacket_purge_time(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get restart() {
        const ret = wasm.__wbg_get_jscommandpacket_restart(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get send_wait() {
        const ret = wasm.__wbg_get_jscommandpacket_send_wait(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint8Array}
     */
    get connect_to_mac() {
        const ret = wasm.jscommandpacket_connect_to_mac(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    constructor() {
        const ret = wasm.jscommandpacket_new();
        this.__wbg_ptr = ret;
        JSCommandPacketFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} arg0
     */
    set fan_speed(arg0) {
        wasm.__wbg_set_jscommandpacket_fan_speed(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set purge_interval(arg0) {
        wasm.__wbg_set_jscommandpacket_purge_interval(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set purge_time(arg0) {
        wasm.__wbg_set_jscommandpacket_purge_time(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set restart(arg0) {
        wasm.__wbg_set_jscommandpacket_restart(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set send_wait(arg0) {
        wasm.__wbg_set_jscommandpacket_send_wait(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) JSCommandPacket.prototype[Symbol.dispose] = JSCommandPacket.prototype.free;

export class JSMessagePacket {
    static __wrap(ptr) {
        const obj = Object.create(JSMessagePacket.prototype);
        obj.__wbg_ptr = ptr;
        JSMessagePacketFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JSMessagePacketFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jsmessagepacket_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get a1_voltage() {
        const ret = wasm.__wbg_get_jsmessagepacket_a1_voltage(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get batt_flags() {
        const ret = wasm.__wbg_get_jsmessagepacket_batt_flags(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get current_cal() {
        const ret = wasm.__wbg_get_jsmessagepacket_current_cal(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get current() {
        const ret = wasm.__wbg_get_jsmessagepacket_current(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get detected_zero() {
        const ret = wasm.__wbg_get_jsmessagepacket_detected_zero(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get elapsed_time() {
        const ret = wasm.__wbg_get_jsmessagepacket_elapsed_time(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get energy_wh() {
        const ret = wasm.__wbg_get_jsmessagepacket_energy_wh(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get fc_current() {
        const ret = wasm.__wbg_get_jsmessagepacket_fc_current(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get fc_flags() {
        const ret = wasm.__wbg_get_jsmessagepacket_fc_flags(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get fc_voltage() {
        const ret = wasm.__wbg_get_jsmessagepacket_fc_voltage(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get free_heap() {
        const ret = wasm.__wbg_get_jsmessagepacket_free_heap(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get hydro_wh() {
        const ret = wasm.__wbg_get_jsmessagepacket_hydro_wh(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get power() {
        const ret = wasm.__wbg_get_jsmessagepacket_power(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get temp() {
        const ret = wasm.__wbg_get_jsmessagepacket_temp(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get time_since_purge() {
        const ret = wasm.__wbg_get_jsmessagepacket_time_since_purge(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get voltage() {
        const ret = wasm.__wbg_get_jsmessagepacket_voltage(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Float32Array}
     */
    get acc() {
        const ret = wasm.jsmessagepacket_acc(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    get gyr() {
        const ret = wasm.jsmessagepacket_gyr(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    constructor() {
        const ret = wasm.jsmessagepacket_new();
        this.__wbg_ptr = ret;
        JSMessagePacketFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} arg0
     */
    set a1_voltage(arg0) {
        wasm.__wbg_set_jsmessagepacket_a1_voltage(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set batt_flags(arg0) {
        wasm.__wbg_set_jsmessagepacket_batt_flags(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set current_cal(arg0) {
        wasm.__wbg_set_jsmessagepacket_current_cal(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set current(arg0) {
        wasm.__wbg_set_jsmessagepacket_current(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set detected_zero(arg0) {
        wasm.__wbg_set_jsmessagepacket_detected_zero(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set elapsed_time(arg0) {
        wasm.__wbg_set_jsmessagepacket_elapsed_time(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set energy_wh(arg0) {
        wasm.__wbg_set_jsmessagepacket_energy_wh(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set fc_current(arg0) {
        wasm.__wbg_set_jsmessagepacket_fc_current(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set fc_flags(arg0) {
        wasm.__wbg_set_jsmessagepacket_fc_flags(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set fc_voltage(arg0) {
        wasm.__wbg_set_jsmessagepacket_fc_voltage(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set free_heap(arg0) {
        wasm.__wbg_set_jsmessagepacket_free_heap(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set hydro_wh(arg0) {
        wasm.__wbg_set_jsmessagepacket_hydro_wh(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set power(arg0) {
        wasm.__wbg_set_jsmessagepacket_power(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set temp(arg0) {
        wasm.__wbg_set_jsmessagepacket_temp(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set time_since_purge(arg0) {
        wasm.__wbg_set_jsmessagepacket_time_since_purge(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set voltage(arg0) {
        wasm.__wbg_set_jsmessagepacket_voltage(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) JSMessagePacket.prototype[Symbol.dispose] = JSMessagePacket.prototype.free;

export class JSScanResult {
    static __wrap(ptr) {
        const obj = Object.create(JSScanResult.prototype);
        obj.__wbg_ptr = ptr;
        JSScanResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JSScanResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jsscanresult_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get available_slots() {
        const ret = wasm.__wbg_get_jsscanresult_available_slots(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint8Array}
     */
    get mac_address() {
        const ret = wasm.jsscanresult_mac_address(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.jsscanresult_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {Uint8Array} mac_address
     */
    set mac_address(mac_address) {
        const ptr0 = passArray8ToWasm0(mac_address, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.jsscanresult_set_mac_address(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {string} name
     */
    set name(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.jsscanresult_set_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {number} arg0
     */
    set available_slots(arg0) {
        wasm.__wbg_set_jsscanresult_available_slots(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) JSScanResult.prototype[Symbol.dispose] = JSScanResult.prototype.free;

/**
 * @returns {Promise<void>}
 */
export function connect() {
    const ret = wasm.connect();
    return ret;
}

/**
 * @param {Uint8Array} mac
 * @returns {Promise<void>}
 */
export function connect_to_brain(mac) {
    const ptr0 = passArray8ToWasm0(mac, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.connect_to_brain(ptr0, len0);
    return ret;
}

/**
 * @returns {Promise<void>}
 */
export function disconnect_brain() {
    const ret = wasm.disconnect_brain();
    return ret;
}

/**
 * Called once from JS on page load to set panic hook.
 */
export function init() {
    wasm.init();
}

/**
 * @returns {boolean}
 */
export function is_connected() {
    const ret = wasm.is_connected();
    return ret !== 0;
}

/**
 * @returns {boolean}
 */
export function is_debugging() {
    const ret = wasm.is_debugging();
    return ret !== 0;
}

/**
 * @param {string} name
 * @returns {Promise<void>}
 */
export function rename_brain(name) {
    const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.rename_brain(ptr0, len0);
    return ret;
}

/**
 * Exported so JS can call e.g. `wasm_bindgen.send_command(bytes)`
 * while the read loop is running in the background.
 */
export function restart_receiver() {
    wasm.restart_receiver();
}

/**
 * @param {boolean} value
 */
export function set_debug(value) {
    wasm.set_debug(value);
}

/**
 * @param {Uint8Array} data
 * @returns {Promise<void>}
 */
export function write_bytes(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.write_bytes(ptr0, len0);
    return ret;
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_boolean_get_fa956cfa2d1bd751: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_c25d447a39f5578f: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_function_1ff95bcc5517c252: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_undefined_c05833b95a3cf397: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_344f42d3211c4765: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_fffb441def202758: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_call_a6e5c5dce5018821: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_clearTimeout_3629d6209dfcc46e: function(arg0) {
            const ret = clearTimeout(arg0);
            return ret;
        },
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_getReader_7455d080fa48369b: function(arg0) {
            const ret = arg0.getReader();
            return ret;
        },
        __wbg_getWriter_d71a99bcd847ccef: function() { return handleError(function (arg0) {
            const ret = arg0.getWriter();
            return ret;
        }, arguments); },
        __wbg_get_78f252d074a84d0b: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_instanceof_ReadableStreamDefaultReader_08a023749ad73fb6: function(arg0) {
            let result;
            try {
                result = arg0 instanceof ReadableStreamDefaultReader;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_SerialPort_9106dab49f55e715: function(arg0) {
            let result;
            try {
                result = arg0 instanceof SerialPort;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_309b927aaf7a3fc7: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_05ba1ee4f6781663: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_length_1f0964f4a5e2c6d8: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_log_d267660666346fb3: function(arg0) {
            console.log(arg0);
        },
        __wbg_navigator_99621db14b3f1099: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_aec3e25493d729fe: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__h37ef6ad6efa4054a(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = 0;
            }
        },
        __wbg_new_da52cf8fe3429cb2: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_from_slice_77cdfb7977362f3c: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_typed_1824d93f294193e5: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__h37ef6ad6efa4054a(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = 0;
            }
        },
        __wbg_of_5f1b88183ddb5d94: function(arg0, arg1) {
            const ret = Array.of(arg0, arg1);
            return ret;
        },
        __wbg_open_214ad026ba2ad8f7: function(arg0, arg1) {
            const ret = arg0.open(arg1);
            return ret;
        },
        __wbg_prototypesetcall_4770620bbe4688a0: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_queueMicrotask_0ab5b2d2393e99b9: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_queueMicrotask_6a09b7bc46549209: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_race_ac5c7b465abcfa15: function(arg0) {
            const ret = Promise.race(arg0);
            return ret;
        },
        __wbg_read_8afa15f12a160ef8: function(arg0) {
            const ret = arg0.read();
            return ret;
        },
        __wbg_readable_1d879debce819f05: function(arg0) {
            const ret = arg0.readable;
            return ret;
        },
        __wbg_releaseLock_33c25ace78b2d4c6: function(arg0) {
            arg0.releaseLock();
        },
        __wbg_releaseLock_5b92874cad775644: function(arg0) {
            arg0.releaseLock();
        },
        __wbg_requestPort_d79a2e899d7eaca9: function(arg0, arg1) {
            const ret = arg0.requestPort(arg1);
            return ret;
        },
        __wbg_resolve_2191a4dfe481c25b: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_send_message_e0b799ae4ee821ed: function(arg0) {
            send_message(JSMessagePacket.__wrap(arg0));
        },
        __wbg_send_scan_76b1bc42d21f369c: function(arg0) {
            send_scan(JSScanResult.__wrap(arg0));
        },
        __wbg_serial_a51917f3bcf9a1b5: function(arg0) {
            const ret = arg0.serial;
            return ret;
        },
        __wbg_setSignals_54c04513c6b0e1af: function(arg0, arg1) {
            const ret = arg0.setSignals(arg1);
            return ret;
        },
        __wbg_setTimeout_56bcdccbad22fd44: function() { return handleError(function (arg0, arg1) {
            const ret = setTimeout(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_setTimeout_cfa2cf195c3738db: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setTimeout(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_baud_rate_5a8e1faf985641c2: function(arg0, arg1) {
            arg0.baudRate = arg1 >>> 0;
        },
        __wbg_set_data_terminal_ready_382c9b2b306bb712: function(arg0, arg1) {
            arg0.dataTerminalReady = arg1 !== 0;
        },
        __wbg_set_filters_9051831be806d312: function(arg0, arg1, arg2) {
            arg0.filters = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_usb_vendor_id_77a2c04e60c40a11: function(arg0, arg1) {
            arg0.usbVendorId = arg1;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_4ef717fb391d88b7: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_8d1badc68b5a74f4: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_146583524fe1469b: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_f2829a2234d7819e: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_then_16d107c451e9905d: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_then_6ec10ae38b3e92f7: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_writable_5067496af1f6af81: function(arg0) {
            const ret = arg0.writable;
            return ret;
        },
        __wbg_write_f6dc0059c3feedce: function(arg0, arg1) {
            const ret = arg0.write(arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 105, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hcfecde9a3eeee277);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("SerialPort")], shim_idx: 67, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h534e3f838f7b8273);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("undefined")], shim_idx: 67, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h534e3f838f7b8273_2);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 84, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hc93a65512ac96ae3);
            return ret;
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./interface_backend_bg.js": import0,
    };
}

function wasm_bindgen__convert__closures_____invoke__hc93a65512ac96ae3(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__hc93a65512ac96ae3(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__hcfecde9a3eeee277(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__hcfecde9a3eeee277(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen__convert__closures_____invoke__h534e3f838f7b8273(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__h534e3f838f7b8273(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen__convert__closures_____invoke__h534e3f838f7b8273_2(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__h534e3f838f7b8273_2(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen__convert__closures_____invoke__h37ef6ad6efa4054a(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures_____invoke__h37ef6ad6efa4054a(arg0, arg1, arg2, arg3);
}

const JSCommandPacketFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jscommandpacket_free(ptr, 1));
const JSMessagePacketFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jsmessagepacket_free(ptr, 1));
const JSScanResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jsscanresult_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayJsValueViewFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('interface_backend_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
