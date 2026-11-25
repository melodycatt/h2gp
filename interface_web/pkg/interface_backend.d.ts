/* tslint:disable */
/* eslint-disable */

export class JSCommandPacket {
    free(): void;
    [Symbol.dispose](): void;
    constructor();
    fan_speed: number;
    purge_interval: number;
    purge_time: number;
    restart: number;
    send_wait: number;
    readonly connect_to_mac: Uint8Array;
}

export class JSMessagePacket {
    free(): void;
    [Symbol.dispose](): void;
    constructor();
    a1_voltage: number;
    batt_flags: number;
    current_cal: number;
    current: number;
    detected_zero: number;
    elapsed_time: number;
    energy_wh: number;
    fc_current: number;
    fc_flags: number;
    fc_voltage: number;
    free_heap: number;
    hydro_wh: number;
    power: number;
    temp: number;
    time_since_purge: number;
    voltage: number;
    readonly acc: Float32Array;
    readonly gyr: Float32Array;
}

export class JSScanResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    available_slots: number;
    mac_address: Uint8Array;
    name: string;
}

export function connect(): Promise<void>;

export function connect_to_brain(mac: Uint8Array): Promise<void>;

export function disconnect_brain(): Promise<void>;

/**
 * Called once from JS on page load to set panic hook.
 */
export function init(): void;

export function is_connected(): boolean;

export function is_debugging(): boolean;

export function rename_brain(name: string): Promise<void>;

/**
 * Exported so JS can call e.g. `wasm_bindgen.send_command(bytes)`
 * while the read loop is running in the background.
 */
export function restart_receiver(): void;

export function set_debug(value: boolean): void;

export function write_bytes(data: Uint8Array): Promise<void>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_jscommandpacket_fan_speed: (a: number) => number;
    readonly __wbg_get_jscommandpacket_purge_interval: (a: number) => number;
    readonly __wbg_get_jscommandpacket_purge_time: (a: number) => number;
    readonly __wbg_get_jscommandpacket_restart: (a: number) => number;
    readonly __wbg_get_jscommandpacket_send_wait: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_a1_voltage: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_batt_flags: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_current: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_current_cal: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_detected_zero: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_elapsed_time: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_energy_wh: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_fc_current: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_fc_flags: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_fc_voltage: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_free_heap: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_hydro_wh: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_power: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_temp: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_time_since_purge: (a: number) => number;
    readonly __wbg_get_jsmessagepacket_voltage: (a: number) => number;
    readonly __wbg_get_jsscanresult_available_slots: (a: number) => number;
    readonly __wbg_jscommandpacket_free: (a: number, b: number) => void;
    readonly __wbg_jsmessagepacket_free: (a: number, b: number) => void;
    readonly __wbg_jsscanresult_free: (a: number, b: number) => void;
    readonly __wbg_set_jscommandpacket_fan_speed: (a: number, b: number) => void;
    readonly __wbg_set_jscommandpacket_purge_interval: (a: number, b: number) => void;
    readonly __wbg_set_jscommandpacket_purge_time: (a: number, b: number) => void;
    readonly __wbg_set_jscommandpacket_restart: (a: number, b: number) => void;
    readonly __wbg_set_jscommandpacket_send_wait: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_a1_voltage: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_batt_flags: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_current: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_current_cal: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_detected_zero: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_elapsed_time: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_energy_wh: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_fc_current: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_fc_flags: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_fc_voltage: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_free_heap: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_hydro_wh: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_power: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_temp: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_time_since_purge: (a: number, b: number) => void;
    readonly __wbg_set_jsmessagepacket_voltage: (a: number, b: number) => void;
    readonly __wbg_set_jsscanresult_available_slots: (a: number, b: number) => void;
    readonly connect: () => any;
    readonly connect_to_brain: (a: number, b: number) => any;
    readonly disconnect_brain: () => any;
    readonly init: () => void;
    readonly is_connected: () => number;
    readonly is_debugging: () => number;
    readonly jscommandpacket_connect_to_mac: (a: number) => [number, number];
    readonly jscommandpacket_new: () => number;
    readonly jsmessagepacket_acc: (a: number) => [number, number];
    readonly jsmessagepacket_gyr: (a: number) => [number, number];
    readonly jsmessagepacket_new: () => number;
    readonly jsscanresult_mac_address: (a: number) => [number, number];
    readonly jsscanresult_name: (a: number) => [number, number];
    readonly jsscanresult_set_mac_address: (a: number, b: number, c: number) => void;
    readonly jsscanresult_set_name: (a: number, b: number, c: number) => void;
    readonly rename_brain: (a: number, b: number) => any;
    readonly restart_receiver: () => void;
    readonly set_debug: (a: number) => void;
    readonly write_bytes: (a: number, b: number) => any;
    readonly wasm_bindgen__convert__closures_____invoke__hcfecde9a3eeee277: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h534e3f838f7b8273: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h534e3f838f7b8273_2: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h37ef6ad6efa4054a: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hc93a65512ac96ae3: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
