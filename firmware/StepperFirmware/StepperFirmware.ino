#include <pico/multicore.h>
#include <pico/sync.h>
#include <pico/time.h>

#include <Adafruit_NeoPixel.h>
#include <TMC2209.h>

#include "steps.h"
#include "pins.h"

#define RUN_CURRENT_MA 2000
#define DEFAULT_STALL_GUARD_THRESHOLD 10

auto_init_mutex(lock);

TMC2209 driver_x;
TMC2209 driver_y;
TMC2209 driver_z;
TMC2209 driver_e;

#define PIXELS 1
Adafruit_NeoPixel pixels(PIXELS, 24, NEO_GRB + NEO_KHZ800);

bool debug = false;


void setup() {
    // Setup SerialUSB
    Serial.begin(2000000);

    // Setup Serial1 (UART)
    Serial1.begin(2000000);

    // Setup Pins
    Serial.println("Setting up pins");
    setup_pins(enn, OUTPUT, HIGH);
    setup_pins(step, OUTPUT);
    setup_pins(dir, OUTPUT);
    setup_pins(diag, INPUT_PULLUP);

    // Setup TMC2209 UART Driver
    Serial.println("Setting up stepper motor drivers");
    setup_driver(driver_x, enn.x, uart_addr.x);
    setup_driver(driver_y, enn.y, uart_addr.y);
    setup_driver(driver_z, enn.z, uart_addr.z);
    setup_driver(driver_e, enn.e, uart_addr.e);

    // Launch stepper setup thread
    Serial.println("Launching stepper thread");
    multicore_launch_core1(stepper_setup);

    // Setup NeoPixel
    pixels.begin();
    refreshNeopixel();

    Serial.println("Setup completed");
}

void refreshNeopixel() {
    if (debug) {
        for (int i = 0; i < PIXELS; i++) {
            pixels.setPixelColor(i, pixels.Color(0, 32, 0));
        }
    } else {
        for (int i = 0; i < PIXELS; i++) {
            pixels.setPixelColor(i, pixels.Color(0, 0, 0));
        }
    }
}

void setup_driver(TMC2209 &driver, uint8_t enn, uint8_t addr) {
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
    driver.setHardwareEnablePin(enn);
    driver.moveUsingStepDirInterface();
    driver.setCoolStepDurationThreshold((1 << 20) - 1);
    driver.setStallGuardThreshold(DEFAULT_STALL_GUARD_THRESHOLD);
}

void stepper_setup() {
    absolute_time_t now = get_absolute_time();

    // 2000 steps per second on a reliable clock (hopefully)
    while (true) {
        mutex_enter_blocking(&lock);
        stepper_loop_re();
        mutex_exit(&lock);

        absolute_time_t fall = delayed_by_us(now, 1000);
        absolute_time_t next = delayed_by_us(now, 2000);

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

void handleDriveData() {
    mutex_enter_blocking(&lock);

    if (pending_steps.x == 0 && target_steps.x != 0) {
        driver_x.disable();
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
        driver_y.disable();
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
        driver_z.disable();
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
        driver_e.disable();
        Serial1.print("ME ");
        Serial1.print(target_steps.e);
        target_steps.e = 0;

        if (stall_bitflag & (1 << uart_addr.e)) {
            Serial1.print(" STALLED");
            stall_bitflag &= ~(1 << uart_addr.e);
        }

        Serial1.println();
    }

    mutex_exit(&lock);
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

            refreshNeopixel();
        } else if (debug_cmd.equals("TEST")) {
            Serial.println("Spinning all motors by 200 steps.");

            mutex_enter_blocking(&lock);
            driver_x.enable();
            driver_y.enable();
            driver_z.enable();
            driver_e.enable();
            target_steps.x = 200;
            pending_steps.x = 200;
            target_steps.y = 200;
            pending_steps.y = 200;
            target_steps.z = 200;
            pending_steps.z = 200;
            target_steps.e = 200;
            pending_steps.e = 200;
            mutex_exit(&lock);
        } else if (debug_cmd.equals("TEST2")) {
            Serial.println("Spinning all motors using firmware at 500pps for 2 seconds.");

            mutex_enter_blocking(&lock);
            driver_x.enable();
            driver_y.enable();
            driver_z.enable();
            driver_e.enable();
            driver_x.moveAtVelocity(500);
            driver_y.moveAtVelocity(500);
            driver_z.moveAtVelocity(500);
            driver_e.moveAtVelocity(500);
            mutex_exit(&lock);

            delay(3000);

            Serial.println("Disengaging...");
            mutex_enter_blocking(&lock);
            driver_x.moveUsingStepDirInterface();
            driver_y.moveUsingStepDirInterface();
            driver_z.moveUsingStepDirInterface();
            driver_e.moveUsingStepDirInterface();
            driver_x.disable();
            driver_y.disable();
            driver_z.disable();
            driver_e.disable();
            mutex_exit(&lock);
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

            mutex_enter_blocking(&lock);
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
            mutex_exit(&lock);

            Serial1.print("INT");
            Serial1.print(motor);
            Serial1.print(" ");
            Serial1.println(steps);
        } else if (cmd.startsWith("M")) {
            char motor = cmd.charAt(1);
            int32_t steps = cmd.substring(3).toInt();

            mutex_enter_blocking(&lock);
            switch (motor) {
                case 'X':
                    driver_x.enable();
                    target_steps.x += steps;
                    pending_steps.x += steps;
                    break;
                case 'Y':
                    driver_y.enable();
                    target_steps.y += steps;
                    pending_steps.y += steps;
                    break;
                case 'Z':
                    driver_z.enable();
                    target_steps.z += steps;
                    pending_steps.z += steps;
                    break;
                default:
                    driver_e.enable();
                    target_steps.e += steps;
                    pending_steps.e += steps;
            }
            mutex_exit(&lock);
        } else if (cmd.startsWith("SGTHRS")) {
            char motor = cmd.charAt(6);
            uint8_t sgthrs = cmd.substring(8).toInt();

            mutex_enter_blocking(&lock);
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
            mutex_exit(&lock);

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

void printDiagnosticsInformation() {
    mutex_enter_blocking(&lock);
    Serial.print("X_SGRESULT:");
    Serial.print(driver_x.getStallGuardResult());
    Serial.print(",");

    Serial.print("Y_SGRESULT:");
    Serial.print(driver_y.getStallGuardResult());
    Serial.print(",");

    Serial.print("Z_SGRESULT:");
    Serial.print(driver_z.getStallGuardResult());
    Serial.print(",");

    Serial.print("E_SGRESULT:");
    Serial.print(driver_e.getStallGuardResult());
    Serial.print(",");

    Serial.println();
    mutex_exit(&lock);
}

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
