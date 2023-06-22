# G-code like communication protocol over UART

## SoC -> Microcontroller

### Move

- `M<u8 motor number> <i32 steps> [u8 SGTHRS=100]`
    - Engage motor to move the specified steps then disengage.

### Interrupt

- `INT<u8 motor number>`
    - Stop motor from current move command and return current status.

## Microcontroller -> SoC

### Move

- `M<u8 motor number> <i32 steps executed> [STALLED]`
    - Return number of steps executed after execution is complete.
    - Where the motor is interrupted by StallGuard4, the steps executed may be less than the amount requested, and the
      return text contains "STALLED".

### Interrupt

- `INT<u8 motor number> <i32 steps executed>`
    - Where there is no task in the queue return 0 for steps executed.
