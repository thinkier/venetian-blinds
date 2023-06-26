#include <TMC2209.h>

#define IRUN 100

uint8_t enn_x = 12;
uint8_t enn_y = 7;
uint8_t enn_e = 15;

TMC2209 drv_x;
TMC2209 drv_y;
TMC2209 drv_e;

void setup() {
    drv_x.setup(Serial2, 115200, TMC2209::SerialAddress::SERIAL_ADDRESS_0);
    drv_x.setRunCurrent(IRUN);
    drv_x.setHardwareEnablePin(enn_x);
    drv_x.setMicrostepsPerStepPowerOfTwo(0);

    drv_y.setup(Serial2, 115200, TMC2209::SerialAddress::SERIAL_ADDRESS_2);
    drv_y.setRunCurrent(IRUN);
    drv_y.setHardwareEnablePin(enn_y);
    drv_y.setMicrostepsPerStepPowerOfTwo(0);

    drv_e.setup(Serial2, 115200, TMC2209::SerialAddress::SERIAL_ADDRESS_3);
    drv_e.setRunCurrent(IRUN);
    drv_e.setHardwareEnablePin(enn_e);
    drv_e.setMicrostepsPerStepPowerOfTwo(0);

    drv_x.enable();
    drv_y.enable();
    drv_e.enable();

    delay(3000);

    drv_x.moveAtVelocity(500);
    drv_y.moveAtVelocity(500);
    drv_e.moveAtVelocity(500);
}

void loop() {
    delay(10000);
    drv_x.disable();
    drv_y.disable();
    drv_e.disable();
}
