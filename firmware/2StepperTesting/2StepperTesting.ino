#include <TMC2209.h>

#define IRUN 100

uint8_t enn_e = 15;
uint8_t enn_x = 12;

TMC2209 drv_e;
TMC2209 drv_x;

void setup() {
    drv_e.setup(Serial2, 115200, TMC2209::SerialAddress::SERIAL_ADDRESS_3);
    drv_e.setRunCurrent(IRUN);
    drv_e.setHardwareEnablePin(enn_e);
    drv_e.setMicrostepsPerStepPowerOfTwo(1);
    
    drv_x.setup(Serial2, 115200, TMC2209::SerialAddress::SERIAL_ADDRESS_0);
    drv_x.setRunCurrent(IRUN);
    drv_x.setHardwareEnablePin(enn_x);
    drv_x.setMicrostepsPerStepPowerOfTwo(1);

    drv_e.enable();
    drv_x.enable();

    delay(3000);

    drv_e.moveAtVelocity(1000);
    drv_x.moveAtVelocity(1000);
}

void loop() {
}
