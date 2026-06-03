use esp_hal::{Blocking, delay::Delay, i2c::master::I2c};
use pca9685::ServoDriver;

const PWM_ADDR: u8 = 0x67;
const ESC_CH: u8 = 0;

const PMIN_US: i32 = 1000;
const PMAX_US: i32 = 2000;
const PNEU_US: i32 = 1500;

pub fn driver<'a>(i2c: &I2c<'a, Blocking>) -> ServoDriver<'a, Blocking> {
    ServoDriver::with_address(PWM_ADDR, i2c)
}
pub fn us_to_ticks(us: i32, freq_hz: f32) -> u16 {
    let mut ticks = (us as f32 * 4096.0 * freq_hz) / 1000000.0;
    if ticks < 0. {
        ticks = 0.;
    }
    if ticks > 4095. {
        ticks = 4095.;
    }
    return (ticks + 0.5) as u16;
}
pub fn write_us(channel: u8, us: i32, pwm: &mut ServoDriver<Blocking>) {
    let off = us_to_ticks(us, pwm.pwm_freq);
    pwm.set_pwm(channel, 0, off).unwrap();
}
pub fn set_throttle_percent(channel: u8, mut pct: f32, pwm: &mut ServoDriver<Blocking>) {
    if pct < 0. {
        pct = 0.
    }
    if pct > 100. {
        pct = 100.
    }
    let us = PMIN_US + ((PMAX_US - PMIN_US) as f32 * (pct / 100.)) as i32;
    write_us(channel, us, pwm);
}
pub fn apply_freq(hz: f32, pwm: &mut ServoDriver<Blocking>) {
    pwm.set_pwm_freq(hz).unwrap();
}
pub fn setup(pwm: &mut ServoDriver<Blocking>) {
    let delay = Delay::new();
    pwm.begin(0).unwrap();
    apply_freq(50., pwm);
    write_us(ESC_CH, PMIN_US, pwm);
    delay.delay_millis(3000);
    write_us(ESC_CH, PNEU_US, pwm)
}
// let uart = Uart::new(
//     peripherals.UART0.reborrow(),
//     Config::default()
//         .with_baudrate(100_000)
//         .with_data_bits(esp_hal::uart::DataBits::_8)
//         .with_parity(esp_hal::uart::Parity::Even)
//         .with_stop_bits(esp_hal::uart::StopBits::_2),
//     // baudrate: 100_000,
//     // data_bits: esp_hal::uart::DataBits::DataBits8,
//     // parity: esp_hal::uart::Parity::Even,
//     // stop_bits: esp_hal::uart::StopBits::STOP2,
//     // ..Default::default()
//     // },
// )
// .unwrap()
// .with_rx(periphral.GPIO9);

// peripherals
//     .UART0
//     .register_block()
//     .conf0()
//     .modify(|_, w| w.rxd_inv().set_bit());
// fn scan_i2c(pwm: &mut ServoDriver<Blocking>) { {}}
/*
XIAO ESP32-C3 -> PCA9685 -> ESC on channel 0
- XIAO ESP32-C3 I2C: SDA=GPIO6, SCL=GPIO7 (default)
- PCA9685 address (A0..A5 open): 0x40
- ESC signal on PCA9685 channel 0
- Arming sends MIN (1000 us) for 3s, then holds neutral

Serial commands:
us <value> e.g. "us 1500" -> set 1500 µs
pct <0..100> e.g. "pct 30" -> 30% (maps 1000..2000 µs)
freq <Hz> e.g. "freq 50" or "freq 60"
Adafruit_PWMServoDriver pwm = Adafruit_PWMServoDriver(PCA9685_ADDR);

void setup() {
Serial.begin(115200);
delay(200);

// Use default XIAO ESP32-C3 I2C pins (SDA=6, SCL=7). You can also do Wire.begin(6,7);
Wire.begin();
Wire.setClock(400000); // Fast-mode I2C for reliability

scanI2C();
Serial.println(F("Init PCA9685 @ 0x40..."));
pwm.begin();
// If pulses seem off, uncomment and tune your oscillator (typical 25-27 MHz):
// pwm.setOscillatorFrequency(27000000);
applyFreq(50.0f);

// *** ARM SEQUENCE: hold MIN on power-up ***
Serial.println(F("Arming ESC at MIN (1000 us) for 3 seconds..."));
writeMicroseconds(ESC_CH, PULSE_MIN_US);
delay(3000);

// Move to neutral and wait
Serial.println(F("Holding NEUTRAL (1500 us). You can now send commands:"));
writeMicroseconds(ESC_CH, PULSE_NEU_US);
}

void handleSerial() {
if (!Serial.available()) return;
String cmd = Serial.readStringUntil('\n');
cmd.trim();

if (cmd.startsWith("us ")) {
    int sp = cmd.substring(3).toInt();
    sp = constrain(sp, 800, 2200); // safety clamp
    writeMicroseconds(ESC_CH, sp);
    Serial.print(F("Set pulse: "));
    Serial.print(sp);
    Serial.println(F(" us"));
} else if (cmd.startsWith("pct ")) {
    float p = cmd.substring(4).toFloat();
    p = constrain(p, 0.0f, 100.0f);
    setThrottlePercent(ESC_CH, p);
    Serial.print(F("Set throttle: "));
    Serial.print(p, 1);
    Serial.println(F(" %"));
} else if (cmd.startsWith("freq ")) {
    float hz = cmd.substring(5).toFloat();
    hz = constrain(hz, 40.0f, 400.0f);
    applyFreq(hz);
} else if (cmd == "min") {
    writeMicroseconds(ESC_CH, PULSE_MIN_US);
    Serial.println(F("MIN (1000 us)"));
} else if (cmd == "neu") {
    writeMicroseconds(ESC_CH, PULSE_NEU_US);
    Serial.println(F("NEU (1500 us)"));
} else if (cmd == "max") {
    writeMicroseconds(ESC_CH, PULSE_MAX_US);
    Serial.println(F("MAX (2000 us)"));
} else if (cmd == "scan") {
    scanI2C();
} else {
Serial.println(F("Commands: 'us <800-2200>', 'pct <0-100>', 'freq <Hz>', 'min','neu','max','scan'"));
}
}

void loop() {
handleSerial();
}
void scanI2C() {
Serial.println(F("Scanning I2C..."));
byte found = 0;
for (uint8_t addr = 1; addr < 127; addr++) {
Wire.beginTransmission(addr);
uint8_t err = Wire.endTransmission();
if (err == 0) {
Serial.print(F(" Found device at 0x"));
if (addr < 16) Serial.print('0');
Serial.print(addr, HEX);
Serial.println();
found++;
}
*/

pub fn decode_sbus(buf: &[u8; 25]) -> [u16; 16] {
    let mut ch = [0u16; 16];

    ch[0] = ((buf[1] as u16 | (buf[2] as u16) << 8) & 0x07FF);
    ch[1] = (buf[2] as u16 >> 3 | (buf[3] as u16) << 5) & 0x07FF;
    ch[2] = ((buf[3] as u16 >> 6 | (buf[4] as u16) << 2 | (buf[5] as u16) << 10) & 0x07FF);
    ch[3] = ((buf[5] as u16 >> 1 | (buf[6] as u16) << 7) & 0x07FF);
    ch[4] = ((buf[6] as u16 >> 4 | (buf[7] as u16) << 4) & 0x07FF);
    ch[5] = ((buf[7] as u16 >> 7 | (buf[8] as u16) << 1 | (buf[9] as u16) << 9) & 0x07FF);
    ch[6] = ((buf[9] as u16 >> 2 | (buf[10] as u16) << 6) & 0x07FF);
    ch[7] = ((buf[10] as u16 >> 5 | (buf[11] as u16) << 3) & 0x07FF);

    ch[8] = ((buf[12] as u16 | (buf[13] as u16) << 8) & 0x07FF);
    ch[9] = ((buf[13] as u16 >> 3 | (buf[14] as u16) << 5) & 0x07FF);
    ch[10] = ((buf[14] as u16 >> 6 | (buf[15] as u16) << 2 | (buf[16] as u16) << 10) & 0x07FF);
    ch[11] = ((buf[16] as u16 >> 1 | (buf[17] as u16) << 7) & 0x07FF);
    ch[12] = ((buf[17] as u16 >> 4 | (buf[18] as u16) << 4) & 0x07FF);
    ch[13] = ((buf[18] as u16 >> 7 | (buf[19] as u16) << 1 | (buf[20] as u16) << 9) & 0x07FF);
    ch[14] = ((buf[20] as u16 >> 2 | (buf[21] as u16) << 6) & 0x07FF);
    ch[15] = ((buf[21] as u16 >> 5 | (buf[22] as u16) << 3) & 0x07FF);

    ch
}

pub fn sbus_to_us(val: u16) -> u16 {
    // SBUS range ~172–1811 → map to 1000–2000 µs
    let v = val.clamp(172, 1811) as i32;
    let us = 1000 + (v - 172) * 1000 / (1811 - 172);
    us as u16
}
