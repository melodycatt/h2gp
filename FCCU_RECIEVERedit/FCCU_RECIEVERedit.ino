#include <esp_now.h>
#include <WiFi.h>

// Command packet structure for remote parameter updates
typedef struct {
    uint32_t purgeTime;
    uint32_t purgeInterval;
    uint8_t  fanSpeed;
} cmd_packet_t;
static cmd_packet_t cmdPacket;

// Remotely adjustable variables (defaults)
static uint32_t purgeTime     = 200;    // ms
static uint32_t purgeInterval = 50000;  // ms
static uint8_t  fanSpeed      = 128;    // 0-255

// ESP-NOW peer MAC address (transmitter) - replace with actual MAC
static uint8_t peerAddr[6] = { 0xB0, 0x81, 0x84, 0x05, 0xD6, 0xF4 }; // TRX B B0:81:84:05:D6:F4

float Ei = 0.0000;  // Total energy usage in Joules (Watt-seconds)
unsigned long lastTime = 0.000;  // Stores the last loop time

const int period = 2;  // Moving average period
float accxValues[period] = {0};
float accyValues[period] = {0};
float acczValues[period] = {0};
float currentValues[period] = {0};
int avgIndex = 0;

// Structure example to receive data
// Must match the sender structure
typedef struct struct_message {
    String message; 
    float ACCX;
    float ACCY;
    float ACCZ;
    float GYRX;
    float GYRY;
    float GYRZ;
    float TEMP;
    float Voltage;
    float Current;
    float Power;
    float ElapsedTime;
    float battVoltage;   // New
    float fcVoltage;     // New
    float fcCurrent;     // New
    uint32_t uptime;     // New
    float energyWh;      // New: total energy in Wh from transmitter
    float hydroWh;       // New: HydroWh (Wh) from transmitter
    float a1Voltage;     // New: A1 analog voltage (V)
} struct_message;

// Create a struct_message called myData
struct_message myData;

// Function to calculate moving average
float calculateMovingAverage(float *values, float newValue) {
    values[avgIndex] = newValue;
    float sum = 0.0;
    for (int i = 0; i < period; i++) {
        sum += values[i];
    }
    return sum / period;
}

// callback function that will be executed when data is received
void OnDataRecv(const uint8_t * mac, const uint8_t *incomingData, int len) {
    memcpy(&myData, incomingData, sizeof(myData));
    float Power = myData.Voltage * myData.Current;
    unsigned long currentTime = myData.ElapsedTime;
    float deltaTime = (currentTime - lastTime) / 1000.0; // Convert ms to seconds
    lastTime = currentTime;

    float power = myData.Voltage * myData.Current;  // Power in Watts
    Ei += power * deltaTime;  // Update total energy

    // Calculate moving averages
    float accxAvg = calculateMovingAverage(accxValues, myData.ACCX);
    float accyAvg = calculateMovingAverage(accyValues, myData.ACCY);
    float acczAvg = calculateMovingAverage(acczValues, myData.ACCZ);
    float currentAvg = calculateMovingAverage(currentValues, myData.Current);

    // Update index for moving average
    avgIndex = (avgIndex + 1) % period;
    
    // Parse new fields
    float battV = myData.battVoltage;
    float fcV   = myData.fcVoltage;
    float fcI   = myData.fcCurrent;
    uint32_t up = myData.uptime;
    float energyWh = myData.energyWh; // New
    float hydroWh  = myData.hydroWh;  // New

    // Send data to the Serial Plotter in the same order as the header labels
    // ACCX_Avg,ACCY_Avg,ACCZ_Avg,CurrentAvg,BattV,FC_V,FC_I,UpTime,EnergyWh,HydroWh,A1_V
    Serial.print(accxAvg);
    Serial.print(",");
    Serial.print(accyAvg);
    Serial.print(",");
    Serial.print(acczAvg);
    Serial.print(",");
    Serial.print(currentAvg/1000);
    Serial.print(",");
    Serial.print(battV);
    Serial.print(",");
    Serial.print(fcV);
    Serial.print(",");
    Serial.print(fcI);
    Serial.print(",");
    Serial.print(up);
    Serial.print(",");
    Serial.print(energyWh, 3); // New: 3 decimal places
    Serial.print(",");
    Serial.print(hydroWh, 3);  // New: 3 decimal places
    Serial.print(",");
    Serial.print(myData.a1Voltage);
    Serial.print("\n");
}

void setup() {
    // Initialize Serial Monitor
    Serial.begin(115200);
    Serial.print("ACCX_Avg,ACCY_Avg,ACCZ_Avg,CurrentAvg,BattV,FC_V,FC_I,UpTime,EnergyWh,HydroWh,A1_V\n");

    // Set device as a Wi-Fi Station
    WiFi.mode(WIFI_STA);

    // Init ESP-NOW
    if (esp_now_init() != ESP_OK) {
        Serial.println("Error initializing ESP-NOW");
        return;
    }

    // Once ESPNow is successfully Init, we will register for recv CB to
    // get recv packer info
    esp_now_register_recv_cb(esp_now_recv_cb_t(OnDataRecv));

    // Add the transmitter as an ESP-NOW peer
    esp_now_peer_info_t peerInfo = {};
    memcpy(peerInfo.peer_addr, peerAddr, 6);
    peerInfo.channel = 0;
    peerInfo.encrypt = false;
    if (esp_now_add_peer(&peerInfo) != ESP_OK) {
        Serial.println("Failed to add ESP-NOW peer");
    }
}

void loop() {
    // Handle incoming serial commands of the form: SET <NAME> <VALUE>
    if (Serial.available()) {
        String line = Serial.readStringUntil('\n');
        line.trim();
        if (line.startsWith("SET ")) {
            int sp1 = line.indexOf(' ');
            int sp2 = line.indexOf(' ', sp1 + 1);
            String name = line.substring(sp1 + 1, sp2);
            uint32_t val = (uint32_t)line.substring(sp2 + 1).toInt();

            if (name == "PURGE_TIME") {
                purgeTime = val;
            } else if (name == "PURGE_INTERVAL") {
                purgeInterval = val;
            } else if (name == "FAN_SPEED") {
                fanSpeed = (uint8_t)val;
            }

            // Populate and send command packet over ESP-NOW
            cmdPacket.purgeTime     = purgeTime;
            cmdPacket.purgeInterval = purgeInterval;
            cmdPacket.fanSpeed      = fanSpeed;
            esp_now_send(peerAddr, (uint8_t*)&cmdPacket, sizeof(cmdPacket));

            Serial.print("Sent command: ");
            Serial.println(line);
        }
    }

    // Optionally, here you can act on purgeTime, purgeInterval, and fanSpeed in your receiver logic
}
