#include <TMC2209.h>

HardwareSerial &serial_stream = Serial2;

const long SERIAL_BAUD_RATE = 2000000;
const int DELAY_US = 125;
// current values may need to be reduced to prevent overheating depending on
// specific motor and power supply voltage
const uint8_t RUN_CURRENT_PERCENT = 90;
const uint8_t SGTHRS = 100;

const uint8_t pinEnn = 12;
const uint8_t pinStep = 11;
const uint8_t pinDiag = 4;


// Instantiate TMC2209
TMC2209 stepper_driver;


void setup() {
    pinMode(pinDiag, INPUT_PULLUP);
    pinMode(pinStep, OUTPUT);
    Serial.begin(SERIAL_BAUD_RATE);

    stepper_driver.setup(serial_stream);

    stepper_driver.setRunCurrent(RUN_CURRENT_PERCENT);
    stepper_driver.setStallGuardThreshold(SGTHRS);
    stepper_driver.setMicrostepsPerStep(2);
    stepper_driver.setHardwareEnablePin(pinEnn);
    stepper_driver.enable();
    stepper_driver.moveUsingStepDirInterface();
    stepper_driver.setCoolStepDurationThreshold((1 << 20) - 1);
}

void loop() {
//  for(int i = 0; i < 4; i++){
    digitalWrite(pinStep, HIGH);
    delayMicroseconds(DELAY_US);
    digitalWrite(pinStep, LOW);
    delayMicroseconds(DELAY_US);
//  }
    debug();
}

void debug() {
    if (not stepper_driver.isSetupAndCommunicating()) {
        Serial.println("Stepper driver not setup and communicating!");
        return;
    }

    Serial.print("SGTHRS2: ");
    Serial.print(2 * (uint16_t) SGTHRS);
    Serial.print(", ");

    uint16_t SG_RESULT = stepper_driver.getStallGuardResult();
    Serial.print("SG_RESULT: ");
    Serial.print(SG_RESULT);
    Serial.print(", ");

    Serial.print("DIAG: ");
    Serial.print(digitalRead(pinDiag) ? 100 : 0);

    Serial.println();
}
