#include <pico/multicore.h>
#include <pico/sync.h>
#include <pico/time.h>

#include <Adafruit_NeoPixel.h>
#include <TMC2209.h>

#include "steps.h"
#include "pins.h"

#define RUN_CURRENT_MA 1600
#define DEFAULT_STALL_GUARD_THRESHOLD 10

critical_section_t *stepper_vars;

TMC2209 driver_x;
TMC2209 driver_y;
TMC2209 driver_z;
TMC2209 driver_e;

#define PIXELS 1
Adafruit_NeoPixel pixels(PIXELS, 24, NEO_GRB + NEO_KHZ800);

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

    // Setup NeoPixel
    pixels.begin();

    Serial.println("Setup completed");
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
    driver.setMicrostepsPerStep(1); // Full stepping
    driver.moveUsingStepDirInterface();
    driver.setCoolStepDurationThreshold((1 << 20) - 1);
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

void handleDriveData() {
    critical_section_enter_blocking(stepper_vars);

    if (pending_steps.x == 0 && target_steps.x != 0) {
        digitalWrite(enn.x, HIGH);
        Serial1.print("MX ");
        Serial1.print(target_steps.x);
        target_steps.x = 0;

        if (stall_bitflag & (1 << uart_addr.x)) {
            Serial1.print(" STALLED");
            stall_bitflag &= ~(1 << uart_addr.x);
        }

        Serial1.println();
    }

    if (pending_steps.y == 0 && target_steps.y != 0) {
        digitalWrite(enn.y, HIGH);
        Serial1.print("MY ");
        Serial1.print(target_steps.y);
        target_steps.y = 0;

        if (stall_bitflag & (1 << uart_addr.y)) {
            Serial1.print(" STALLED");
            stall_bitflag &= ~(1 << uart_addr.y);
        }

        Serial1.println();
    }

    if (pending_steps.z == 0 && target_steps.z != 0) {
        digitalWrite(enn.z, HIGH);
        Serial1.print("MZ ");
        Serial1.print(target_steps.z);
        target_steps.z = 0;

        if (stall_bitflag & (1 << uart_addr.z)) {
            Serial1.print(" STALLED");
            stall_bitflag &= ~(1 << uart_addr.z);
        }

        Serial1.println();
    }

    if (pending_steps.e == 0 && target_steps.e != 0) {
        digitalWrite(enn.e, HIGH);
        Serial1.print("ME ");
        Serial1.print(target_steps.e);
        target_steps.e = 0;

        if (stall_bitflag & (1 << uart_addr.e)) {
            Serial1.print(" STALLED");
            stall_bitflag &= ~(1 << uart_addr.e);
        }

        Serial1.println();
    }
}

bool debugCmdHandler() {
    if (Serial.available()) {
        String debug_cmd = Serial.readStringUntil('\n');
        debug_cmd.trim();
        debug_cmd.toUpperCase();

        if (debug_cmd.equals("DEBUG")) {
            debug = !debug;
            Serial.print("Toggled debug mode: debug=");
            Serial.println(debug);

            if (debug) {
                for (int i = 0; i < PIXELS; i++) {
                    pixels.setPixelColor(i, pixels.Color(0, 32, 0));
                }
            } else {
                for (int i = 0; i < PIXELS; i++) {
                    pixels.setPixelColor(i, pixels.Color(0, 0, 0));
                }
            }
        } else if (debug_cmd.equals("TEST")) {
            Serial.println("Spinning all motors by 200 steps.");

            digitalWrite(enn.x, LOW);
            digitalWrite(enn.y, LOW);
            digitalWrite(enn.z, LOW);
            digitalWrite(enn.e, LOW);

            critical_section_enter_blocking(stepper_vars);
            target_steps.x = 200;
            pending_steps.x = 200;
            target_steps.y = 200;
            pending_steps.y = 200;
            target_steps.z = 200;
            pending_steps.z = 200;
            target_steps.e = 200;
            pending_steps.e = 200;
            critical_section_exit(stepper_vars);
        } else {
            Serial.print("Unknown command: ");
            Serial.println(debug_cmd);
        }

        return true;
    }

    return false;
}

bool steppingCmdHandler() {
    if (Serial1.available()) {
        String cmd = Serial1.readStringUntil('\n');
        cmd.trim();
        cmd.toUpperCase();

        if (cmd.startsWith("INT")) {
            char motor = cmd.charAt(3);
            int32_t steps;

            critical_section_enter_blocking(stepper_vars);
            switch (motor) {
                case 'X':
                    steps = target_steps.x - pending_steps.x;
                    target_steps.x = 0;
                    pending_steps.x = 0;
                    break;
                case 'Y':
                    steps = target_steps.y - pending_steps.y;
                    target_steps.y = 0;
                    pending_steps.y = 0;
                    break;
                case 'Z':
                    steps = target_steps.z - pending_steps.z;
                    target_steps.z = 0;
                    pending_steps.z = 0;
                    break;
                default:
                    steps = target_steps.e - pending_steps.e;
                    target_steps.e = 0;
                    pending_steps.e = 0;
            }
            critical_section_exit(stepper_vars);

            Serial1.print("INT");
            Serial1.print(motor);
            Serial1.print(" ");
            Serial1.println(steps);
        } else if (cmd.startsWith("M")) {
            char motor = cmd.charAt(1);
            int32_t steps = cmd.substring(3).toInt();

            critical_section_enter_blocking(stepper_vars);
            switch (motor) {
                case 'X':
                    digitalWrite(enn.x, LOW);
                    target_steps.x += steps;
                    pending_steps.x += steps;
                    break;
                case 'Y':
                    digitalWrite(enn.y, LOW);
                    target_steps.y += steps;
                    pending_steps.y += steps;
                    break;
                case 'Z':
                    digitalWrite(enn.z, LOW);
                    target_steps.z += steps;
                    pending_steps.z += steps;
                    break;
                default:
                    digitalWrite(enn.e, LOW);
                    target_steps.e += steps;
                    pending_steps.e += steps;
            }
            critical_section_exit(stepper_vars);
        } else if (cmd.startsWith("SGTHRS")) {
            char motor = cmd.charAt(6);
            uint8_t sgthrs = cmd.substring(8).toInt();

            char usedMotor;
            switch (motor) {
                case 'X':
                    usedMotor = 'X';
                    driver_x.setStallGuardThreshold(sgthrs);
                    break;
                case 'Y':
                    usedMotor = 'Y';
                    driver_y.setStallGuardThreshold(sgthrs);
                    break;
                case 'Z':
                    usedMotor = 'Z';
                    driver_z.setStallGuardThreshold(sgthrs);
                    break;
                default:
                    usedMotor = 'E';
                    driver_e.setStallGuardThreshold(sgthrs);
            }

            Serial1.print("SGTHRS");
            Serial1.print(usedMotor);
            Serial1.print(" ");
            Serial1.println(sgthrs);
        } else {
            Serial.print("Unknown command received on UART: ");
            Serial.println(cmd);
        }

        return true;
    }

    return false;
}

void printDiagnosticsInformation() {}

void loop() {
    handleDriveData();

    if (debugCmdHandler()) {
        return;
    }

    if (steppingCmdHandler()) {
        return;
    }

    if (debug) {
        printDiagnosticsInformation();
    }

    delay(100); // Prevent revving the CPU too much
}
