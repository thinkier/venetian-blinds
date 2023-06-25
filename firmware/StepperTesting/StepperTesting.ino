#include <TMC2209.h>

const int DELAY = 100;

uint8_t enn = 15;
uint8_t step = 14;

TMC2209 drv_e;

void setup() {
    drv_e.setup(Serial2, 115200, TMC2209::SerialAddress::SERIAL_ADDRESS_3);
    drv_e.setRunCurrent(100);
    drv_e.setHardwareEnablePin(enn);
    drv_e.setMicrostepsPerStepPowerOfTwo(8);

    Serial.begin(115200);
    delay(1000);
    Serial.print("drv_e.isSetupAndCommunicating: ");
    Serial.println(drv_e.isSetupAndCommunicating());

    drv_e.enable();
    drv_e.moveAtVelocity(128000);
}

void loop() {
}
