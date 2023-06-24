#include <pico/stdlib.h>
#include <pico/time.h>
#include <pico/multicore.h>
#include <hardware/pwm.h>
#include <TMC2209.h>

HardwareSerial &serial_stream = Serial2;

const long SERIAL_BAUD_RATE = 2000000;
// current values may need to be reduced to prevent overheating depending on
// specific motor and power supply voltage
// const uint8_t RUN_CURRENT_PERCENT = 66; // Just slightly more than stall at 250 steps/s
const uint8_t RUN_CURRENT_PERCENT = 80;
const uint8_t SGTHRS = 10;

const uint8_t pinEnn = 12;
const uint8_t pinStep = 11;
const uint8_t pinDiag = 4;


// Instantiate TMC2209
TMC2209 stepper_driver;
absolute_time_t now;

void setup() {
    pinMode(pinDiag, INPUT_PULLUP);
    Serial.begin(SERIAL_BAUD_RATE);

    stepper_driver.setup(serial_stream);

    stepper_driver.setRunCurrent(RUN_CURRENT_PERCENT);
    stepper_driver.setStallGuardThreshold(SGTHRS);
    stepper_driver.setMicrostepsPerStepPowerOfTwo(1);
    stepper_driver.setHardwareEnablePin(pinEnn);
    stepper_driver.enable();
    stepper_driver.setCoolStepDurationThreshold((1 << 20) - 1);
    // stepper_driver.moveAtVelocity(500);

    stepper_driver.moveUsingStepDirInterface();
    setupPwm();
    // multicore_launch_core1(stepDelayTask);

    now = get_absolute_time();
}

void setupPwm() {
    gpio_set_function(pinStep, GPIO_FUNC_PWM);

    uint slice = pwm_gpio_to_slice_num(pinStep);
    uint chan = pwm_gpio_to_channel(pinStep);

    // Set sysclock to 120MHz
    set_sys_clock_khz(120000, true);
    // sysclk / 120 = 1MHz
    pwm_set_clkdiv(slice, 120);
    pwm_set_clkdiv_mode(slice, pwm_clkdiv_mode::PWM_DIV_FREE_RUNNING);

    // 1MHz pwm clock (divided)
    // 1M / 2k = 500Hz
    pwm_set_wrap(slice, 2000);
    pwm_set_chan_level(slice, chan, 999);

    // // 1M / 1k = 1kHz
    // pwm_set_wrap(slice, 1000);
    // pwm_set_chan_level(slice, chan, 499);

    pwm_set_enabled(slice, true);
}

void stepDelayTask() {
    pinMode(pinStep, OUTPUT);
    while (true) {
        digitalWrite(pinStep, HIGH);
        delayMicroseconds(1000);
        digitalWrite(pinStep, LOW);
        delayMicroseconds(1000);
    }
}

void loop() {
    if (not stepper_driver.isSetupAndCommunicating()) {
        Serial.println("Stepper driver not setup and communicating!");
        return;
    }

    Serial.print("2*SGTHRS:");
    Serial.print(2 * (uint16_t) SGTHRS);
    Serial.print(",");

    uint16_t SG_RESULT = stepper_driver.getStallGuardResult();
    Serial.print("SG_RESULT:");
    Serial.print(SG_RESULT);
    Serial.print(",");

    Serial.print("DIAG:");
    Serial.print(digitalRead(pinDiag) ? 100 : 0);

    Serial.println();

    absolute_time_t next = delayed_by_ms(now, 10);
    sleep_until(next);
    now = next;
}
