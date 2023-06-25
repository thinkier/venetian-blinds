# G-code like communication protocol over UART

## SoC -> Microcontroller

### StallGuard Configuration

- `SGTHRS<char motor id> <u8 SGTHRS>`
    - Set StallGuard4 threshold for the specified motor.

### Move

- `M<char motor id> <i32 steps>`
    - Engage motor to move the specified steps then disengage.

### Interrupt

- `INT<char motor id>`
    - Stop motor from current move command and return current status.

## Microcontroller -> SoC

### StallGuard Configuration

- `SGTHRS<char motor id> <u8 SGTHRS>`
    - Echo the response once it's set

### Move

- `M<char motor id> <i32 steps executed> [STALLED]`
    - Return number of steps executed after execution is complete.
    - Where the motor is interrupted by StallGuard4, the steps executed may be less than the amount requested, and the
      return text contains "STALLED".

### Interrupt

- `INT<char motor id> <i32 steps executed>`
    - Where there is no task in the queue return 0 for steps executed.
