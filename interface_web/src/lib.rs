use gloo_timers::future::TimeoutFuture;
use js_sys::Math::log;
use js_sys::{Uint8Array};
use util_types::{CommandPacket, RenamePacket, StaticString};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::ReadableStreamDefaultReader;
use web_sys::SerialOutputSignals;
use web_sys::{Serial, SerialOptions, SerialPort, SerialPortFilter, SerialPortRequestOptions};

use util_types::{MessagePacket, HandshakeInit, MESSAGE, PRINT, SCAN};

#[wasm_bindgen]
pub struct JSMessagePacket {
    acc: Box<[f32]>,
    gyr: Box<[f32]>,
    pub temp: f32,
    pub time_since_purge: u32,
    pub elapsed_time: u32,
    pub voltage: f32,
    pub current: f32,
    pub power: f32,
    pub fc_voltage: f32,
    pub fc_current: f32,
    pub batt_flags: u16,
    pub fc_flags: u16,
    pub energy_wh: f32,
    pub hydro_wh: f32,
    pub a1_voltage: f32,
    pub free_heap: u32,
    pub detected_zero: u16,
    pub current_cal: u16,
}
#[wasm_bindgen]
impl JSMessagePacket {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    #[wasm_bindgen(getter)]
    pub fn acc(&self) -> Vec<f32> {
        self.acc.to_vec()
    }
    #[wasm_bindgen(getter)]
    pub fn gyr(&self) -> Vec<f32> {
        self.gyr.to_vec()
    }
}
impl From<MessagePacket> for JSMessagePacket {
    fn from(value: MessagePacket) -> Self {
        Self {
            acc: Box::new(value.acc),
            gyr: Box::new(value.gyr),
            temp: value.temp,
            time_since_purge: value.time_since_purge,
            elapsed_time: value.elapsed_time,
            voltage: value.voltage,
            current: value.current,
            power: value.power,
            fc_voltage: value.fc_voltage,
            fc_current: value.fc_current,
            batt_flags: value.batt_flags,
            fc_flags: value.fc_flags,
            energy_wh: value.energy_wh,
            hydro_wh: value.hydro_wh,
            a1_voltage: value.a1_voltage,
            free_heap: value.free_heap,
            detected_zero: value.detected_zero,
            current_cal: value.current_cal,
        }
    }
}
impl Default for JSMessagePacket {
    fn default() -> Self {
        MessagePacket::default().into()
    }
}

#[wasm_bindgen]
pub struct JSScanResult {
    name: String,
    pub available_slots: u8,
    mac_address: Box<[u8]>,
}
#[wasm_bindgen]
impl JSScanResult {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    // Setter to update the string from JavaScript
    #[wasm_bindgen(setter)]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[wasm_bindgen(getter)]
    pub fn mac_address(&self) -> Box<[u8]> {
        self.mac_address.clone()
    }

    // Setter to update the string from JavaScript
    #[wasm_bindgen(setter)]
    pub fn set_mac_address(&mut self, mac_address: Box<[u8]>) {
        self.mac_address = mac_address;
    }
}

#[wasm_bindgen]
pub struct JSCommandPacket {
    pub purge_time: u32,
    pub purge_interval: u32,
    pub fan_speed: u8,
    pub restart: u8,
    pub send_wait: u8,
    connect_to_mac: Box<[u8]>,
}

#[wasm_bindgen]
extern "C" {
    fn send_message(m: JSMessagePacket);
    fn send_scan(s: JSScanResult);

    fn log_message(s: &str);
}

#[wasm_bindgen]
impl JSCommandPacket {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    #[wasm_bindgen(getter)]
    pub fn connect_to_mac(&self) -> Vec<u8> {
        self.connect_to_mac.to_vec()
    }
}
impl From<CommandPacket> for JSCommandPacket {
    fn from(value: CommandPacket) -> Self {
        Self {
            purge_time: value.purge_time,
            purge_interval: value.purge_interval,
            fan_speed: value.fan_speed,
            restart: value.restart,
            send_wait: value.send_wait,
            connect_to_mac: Box::new(value.connect_to_mac),
        }
    }
}
impl Default for JSCommandPacket {
    fn default() -> Self {
        CommandPacket::default().into()
    }
}

macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into())
    };
}


/// Called once from JS on page load to set panic hook.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Request a port from the browser's picker (must run from a click handler).
/// Vendor ID adjustable — 12346 kept from the original code as an example.
async fn request_port(vendor_id: u16) -> Result<SerialPort, JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let navigator = window.navigator();
    let serial: Serial = navigator.serial();

    let vendor = SerialPortFilter::new();
    vendor.set_usb_vendor_id(vendor_id);

    let options = SerialPortRequestOptions::new();
    options.set_filters(&[vendor]);

    let port_promise = serial.request_port_with_options(&options);
    let port_js = JsFuture::from(port_promise).await?;
    let port: SerialPort = port_js.dyn_into()?;

    Ok(port)
}

async fn open_port(port: &SerialPort, baud_rate: u32) -> Result<(), JsValue> {
    let opts = SerialOptions::new(baud_rate);
    JsFuture::from(port.open(&opts)).await?;
    Ok(())
}

async fn read_exact(reader: &mut ReadableStreamDefaultReader, num_bytes: usize) -> Result<Vec<u8>, JsValue> {
    let mut buffer = vec![0; num_bytes];
    let mut offset = 0;

    while offset < num_bytes {
        let result = JsFuture::from(reader.read()).await?;

        let done = js_sys::Reflect::get(&result, &"done".into())?
            .as_bool()
            .unwrap_or(true);
        if done {
            log!("stream closed");
            break;
        }

        let value = js_sys::Reflect::get(&result, &"value".into())?;
        let chunk: Uint8Array = value.dyn_into()?;
        log!("chunk: {:#?}", chunk.to_vec());
        let remaining = num_bytes - offset;
        let len = chunk.length() as usize;
        if len <= remaining && len != 0 {
            buffer[offset..(offset + len)].copy_from_slice(chunk.to_vec().as_slice());
            offset += len;
        } else {
            log!("chunk too long!!!");
            return Err(JsValue::from_str("chunk too long"))
        }
    }
    Ok(buffer)
}


async fn flush_stale_data(reader: &web_sys::ReadableStreamDefaultReader) -> Result<(), JsValue> {
    log!("flushing stale buffered data...");
    let mut total_discarded = 0usize;

    loop {
        let read_promise: js_sys::Promise = reader.read();

        let timeout_promise = js_sys::Promise::new(&mut |resolve, _reject| {
            let window = web_sys::window().unwrap();
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 1)
                .unwrap();
        });

        // race takes two Promises directly, not JsFutures
        let race = js_sys::Promise::race(&js_sys::Array::of2(&read_promise, &timeout_promise));

        // wrap the *result* of the race in JsFuture, once
        let result = JsFuture::from(race).await?;

        if result.is_undefined() {
            log!("flush: quiet period reached, discarded {} bytes", total_discarded);
            break;
        }

        let done = js_sys::Reflect::get(&result, &"done".into())?
            .as_bool()
            .unwrap_or(true);
        if done {
            log!("flush: stream closed during flush");
            break;
        }

        let value = js_sys::Reflect::get(&result, &"value".into())?;
        let chunk: Uint8Array = value.dyn_into()?;
        total_discarded += chunk.length() as usize;

        if total_discarded > 100_000_000 {
            log!("flush: hit safety cap, giving up and proceeding anyway");
            break;
        }
    }

    Ok(())
}
/// Reads bytes from the port forever, parsing the same header-based
/// protocol as the original desktop app (MESSAGE / PRINT / SCAN framed
/// packets), logging results to the browser console.
async fn read_loop(port: &SerialPort) -> Result<(), JsValue> {
    let readable = port.readable();
    let reader = readable
        .get_reader()
        .dyn_into::<web_sys::ReadableStreamDefaultReader>()?;

    flush_stale_data(&reader).await?;

    'read: loop { 
        let debug = DEBUG_MODE.get();
        log!("{}", debug);
        if debug {
            log!("waiting for bytes");
        }
        let result = JsFuture::from(reader.read()).await?;

        let done = js_sys::Reflect::get(&result, &"done".into())?
            .as_bool()
            .unwrap_or(true);
        if done {
            log!("stream closed");
            break;
        }

        let value = js_sys::Reflect::get(&result, &"value".into())?;
        let mut chunk = value.dyn_into::<Uint8Array>()?.to_vec();
        // let rheader = read_exact(&mut reader, 4).await;
        // if rheader.is_err() { continue }
        // let header = rheader.unwrap();
        if debug {
            log!("gottem {:#?}", chunk);
        }

        'drain: while chunk.len() > 0 {
            if chunk.len() <= 4 {
                log!("stream closed or something broke");
                break 'drain;
            }
            let len = u16::from_be_bytes([chunk[2], chunk[3]]) as usize;
            if len >= 520 {
                log!("chunk supposedly too long, ignoring");
                break 'drain;
            }
            let mut leftover = (len as isize) - 60;
            while leftover > 0 {
                let result = JsFuture::from(reader.read()).await?;

                let done = js_sys::Reflect::get(&result, &"done".into())?
                    .as_bool()
                    .unwrap_or(true);
                if done {
                    log!("stream closed");
                    break;
                }

                let value = js_sys::Reflect::get(&result, &"value".into())?;
                let chunk2 = value.dyn_into::<Uint8Array>()?.to_vec();
                chunk.extend(chunk2.iter());
                leftover -= 64
            }
            let payload = &chunk[4..(4 + len)];
    
            match [chunk[0], chunk[1]] {
                [MESSAGE, MESSAGE] => {
                    if len != core::mem::size_of::<MessagePacket>() {
                        log!(
                            "MESSAGE packet wrong size: got {}, expected {}",
                            len,
                            core::mem::size_of::<MessagePacket>()
                        );
                    } else {
                        let packet: MessagePacket = *bytemuck::from_bytes(payload);
                        if debug {
                            log!("MESSAGE: {:?}", packet);
                        }
                        send_message(packet.into());
                    }
                }
                [PRINT, PRINT] => {
                    if let Ok(s) = String::from_utf8(payload.to_vec()) {
                        log!("PRINT: {s}");
                    } else {
                        log!("PRINT: <invalid utf8>");
                    }
                }
                [SCAN, SCAN] => {
                    let expected = core::mem::size_of::<HandshakeInit>() + 6; // + [u8; 6] mac
                    if len != expected {
                        log!("SCAN packet wrong size: got {}, expected {}", len, expected);
                    } else {
                        let init_size = core::mem::size_of::<HandshakeInit>();
                        let init: HandshakeInit = *bytemuck::from_bytes(&payload[..init_size]);
                        let mac: [u8; 6] = payload[init_size..init_size + 6].try_into().unwrap();
                        send_scan(JSScanResult {
                            name: init.name.to_string(),
                            available_slots: init.available_slots,
                            mac_address: Box::new(mac)
                        });
                        log!("SCAN: {:?} mac={:02x?}", init, mac);
                    }
                }
                other => {
                    log!("unknown header {:?}, discarding {} bytes", other, len);
                }
            }
            chunk.drain(0..(4+len));
        }
    }

    reader.release_lock();
    Ok(())
}

use std::cell::Cell;
use std::cell::RefCell;

thread_local! {
    static PORT: RefCell<Option<SerialPort>> =       RefCell::new(None);
    static CONNECTED_BRAIN: Cell<Option<[u8; 6]>> =  Cell::new(None);
    static DEBUG_MODE: Cell<bool> =                  Cell::new(false);
}

#[wasm_bindgen]
pub async fn write_bytes(data: &[u8]) -> Result<(), JsValue> {
    let port = PORT.with(|p| p.borrow().clone());

    match port {
        Some(port) => {
            let writable = port.writable();
            let writer = writable
                .get_writer()?;

            let array = Uint8Array::from(data);
            JsFuture::from(writer.write_with_chunk(&array)).await?;

            writer.release_lock();
            Ok(())
        }
        None => Ok(log!("restart_receiver: no port connected")),
    }
}

#[wasm_bindgen]
pub async fn connect() {
    match request_port(12346).await {
        Ok(port) => {
            log!("port selected");
            if let Err(e) = open_port(&port, 115_200).await {
                log!("failed to open port: {:?}", e);
                return;
            }
            log!("port opened at 115200 baud");

            // Stash a clone of the handle for other functions to use.
            // web_sys::SerialPort is a JsValue wrapper, so .clone() just
            // clones the JS reference, not the underlying object.
            spawn_local(async move {
                PORT.with(|p| *p.borrow_mut() = Some(port.clone()));
    
                if let Err(e) = read_loop(&port).await {
                    log!("read loop error: {:?}", e);
                }
    
                // Read loop exited (stream closed/errored) — clear it.
                PORT.with(|p| *p.borrow_mut() = None);
            });
        }
        Err(e) => {
            log!("request_port failed or was cancelled: {:?}", e);
        }
    }

}

/// Exported so JS can call e.g. `wasm_bindgen.send_command(bytes)`
/// while the read loop is running in the background.
#[wasm_bindgen]
pub fn restart_receiver() {
    spawn_local(async move {
        // Grab a clone of the port handle, then drop the borrow before
        // awaiting — never hold a RefCell borrow across an .await point.
        let port = PORT.with(|p| p.borrow().clone());

        match port {
            Some(port) => {
                let signals = SerialOutputSignals::new();
                signals.set_data_terminal_ready(true);
                port.set_signals_with_signals(&signals).await.expect("bruh");
                log!("aaa bro1");
                TimeoutFuture::new(100).await;
                let signals = SerialOutputSignals::new();
                signals.set_data_terminal_ready(false);
                port.set_signals_with_signals(&signals).await.expect("bruh");
                log!("aaa bro")
            }
            None => log!("restart_receiver: no port connected"),
        }
    });
}

#[wasm_bindgen]
pub async fn disconnect_brain() -> Result<(), JsValue> {
    let cpckt = CommandPacket {
        connect_to_mac: [0; 6],
        restart: 0,
        purge_time: 0,
        purge_interval: 0,
        fan_speed: 0,
        send_wait: 0,
        _pad: [0; 3]
    };
    CONNECTED_BRAIN.replace(None);

    write_bytes(&[&[0][..], bytemuck::bytes_of(&cpckt)].concat()).await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn rename_brain(name: &str) -> Result<(), JsValue> {
    let cpckt = RenamePacket {
        new_name: StaticString::new(name)
    };
    CONNECTED_BRAIN.replace(None);

    write_bytes(&[&[3][..], bytemuck::bytes_of(&cpckt)].concat()).await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn connect_to_brain(mac: Box<[u8]>) -> Result<(), JsValue> {
    log!("hi {mac:?}");
    let mac: Option<[u8; 6]> = mac[0..6].try_into().ok();
    let cpckt = CommandPacket {
        connect_to_mac: mac.unwrap(),
        restart: 0,
        purge_time: 0,
        purge_interval: 0,
        fan_speed: 0,
        send_wait: 0,
        _pad: [0; 3]
    };
    CONNECTED_BRAIN.replace(mac);

    write_bytes(&[&[0][..], bytemuck::bytes_of(&cpckt)].concat()).await?;

    Ok(())
}

#[wasm_bindgen]
pub fn is_connected() -> bool {
    PORT.with(|p| p.borrow().is_some())
}

#[wasm_bindgen]
pub fn set_debug(value: bool) {
    log!("setting {value}");
    DEBUG_MODE.with(|f| f.set(value));
}

#[wasm_bindgen]
pub fn is_debugging() -> bool {
    !DEBUG_MODE.with(|f| f.get())
}