#include <Adafruit_NeoPixel.h>

// Which pin on the Arduino is connected to the NeoPixels?
#define PIN 24

// How many NeoPixels are attached to the Arduino?
#define NUMPIXELS 1

Adafruit_NeoPixel pixels(NUMPIXELS, PIN, NEO_GRB + NEO_KHZ800);

#define DELAYVAL 500  // Time (in milliseconds) to pause between pixels

void setup() {
    Serial.begin(200000);
    pixels.begin();  // INITIALIZE NeoPixel strip object (REQUIRED)
}

bool toggle = false;

void loop() {
    pixels.clear();

    for (int i = 0; i < NUMPIXELS; i++) {
        if (toggle && i % 2 == 0) {
            pixels.setPixelColor(i, pixels.Color(0, 0, 64));
        } else {
            pixels.setPixelColor(i, pixels.Color(0, 0, 0));
        }
    }

    pixels.show();
    Serial.print("STATE:");
    Serial.println(toggle ? 1 : 0);
    Serial.print("STATE:");
    Serial.println(toggle ? 1 : 0);
    delay(DELAYVAL);
    toggle = !toggle;
}
