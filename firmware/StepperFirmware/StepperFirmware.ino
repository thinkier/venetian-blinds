#include <pico/multicore.h>
#include <pico/sync.h>
#include <pico/time.h>
#include <TMC2209.h>

#define RUN_CURRENT_MA 1600
#define DEFAULT_STALL_GUARD_THRESHOLD 10

critical_section_t *stepper_vars;

struct Steps {
    int32_t x;
    int32_t y;
    int32_t z;
    int32_t e;
};

struct PinDefs {
    uint8_t x;
    uint8_t y;
    uint8_t z;
    uint8_t e;
};

Steps target_steps = {0, 0, 0, 0};
Steps pending_steps = {0, 0, 0, 0};
uint8_t stall_bitflag = 0b0000;

PinDefs enn = {12, 7, 2, 15};
PinDefs step = {11, 6, 19, 14};
PinDefs dir = {10, 5, 28, 13};
PinDefs diag = {4, 3, 25, 16};
PinDefs uart_addr = {0, 2, 1, 3};

TMC2209 driver_x;
TMC2209 driver_y;
TMC2209 driver_z;
TMC2209 driver_e;

void setup() {
    // Setup SerialUSB
    Serial.begin(2000000);

    // Setup Pins
    Serial.println("Setting up pins");
    setup_pins(enn, OUTPUT);
    setup_pins(step, OUTPUT);
    setup_pins(dir, OUTPUT);
    setup_pins(diag, INPUT);

    // Setup TMC2209 UART Driver
    Serial.println("Setting up stepper motor drivers");
    setup_driver(driver_x, uart_addr.x);
    setup_driver(driver_y, uart_addr.y);
    setup_driver(driver_z, uart_addr.z);
    setup_driver(driver_e, uart_addr.e);

    // Setup threads and concurrency
    Serial.println("Setting up concurrency");
    critical_section_init(stepper_vars);
    multicore_launch_core1(stepper_setup);
    Serial.println("Setup completed");
}

void setup_pins(PinDefs &pins, uint8_t mode) {
    pinMode(pins.x, mode);
    pinMode(pins.y, mode);
    pinMode(pins.z, mode);
    pinMode(pins.e, mode);
}

void setup_driver(TMC2209 &driver, uint8_t addr) {
    TMC2209::SerialAddress uart_addr;

    switch (addr) {
        case 1:
            uart_addr = TMC2209::SerialAddress::SERIAL_ADDRESS_1;
            break;
        case 2:
            uart_addr = TMC2209::SerialAddress::SERIAL_ADDRESS_2;
            break;
        case 3:
            uart_addr = TMC2209::SerialAddress::SERIAL_ADDRESS_3;
            break;
        default:
            uart_addr = TMC2209::SerialAddress::SERIAL_ADDRESS_0;
    }

    driver.setup(Serial2, 115200, uart_addr);
    driver.setRunCurrent(RUN_CURRENT_MA / 20); // 2A Peak on TMC2209, function takes 0-100 percentage
    stepper_driver.setMicrostepsPerStep(2);
    stepper_driver.setHardwareEnablePin(pinEnn);
    stepper_driver.moveUsingStepDirInterface();
    stepper_driver.setCoolStepDurationThreshold((1 << 20) - 1);
    driver.setStallGuardThreshold(DEFAULT_STALL_GUARD_THRESHOLD);
}

void stepper_setup() {
    absolute_time_t now = get_absolute_time();

    // 2000 steps per second on a reliable clock (hopefully)
    while (true) {
        critical_section_enter_blocking(stepper_vars);
        stepper_loop_re();
        critical_section_exit(stepper_vars);

        absolute_time_t fall = delayed_by_us(now, 500);
        absolute_time_t next = delayed_by_us(now, 500 * 2);

        sleep_until(fall);
        stepper_loop_fe();

        sleep_until(next);
        now = next;
    }
}

void stepper_loop_re() {
    // Template for x series
    if (digitalRead(diag.x) == HIGH) {
        pending_steps.x = 0;
        stall_bitflag |= 1 << uart_addr.x;
    } else if (pending_steps.x > 0) {
        digitalWrite(dir.x, HIGH);
        digitalWrite(step.x, HIGH);
        pending_steps.x--;
    } else if (pending_steps.x < 0) {
        digitalWrite(dir.x, LOW);
        digitalWrite(step.x, HIGH);
        pending_steps.x++;
    }

    // Template for y series
    if (digitalRead(diag.y) == HIGH) {
        pending_steps.y = 0;
        stall_bitflag |= 1 << uart_addr.y;
    } else if (pending_steps.y > 0) {
        digitalWrite(dir.y, HIGH);
        digitalWrite(step.y, HIGH);
        pending_steps.y--;
    } else if (pending_steps.y < 0) {
        digitalWrite(dir.y, LOW);
        digitalWrite(step.y, HIGH);
        pending_steps.y++;
    }

    // Template for z series
    if (digitalRead(diag.z) == HIGH) {
        pending_steps.z = 0;
        stall_bitflag |= 1 << uart_addr.z;
    } else if (pending_steps.z > 0) {
        digitalWrite(dir.z, HIGH);
        digitalWrite(step.z, HIGH);
        pending_steps.z--;
    } else if (pending_steps.z < 0) {
        digitalWrite(dir.z, LOW);
        digitalWrite(step.z, HIGH);
        pending_steps.z++;
    }

    // Template for e series
    if (digitalRead(diag.e) == HIGH) {
        pending_steps.e = 0;
        stall_bitflag |= 1 << uart_addr.e;
    } else if (pending_steps.e > 0) {
        digitalWrite(dir.e, HIGH);
        digitalWrite(step.e, HIGH);
        pending_steps.e--;
    } else if (pending_steps.e < 0) {
        digitalWrite(dir.e, LOW);
        digitalWrite(step.e, HIGH);
        pending_steps.e++;
    }
}

void stepper_loop_fe() {
    digitalWrite(step.x, LOW);
    digitalWrite(step.y, LOW);
    digitalWrite(step.z, LOW);
    digitalWrite(step.e, LOW);
}

bool debug = false;

void loop() {
    Serial.println("Enter the command DEBUG to enable stepper diagnostics.");
    if (Serial.available()) {
        String str = Serial.readStringUntil('\n');
        str.trim();
        str.toUpperCase();

        if (str.equals("DEBUG")) {
            Serial.println("Enabled debug mode");
            debug = true;
        }
    }
    // TODO Handle commands from Serial1
    // TODO Handle responses to Serial1

    // TODO If SerialUSB commands, print diagnostic info (adds load to Serial2)

    // TODO Use main loop to drive the ENN pin, as ENN also resets DIAG and we need that diagnostic info on the comms thread

    delay(100); // Prevent revving the CPU too much
}
