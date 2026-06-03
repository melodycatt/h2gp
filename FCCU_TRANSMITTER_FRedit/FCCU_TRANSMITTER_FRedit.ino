#include <esp_now.h>
#include <WiFi.h>
#include <Wire.h>

// VOLTAGE + CURRENT MONITOR (VC)
#include <INA226_WE.h>
// MOTION + GYRO MONITOR (MG)
#include <ICM42688.h>

// BATTERY VC ADDR
#define I2C_ADDRESS 0x41
// FUEL CELL VC ADDR
#define I2C_ADDRESS1 0x44

// NO FREAKING IDEA. BELOW IS AS MUCH AS I KNOW
// Command packet for remote updates
typedef struct {
    uint32_t purgeTime;
    uint32_t purgeInterval;
    uint8_t  fanSpeed;
} cmd_packet_t;
static cmd_packet_t cmdRecvPacket;

// WHAT HE SAID
// Remote-tunable parameters (defaults)
static uint32_t purgeTime = 1000;    // ms valve open duration
static uint32_t purgeInterval = 5000;  // ms between purges
static uint8_t fanSpeed = 255;    // 0–255 PWM on D8 (I DO NOT KNOW WHAT D8 IS)

// UM
int fan;
// WHY ARE WE MAKING TWO VARIABLES WITH THE EXACT SAME NAME
// EXCEPT ONE HAS THE SHORT PREFIX
// BUT IT IS ALSO A LONG
// WHAT DOES THIS MEANNNNN
unsigned long previousMillis = 0;
unsigned long shortPreviousMillis = 0;
const unsigned long shortInterval = 60000;  // 60 000 ms


// BATTERY VC
INA226_WE ina226 = INA226_WE(I2C_ADDRESS);
// FUEL CELL VC
INA226_WE ina226_fc = INA226_WE(I2C_ADDRESS1);

// Energy accumulation (Wh)
static double energyWh = 0.0;
static double hydroWh  = 0.0;  // New: FC energy accumulation
static unsigned long lastEnergySampleMs = 0;

// BLACK MAGIC? WHAT IS THIS MAC ADDRESS ASS SHIT
// ACTUALLY IT MIGHT BE A MAC ADDRESS THAT WOULD MAKE SENSE 
// BUT IDK HOW THAT COMMENT RELATES TO THE CODE OR WHY THERES 3
// OR WHAT CONDITION IS PROBS SOME FLAGS
int condition = 1;
uint8_t broadcastAddress[] = {0x58, 0x8C, 0x81, 0xA1, 0x0F, 0xC4}; //B0:81:84:05:D6:F4, B0:81:84:05:D8:64 Red rcv: 58:8C:81:A1:0F:C4
// MG MONITOR
ICM42688 IMU(Wire, 0x68);

// I DONT KNOW
// Variables to control the message display duration
unsigned long messageStartTime = 0;  // Time when the message was set
unsigned long displayDuration = 5000; // Duration (in milliseconds) to show the message
int purgetime = 0;
// WHY ARE WE CALLING THIS STRUCT_MESSAGE
// BRICKKKKKKK
// Structure to send data
typedef struct struct_message {
	String message = "";  // URGHHHH
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
						 // THANKS CHAT FOR ACTUALLY HELPFUL COMMENTS
	float battVoltage;   // Battery voltage at INA226 0x41
	float fcVoltage;     // Fuel-cell voltage at INA226 0x44
	float fcCurrent;     // Fuel-cell current (A) at INA226 0x44
	uint32_t uptime;     // milliseconds since start
	float energyWh;      // Added: integrated energy (Wh)
	float hydroWh;       // New: integrated HydroWh (Wh) 
	float a1Voltage;     // New: A1 analog voltage (V)
						 // New: SUCK MY DICKKKKKK
} struct_message;

// Create a struct_message called myData
// NO WERE TROLLING LIKE ARE YOU FOUR ARE YOU FUCKIGN FOUR YEARS OLD WHAT IS MYDATAAAAAA
struct_message myData;

// HURGHHHHHHHH
esp_now_peer_info_t peerInfo;

// UPPERCASE??? KILLLLL MEEEEEEEEEEEE
// BUT YEAH IT PROBS WAS A MAC ADDR GIVENT THAT ARG
// callback when data is sent
void OnDataSent(const wifi_tx_info_t *mac_addr, esp_now_send_status_t status) {
	//WHY ARE WE PRINTING MID RACE :)
	Serial.print("\r\nLast Packet Send Status:\t");
	if (status == ESP_NOW_SEND_SUCCESS) {
		Serial.println("Delivery Success");
		// NO EXPLANATION OF WHAT THIS PIN IS!
		//digitalWrite(A1, HIGH);  // Set the pin high when delivery is successful
	} else {
		Serial.println("Delivery Fail");
		//digitalWrite(A1, LOW);   // Optionally, set the pin low when delivery fails
	}
}
// Callback for incoming ESP-NOW commands
void OnDataRecv(const esp_now_recv_info *mac_addr, const uint8_t *incomingData, int len) {
    memcpy(&cmdRecvPacket, incomingData, sizeof(cmdRecvPacket));
    purgeTime = cmdRecvPacket.purgeTime;
    purgeInterval = cmdRecvPacket.purgeInterval;
    fanSpeed = cmdRecvPacket.fanSpeed;
    Serial.printf("Received params → purgeTime=%u, purgeInterval=%u, fanSpeed=%u\n",
                  purgeTime, purgeInterval, fanSpeed);
    // Debug print for new fanSpeed
    //Serial.printf("Received params → fanSpeed=%u\n", fanSpeed);
}
 
void setup() {
	Serial.begin(115200);
	Serial.println("starting");
	float ENow = 0.000;
	float ELast = 0.000;
	analogWrite(D9, 255); 
	analogWrite(D10, 10);
	delay(1500);
	analogWrite(D10, 255);
	analogWrite(D9, 10);
	analogWrite(D1, 255);
	analogWrite(D0, 255);
	delay(1000);
	analogWrite(D1, 0);
	analogWrite(D0, 0);
	pinMode(D1, OUTPUT);
	pinMode(D0, OUTPUT);
	pinMode(D10, OUTPUT);
	pinMode(D9, OUTPUT);
	pinMode(D7, OUTPUT);
	// Configure LEDC channel for PWM on GPIO 8
	//ledcAttach(7, 5000, 8);      // channel 0, 5 kHz, 8-bit resolution
	//ledcAttachPin(8, 0);        // attach channel 0 to pin D8 (GPIO8)
	//pinMode(A3, OUTPUT);
	pinMode(D2, INPUT);    // Thermistor divider input
	// Init Serial Monitor
	Wire.begin();
	ina226.init();
	ina226_fc.init();
	
	// Set device as a Wi-Fi Station
	WiFi.mode(WIFI_STA);

	// Init ESP-NOW
	if (esp_now_init() != ESP_OK) {
		Serial.println("Error initializing ESP-NOW");
		return;
	}
	float Voltage = 0.0;
	float Current = 0.0;
	float Power = 0.0;
	// CB = CALLBACK
	// Once ESPNow is successfully Init, we will register for Send CB to
	// get the status of Transmitted packet
	esp_now_register_send_cb(OnDataSent);
	// Also listen for incoming parameter updates
	esp_now_register_recv_cb(OnDataRecv);

	//INA WORKS BY MEASURING VOLTAGE DROP OVER PRECISION (SHUNT) RESISTOR
	//setResistorRange(SHUNT, MAX AMPERES??)
	ina226.setResistorRange(0.003,20);
	ina226_fc.setResistorRange(0.003,20);
	//THANK YOU BILLY
	Serial.println("INA's ON");
	//ON THE TIN
	ina226.waitUntilConversionCompleted();
	// Register peer
	// ARGHHHHH UB
	memcpy(peerInfo.peer_addr, broadcastAddress, 6);
	peerInfo.channel = 0;  
	peerInfo.encrypt = false;
	
	// Add peer        
	if (esp_now_add_peer(&peerInfo) != ESP_OK){
		Serial.println("Failed to add peer");
		return;
	}
	// holy shit. this is the last straw
	// ITS LITERALLY INN THE FUCKLING README OF THE LIBRARY ON GITHUB IT SAYS RIGHT THEREE
	// > This function returns a positive value on a successful initialization and returns a negative value on an unsuccessful initialization.
	// YOURE BRICKEDDDDDDD
	if (!IMU.begin()) {  // Assuming IMU.begin() returns false on failure
		Serial.println("IMU initialization failed");
		// thank GOD this shit never ran because .begin() is an int
		while (1) {
			delay(10);  // Prevents watchdog timer resets and allows serial messages to be seen
		}
	}

	Serial.println("IMU Initialized successfully");

	// Initialize energy integrator timebase
	lastEnergySampleMs = millis();
}
 
void loop() {
	// Apply remote fan speed
	// Drive fan via LEDC PWM
	// LEDC WHY NOT
	//ledcWrite(0, fanSpeed);
	// GET METRICS | UPDATE IMU?
	IMU.getAGT();
	// UPDATE VCS?
	ina226.readAndClearFlags();
	ina226_fc.readAndClearFlags();
	int time = 40;
	float Voltage = ina226.getBusVoltage_V();
	float Current = ina226.getCurrent_mA();
	float Power = ina226.getBusPower();
	// AHH OH NO
	if (!ina226.overflow) {
		Serial.println("Values OK - no overflow");
	} else {
		Serial.println("Overflow! Choose higher current range");
	}

	unsigned long currentMillis = millis();

	// Integrate energies (Wh): Wh += (V * A) * (h)
	unsigned long dtMs = currentMillis - lastEnergySampleMs;
	lastEnergySampleMs = currentMillis;
	// Battery channel (Current in mA -> A)
	// I AM NOT BOTHERED WIRTH C++ CAST DUMBNESS
	energyWh += (Voltage * (Current / 1000.0f)) * (static_cast<double>(dtMs) / 3600000.0);
	// Fuel-cell channel
	float fcV = ina226_fc.getBusVoltage_V();
	//MICROAMPS????? 
	float fcI_A = ina226_fc.getCurrent_mA() / 1000.0f;
	hydroWh += (fcV * fcI_A) * (static_cast<double>(dtMs) / 3600000.0);

	if (currentMillis - previousMillis >= purgeInterval) { // Every 50s, or time (interval)
		previousMillis = currentMillis;
		analogWrite(D1, 255); 
		analogWrite(D0, 255); 
		delay(purgeTime);  // Open Valve 2 for purgeTime ms
		analogWrite(D1, 0);
		analogWrite(D0, 0); 
		int purgetime = currentMillis - previousMillis;
		myData.message = String("FC CLEARED (") + purgetime + " ms )";
		messageStartTime = millis();  // Record the time when the message was updated
	} else {
		analogWrite(D1, 0);
		analogWrite(D0, 0); 
	}


	// Read raw divider voltage
	int rawTherm = analogRead(A2);
	float Vout = rawTherm;
	Serial.print("Divider Voltage [V]: ");
	Serial.println(Vout);
	// Include A1 voltage in transmitted data
	myData.a1Voltage = Vout;

	/*Serial.print("Load Voltage [V]: "); Serial.println(Voltage);
	Serial.print("Current[mA]: ");    Serial.println(Current);
	Serial.print("Bus Power [mW]: ");  Serial.println(Power);
	Serial.print(ina226.getBusVoltage_V());
	Serial.print(",");
	Serial.print(ina226_fc.getBusVoltage_V());
	Serial.print(",");
	Serial.println(ina226_fc.getCurrent_mA() / 1000.0);*/

	// Set values to send
	myData.ACCX = IMU.accX();
	myData.ACCY = IMU.accY();
	myData.ACCZ = IMU.accZ();
	myData.GYRX = IMU.gyrX();
	myData.GYRY = IMU.gyrY();
	myData.GYRZ = IMU.gyrZ();
	myData.TEMP = IMU.temp();
	myData.Voltage = Voltage;
	myData.Current = Current;
	myData.Power = Power;
	myData.ElapsedTime = millis();
	myData.battVoltage = ina226.getBusVoltage_V();
	myData.fcVoltage = ina226_fc.getBusVoltage_V();
	myData.fcCurrent = ina226_fc.getCurrent_mA() / 1000.0;
	myData.uptime      = millis();
	myData.energyWh    = (float)energyWh; // Added
	myData.hydroWh     = (float)hydroWh;  // New

	// Optional debug
	Serial.print("Energy [Wh]: ");
	Serial.println(myData.energyWh, 6);

	// Read latest voltages for control logic
	float battV = ina226.getBusVoltage_V();

	// Pin 10: set to 100 if battery voltage exceeds 1.0 V, else 0
	if (battV > 1.0) {
		analogWrite(D10, 50);
	} else {
		analogWrite(D10, 1);
	}

	// Pin 9: set to 100 if FC voltage exceeds batt voltage by at least 0.1 V, else 0
	if (fcV > battV + 0.1) {
		analogWrite(D9, 50);
	} else {
		analogWrite(D9, 1);
	}
	
	if (myData.message != "" && (millis() - messageStartTime >= displayDuration)) {
		myData.message = "";  // Clear the message
	}
	// Send message via ESP-NOW
	esp_err_t result = esp_now_send(broadcastAddress, (uint8_t *) &myData, sizeof(myData));
	
	if (result == ESP_OK) {
		Serial.println("Sent with success");
	}
	else {
		Serial.println("Error sending the data");
	}
	delay(20);
}
