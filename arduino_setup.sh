#!/bin/bash

sudo apt-get install --yes git gcc gcc-arm-none-eabi

curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | sh
source ~/.profile

arduino-cli core --additional-urls https://github.com/earlephilhower/arduino-pico/releases/download/global/package_rp2040_index.json search rp2040
arduino-cli core --additional-urls https://github.com/earlephilhower/arduino-pico/releases/download/global/package_rp2040_index.json install rp2040:rp2040

arduino-cli lib install tmc2209

