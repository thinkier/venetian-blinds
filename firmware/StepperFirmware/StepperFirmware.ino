#include <hardware/watchdog.h>
#include <pico/multicore.h>
#include <pico/sync.h>
#include <pico/time.h>

#include <Adafruit_NeoPixel.h>
#include <TMC2209.h>

#include "steps.h"
#include "pins.h"

#define RUN_CURRENT_MA 2000
#define DEFAULT_STALL_GUARD_THRESHOLD 50
#define MICROSTEPS_LOG2 2
#define PHASE_PER_SECOND 250
#define CYCLE_US ((1000000 >> MICROSTEPS_LOG2) / PHASE_PER_SECOND)

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
    setup_pins(diag, INPUT_PULLDOWN);

    // Setup TMC2209 UART Driver
    Serial.println("Setting up stepper motor drivers");
    // setup_driver(driver_x, enn.x, uart_addr.x);
    // setup_driver(driver_y, enn.y, uart_addr.y);
    // setup_driver(driver_z, enn.z, uart_addr.z);
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
    driver.setMicrostepsPerStepPowerOfTwo(MICROSTEPS_LOG2);
    driver.setHardwareEnablePin(enn);
    driver.moveUsingStepDirInterface();
    driver.setCoolStepDurationThreshold((1 << 20) - 1);
    driver.setStallGuardThreshold(DEFAULT_STALL_GUARD_THRESHOLD);
}

void stepper_setup() {
    absolute_time_t now = get_absolute_time();

    // 2000 steps per second on a reliable clock (hopefully)
    while (true) {
        stepper_loop_diag();
        stepper_loop_re();

        absolute_time_t fall = delayed_by_us(now, CYCLE_US >> 1);
        absolute_time_t next = delayed_by_us(now, CYCLE_US);

        sleep_until(fall);
        stepper_loop_fe();

        sleep_until(next);
        now = next;
    }
}

void stepper_loop_diag() {
    if (digitalRead(diag.x) == HIGH) {
        mutex_enter_blocking(&lock);
//        pending_steps.x = 0;
        stall_bitflag |= 1 << uart_addr.x;
        mutex_exit(&lock);
    }

    if (digitalRead(diag.y) == HIGH) {
        mutex_enter_blocking(&lock);
//        pending_steps.y = 0;
        stall_bitflag |= 1 << uart_addr.y;
        mutex_exit(&lock);
    }

    if (digitalRead(diag.z) == HIGH) {
        mutex_enter_blocking(&lock);
//        pending_steps.z = 0;
        stall_bitflag |= 1 << uart_addr.z;
        mutex_exit(&lock);
    }

    if (digitalRead(diag.e) == HIGH) {
        mutex_enter_blocking(&lock);
//        pending_steps.e = 0;
        stall_bitflag |= 1 << uart_addr.e;
        mutex_exit(&lock);
    }
}

void stepper_loop_re() {
    // Template for x series
    if (pending_steps.x > 0) {
        digitalWrite(dir.x, HIGH);
        digitalWrite(step.x, HIGH);
        mutex_enter_blocking(&lock);
        pending_steps.x--;
        mutex_exit(&lock);
    } else if (pending_steps.x < 0) {
        digitalWrite(dir.x, LOW);
        digitalWrite(step.x, HIGH);
        mutex_enter_blocking(&lock);
        pending_steps.x++;
        mutex_exit(&lock);
    }

    // Template for y series
    if (pending_steps.y > 0) {
        digitalWrite(dir.y, HIGH);
        digitalWrite(step.y, HIGH);
        mutex_enter_blocking(&lock);
        pending_steps.y--;
        mutex_exit(&lock);
    } else if (pending_steps.y < 0) {
        digitalWrite(dir.y, LOW);
        digitalWrite(step.y, HIGH);
        mutex_enter_blocking(&lock);
        pending_steps.y++;
        mutex_exit(&lock);
    }

    // Template for z series
    if (pending_steps.z > 0) {
        digitalWrite(dir.z, HIGH);
        digitalWrite(step.z, HIGH);
        mutex_enter_blocking(&lock);
        pending_steps.z--;
        mutex_exit(&lock);
    } else if (pending_steps.z < 0) {
        digitalWrite(dir.z, LOW);
        digitalWrite(step.z, HIGH);
        mutex_enter_blocking(&lock);
        pending_steps.z++;
        mutex_exit(&lock);
    }

    // Template for e series
    if (pending_steps.e > 0) {
        digitalWrite(dir.e, HIGH);
        digitalWrite(step.e, HIGH);
        mutex_enter_blocking(&lock);
        pending_steps.e--;
        mutex_exit(&lock);
    } else if (pending_steps.e < 0) {
        digitalWrite(dir.e, LOW);
        digitalWrite(step.e, HIGH);
        mutex_enter_blocking(&lock);
        pending_steps.e++;
        mutex_exit(&lock);
    }
}

void stepper_loop_fe() {
    digitalWrite(step.x, LOW);
    digitalWrite(step.y, LOW);
    digitalWrite(step.z, LOW);
    digitalWrite(step.e, LOW);
}

void handleDriveData() {
    if (pending_steps.x == 0 && target_steps.x != 0) {
        driver_x.disable();
        Serial1.print("MX ");
        Serial1.print(target_steps.x >> MICROSTEPS_LOG2);
        target_steps.x = 0;

        if (stall_bitflag & (1 << uart_addr.x)) {
            Serial1.print(" STALLED");
            mutex_enter_blocking(&lock);
            stall_bitflag &= ~(1 << uart_addr.x);
            mutex_exit(&lock);
        }

        Serial1.println();
    }

    if (pending_steps.y == 0 && target_steps.y != 0) {
        driver_y.disable();
        Serial1.print("MY ");
        Serial1.print(target_steps.y >> MICROSTEPS_LOG2);
        target_steps.y = 0;

        if (stall_bitflag & (1 << uart_addr.y)) {
            Serial1.print(" STALLED");
            mutex_enter_blocking(&lock);
            stall_bitflag &= ~(1 << uart_addr.y);
            mutex_exit(&lock);
        }

        Serial1.println();
    }

    if (pending_steps.z == 0 && target_steps.z != 0) {
        driver_z.disable();
        Serial1.print("MZ ");
        Serial1.print(target_steps.z >> MICROSTEPS_LOG2);
        target_steps.z = 0;

        if (stall_bitflag & (1 << uart_addr.z)) {
            Serial1.print(" STALLED");
            mutex_enter_blocking(&lock);
            stall_bitflag &= ~(1 << uart_addr.z);
            mutex_exit(&lock);
        }

        Serial1.println();
    }

    if (pending_steps.e == 0 && target_steps.e != 0) {
        driver_e.disable();
        Serial1.print("ME ");
        Serial1.print(target_steps.e >> MICROSTEPS_LOG2);
        target_steps.e = 0;

        if (stall_bitflag & (1 << uart_addr.e)) {
            Serial1.print(" STALLED");
            mutex_enter_blocking(&lock);
            stall_bitflag &= ~(1 << uart_addr.e);
            mutex_exit(&lock);
        }

        Serial1.println();
    }
}

bool debugCmdHandler(HardwareSerial &port, String debug_cmd) {
    if (debug_cmd.equals("DEBUG")) {
        debug = !debug;
        port.print("Toggled debug mode: debug=");
        port.println(debug);

        refreshNeopixel();
    } else if (debug_cmd.equals("RESET")) {
        digitalWrite(enn.x, LOW);
        digitalWrite(enn.y, LOW);
        digitalWrite(enn.z, LOW);
        digitalWrite(enn.e, LOW);
        port.println("Engaged all motors for 100ms...");
        delay(100);
        digitalWrite(enn.x, HIGH);
        digitalWrite(enn.y, HIGH);
        digitalWrite(enn.z, HIGH);
        digitalWrite(enn.e, HIGH);
        port.println("Disengaged all motors.");
    } else if (debug_cmd.equals("HARDRESET")) {
        watchdog_enable(1, true);
    } else if (debug_cmd.equals("TEST")) {
        port.println("Spinning all motors by 200 steps.");

        driver_x.enable();
        driver_y.enable();
        driver_z.enable();
        driver_e.enable();
        target_steps.x = 200;
        target_steps.y = 200;
        target_steps.z = 200;
        target_steps.e = 200;
        mutex_enter_blocking(&lock);
        pending_steps.x = 200;
        pending_steps.y = 200;
        pending_steps.z = 200;
        pending_steps.e = 200;
        mutex_exit(&lock);
    } else if (debug_cmd.equals("TEST2")) {
        port.println("Spinning all motors using firmware at 500 for 2 seconds.");

        driver_x.moveAtVelocity(500);
        driver_y.moveAtVelocity(500);
        driver_z.moveAtVelocity(500);
        driver_e.moveAtVelocity(500);
        driver_x.enable();
        driver_y.enable();
        driver_z.enable();
        driver_e.enable();

        delay(2000);

        port.println("Disengaging...");
        driver_x.moveUsingStepDirInterface();
        driver_y.moveUsingStepDirInterface();
        driver_z.moveUsingStepDirInterface();
        driver_e.moveUsingStepDirInterface();
        driver_x.disable();
        driver_y.disable();
        driver_z.disable();
        driver_e.disable();
    } else {
        return false;
    }
    return true;
}

bool steppingCmdHandler(HardwareSerial &port, String cmd) {
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

        port.print("INT");
        port.print(motor);
        port.print(" ");
        port.println(steps);
    } else if (cmd.startsWith("M")) {
        char motor = cmd.charAt(1);
        int32_t steps = cmd.substring(3).toInt();

        switch (motor) {
            case 'X':
                driver_x.enable();

                mutex_enter_blocking(&lock);
                target_steps.x += steps << MICROSTEPS_LOG2;
                pending_steps.x += steps << MICROSTEPS_LOG2;
                mutex_exit(&lock);
                break;
            case 'Y':
                driver_y.enable();

                mutex_enter_blocking(&lock);
                target_steps.y += steps << MICROSTEPS_LOG2;
                pending_steps.y += steps << MICROSTEPS_LOG2;
                mutex_exit(&lock);
                break;
            case 'Z':
                driver_z.enable();

                mutex_enter_blocking(&lock);
                target_steps.z += steps << MICROSTEPS_LOG2;
                pending_steps.z += steps << MICROSTEPS_LOG2;
                mutex_exit(&lock);
                break;
            default:
                driver_e.enable();

                mutex_enter_blocking(&lock);
                target_steps.e += steps << MICROSTEPS_LOG2;
                pending_steps.e += steps << MICROSTEPS_LOG2;
                mutex_exit(&lock);
        }
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

        port.print("SGTHRS");
        port.print(usedMotor);
        port.print(" ");
        port.println(sgthrs);
    } else {
        return false;
    }
    return true;
}

bool isStalling(uint8_t uart_addr) {
    return (stall_bitflag >> uart_addr) & 1;
}

void printDiagnosticsInformation() {
    Serial.print("X_SGRESULT:");
    Serial.print(driver_x.getStallGuardResult());
    Serial.print(",");
    Serial.print("X_DIAG:");
    Serial.print(isStalling(uart_addr.x) * 100);
    Serial.print(",");

    Serial.print("Y_SGRESULT:");
    Serial.print(driver_y.getStallGuardResult());
    Serial.print(",");
    Serial.print("Y_DIAG:");
    Serial.print(isStalling(uart_addr.y) * 100);
    Serial.print(",");

    Serial.print("Z_SGRESULT:");
    Serial.print(driver_z.getStallGuardResult());
    Serial.print(",");
    Serial.print("Z_DIAG:");
    Serial.print(isStalling(uart_addr.z) * 100);
    Serial.print(",");

    Serial.print("E_SGRESULT:");
    Serial.print(driver_e.getStallGuardResult());
    Serial.print(",");
    Serial.print("E_DIAG:");
    Serial.print(isStalling(uart_addr.e) * 100);

    Serial.println();
}

String parseCommand(HardwareSerial &port) {
    String cmd = port.readStringUntil('\n');
    cmd.trim();
    cmd.toUpperCase();
    return cmd;
}

bool handlerSerialUSB() {
    if (Serial.available()) {
        String cmd = parseCommand(Serial);

        if (!(debugCmdHandler(Serial, cmd) || steppingCmdHandler(Serial, cmd))) {
            Serial.print("Unknown command: ");
            Serial.println(cmd);
        }

        return true;
    }

    return false;
}

bool handlerSerial1() {
    if (Serial1.available()) {
        String cmd = parseCommand(Serial1);

        if (!steppingCmdHandler(Serial1, cmd)) {
            Serial.print("Unknown command on UART: ");
            Serial.println(cmd);
        }

        return true;
    }

    return false;
}

void loop() {
    handleDriveData();

    if (handlerSerialUSB() || handlerSerial1()) {
        return;
    }

    if (debug) {
        printDiagnosticsInformation();
    }

    delay(100); // Prevent revving the CPU too much
}
