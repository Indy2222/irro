// EN are PWM output pins regulating motor power supply (via a MOSFET
// transistors)
#define MOTOR_L_EN 10
#define MOTOR_R_EN 11

// The following pins control motor direction, via H-bridges.
// See https://irro.cz/hw.html
//
// +------+------+----------+
// | IN1  | IN2  | State    |
// +------+------+----------+
// | HIGH | LOW  | forward  |
// | LOW  | HIGH | backward |
// | LOW  | LOW  | braking  |
// +------+------+----------+
#define MOTOR_L_IN1 2
#define MOTOR_L_IN2 4
#define MOTOR_R_IN1 7
#define MOTOR_R_IN2 8

int currentLedMask = 0;

void setup() {
  Serial.begin(115200);

  pinMode(LED_BUILTIN, OUTPUT);

  pinMode(MOTOR_L_EN, OUTPUT);
  pinMode(MOTOR_R_EN, OUTPUT);
  pinMode(MOTOR_L_IN1, OUTPUT);
  pinMode(MOTOR_L_IN2, OUTPUT);
  pinMode(MOTOR_R_IN1, OUTPUT);
  pinMode(MOTOR_R_IN2, OUTPUT);
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

    responseLen = 0;

    if (cmd == 0x0000) {
      ledMask(payload, payloadLen);
    } else if (cmd == 0x0001) {
      responseLen = readLeds(response);
    } else if (cmd == 0x0100) {
      setMotorsPowerRatioCmd(payload, payloadLen);
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

void setMotorsPowerRatioCmd(byte *payload, int len) {
  // Cut off power to the motors first so we can freely play with the H-bridge.
  analogWrite(MOTOR_L_EN, 0);
  analogWrite(MOTOR_R_EN, 0);

  setMotorPowerRatio(payload, MOTOR_L_EN, MOTOR_L_IN1, MOTOR_L_IN2);
  setMotorPowerRatio((payload + 2), MOTOR_R_EN, MOTOR_R_IN1, MOTOR_R_IN2);
}

void setMotorPowerRatio(byte *payload, int enPin, int in1Pin, int in2Pin) {
  int value = payload[0] << 8 | payload[1];

  // Outputs have only 8 bits of precision: toss away most significant big
  // (sign -- used only for H-bride direction) and 7 lest significant bits.
  //
  // This value is further changed if `value` is negative, see below.
  int en_pin_value = (value >> 7) & 0xff;

  if (value == 0) {
    // Turn off motor H-bridge.
    digitalWrite(in1Pin, LOW);
    digitalWrite(in2Pin, LOW);
  } else if (value > 0) {
    // Put the H-bridge to forward direction.
    digitalWrite(in2Pin, LOW);
    digitalWrite(in1Pin, HIGH);
  } else {
    digitalWrite(in1Pin, LOW);
    digitalWrite(in2Pin, HIGH);

    // `value` was negative, we need to compute two's complement. Also, +1
    // below is done to compensate for different "min-max" value range for
    // negative and positive integers.
    en_pin_value = ~en_pin_value;
    if (value > -16384) {
      en_pin_value += 1;
    }
  }

  analogWrite(enPin, en_pin_value);
}
