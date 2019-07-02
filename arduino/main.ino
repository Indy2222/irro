void setup() {
  Serial.begin(115200);

  pinMode(LED_BUILTIN, OUTPUT);
}

void loop() {
  while (Serial.available() >= 4) {
    // Read all commands first before continuing to do other work. The buffer
    // is only 64 bytes, better to read everything as often as possible.
    int cmd = readInt();
    int len = readInt();
    byte payload[len];
    Serial.readBytes(payload, len);

    if (cmd == 0) {
      ledMask(payload, len);
    }

    // The response has 0 length.
    Serial.write(0);
    Serial.write(0);
  }

  // Other logic will be placed in a [short] loop here.
}

// Read 2 byte int from serial port. Do not call this method if there is less
// than 2 bytes available in the buffer.
int readInt() {
  int cmd = Serial.read() << 8;
  cmd |= Serial.read();
  return cmd;
}

void ledMask(byte *payload, int len) {
  int mask = payload[0];
  setLed(mask, 0, LED_BUILTIN);
}

void setLed(int mask, int ledNum, int ledPin) {
  int isOn = (mask >> (7 - ledNum)) & 0x01;
  if (isOn) {
    digitalWrite(ledPin, HIGH);
  } else {
    digitalWrite(ledPin, LOW);
  }
}
