int currentLedMask = 0;

void setup() {
  Serial.begin(115200);

  pinMode(LED_BUILTIN, OUTPUT);
}

void loop() {
  int cmd;

  byte payload[64];
  int payloadLen;

  byte response[64];
  int responseLen;

  while (Serial.available() >= 4) {
    // Read all commands first before continuing to do other work. The buffer
    // is only 64 bytes, better to read everything as often as possible.
    cmd = readInt();
    payloadLen = readInt();
    Serial.readBytes(payload, payloadLen);

    if (cmd == 0) {
      ledMask(payload, payloadLen);
      responseLen = 0;
    } else if (cmd == 1) {
      responseLen = readLeds(response);
    }

    Serial.write(0);
    Serial.write(responseLen);

    int i;
    for (i = 0; i < responseLen; i++) {
      Serial.write(response[i]);
    }
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

int readLeds(byte *response) {
  response[0] = currentLedMask;
  return 1;
}

void ledMask(byte *payload, int len) {
  currentLedMask = payload[0];
  setLed(currentLedMask, 0, LED_BUILTIN);
}

void setLed(int mask, int ledNum, int ledPin) {
  int isOn = (mask >> (7 - ledNum)) & 0x01;
  if (isOn) {
    digitalWrite(ledPin, HIGH);
  } else {
    digitalWrite(ledPin, LOW);
  }
}
