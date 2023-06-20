# HomeKit Venetian Blinds
Powered by Raspberry Pi 4B 2GB HAP.rs driving Arduino on BTT SKR Pico v1.0.

## Hardware
- Raspberry Pi 4B 2GB
- [BTT SKR Pico v1.0](https://github.com/bigtreetech/SKR-Pico)

## Key Software
- [HAP.rs](https://docs.rs/hap/0.1.0-pre.15/hap/)
- [Arduino mbed OS for Raspberry Pi Pico](https://docs.arduino.cc/hardware/nano-rp2040-connect)
- [TMCStepper](https://github.com/teemuatlut/TMCStepper) based on [this stallguard example.](https://github.com/teemuatlut/TMCStepper/blob/master/examples/StallGuard_TMC2209/StallGuard_TMC2209.ino)
    - [TMC2209](https://github.com/janelia-arduino) should also work and is [less cryptic.](https://github.com/janelia-arduino/TMC2209/blob/main/examples/BidirectionalCommunication/StallGuard/StallGuard.ino)

## Research & Ideas Notes
- There's [board jumpers (sect 3. sensorless homing)](https://github.com/bigtreetech/SKR-Pico/blob/master/BTT%20SKR%20Pico%20V1.0%20Instruction%20Manual.pdf) to connect the TMC2209 DIAG (aka StallGuard interrupt) pin to the endstop pins (X/Y/Z & Filament Sensor)
  - DIAG pins needs to be pulled up according to BTT supplied Klipper config, active high 
- [BTT SKR Pico v1.0 Pin Diagram](https://github.com/bigtreetech/SKR-Pico/blob/master/Hardware/BTT%20SKR%20Pico%20V1.0-PIN.pdf)
- [BTT SKR Pico v1.0 Klipper config](https://github.com/bigtreetech/SKR-Pico/blob/master/Klipper/SKR%20Pico%20klipper.cfg)
  - TMC2209 MS1/MS2 address is documented in Klipper configuration. Pin documentation does not specify it.
- StealthChop is on by default (SpreadCycle off, mutually exclusive)
- StallGuard needs minor configuration to determine stall.
  - Measure SG_RESULT by cycling the entire venetian blind so I can get the torque measurement "load value" through the entire cycle of dropping the blinds, having it switch vertical tilt angle, then raising the entire thing. 
  - Calibrate SG_RESULT to prevent the vertical tilt adjustment operation from changing the vertical height.
- Logic is required for the following components:
  - Position state
  - Position cur/tgt
  - Vertical tilt angle cur/tgt
- For homing, it would try to retract the blinds 100% to the very top and wait until stallguard procs, no endstop switch required
  - When homed vertical tilt would be 90%
  - State does not need to be stored to nonvolatile memory. Self calibration works.
  - Trigger self calibration when fully opening instead of executing the exact amount of calculated steps.
- Measurements need to be made for steps-to-extend and steps-to-tilt. These will be stored in [Accessory.toml](./Accessory.toml)
- A 3D printed D-shaft doesn't hold its shape from experience. Use a [mounting hub](https://www.pololu.com/product/1998)
