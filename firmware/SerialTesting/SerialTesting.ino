#include <Arduino.h>

const long SERIAL_BAUD_RATE = 115200;

void setup() {
    Serial.begin(SERIAL_BAUD_RATE);
    Serial1.begin(SERIAL_BAUD_RATE);
    Serial2.begin(SERIAL_BAUD_RATE);
}

void loop() {
    Serial.println("This is RP2040 on BTT SKR Pico v1.0 via SerialUSB");
    Serial1.println("This is RP2040 on BTT SKR Pico v1.0 via Serial1");
    Serial2.println("This is RP2040 on BTT SKR Pico v1.0 via Serial2");
    delay(1000);
}
