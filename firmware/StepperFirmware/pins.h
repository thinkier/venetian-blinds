struct PinDefs {
    uint8_t x;
    uint8_t y;
    uint8_t z;
    uint8_t e;
};

PinDefs enn = {12, 7, 2, 15};
PinDefs step = {11, 6, 19, 14};
PinDefs dir = {10, 5, 28, 13};
PinDefs diag = {4, 3, 25, 16};
PinDefs uart_addr = {0, 2, 1, 3};

void setup_pins(PinDefs &pins, uint8_t mode) {
    pinMode(pins.x, mode);
    pinMode(pins.y, mode);
    pinMode(pins.z, mode);
    pinMode(pins.e, mode);
}
