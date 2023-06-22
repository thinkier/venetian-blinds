#include <pico/multicore.h>
#include <pico/sync.h>
#include <pico/time.h>

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

PinDefs enn = {12, 7, 2, 15};
PinDefs step = {11, 6, 19, 14};
PinDefs dir = {10, 5, 28, 13};
PinDefs diag = {4, 3, 25, 16};
PinDefs uart_addr = {0, 2, 1, 3};

void setup() {
    Serial2.begin(115200);
    // TODO Setup TMC2209 UART Driver & pins

    critical_section_init(stepper_vars);
    multicore_launch_core1(stepper_setup);
}

void stepper_setup() {
    absolute_time_t now = get_absolute_time();

    // 2000 steps per second on a reliable clock (hopefully)
    while (true) {
        critical_section_enter_blocking(stepper_vars);
        stepper_loop();
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
    // TODO Engage relevant pins, read stallguard pin, etc.
}

void stepper_loop_fe() {
    // TODO Disengage step pin

    digitalWrite(step.x, LOW);
    digitalWrite(step.y, LOW);
    digitalWrite(step.z, LOW);
    digitalWrite(step.e, LOW);
}

void loop() {
    // TODO Handle commands from Serial1
    // TODO Handle responses to Serial1

    // TODO If SerialUSB commands, print diagnostic info (adds load to Serial2)

    delay(1); // Prevent revving the CPU too much
}