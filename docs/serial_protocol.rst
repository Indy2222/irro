===============
Serial Protocol
===============

Robot's Raspberry Pi communicates with Arduino over a USB serial port with a
simple binary protocol. RPI sends commands and Arduino sends responses. Arduino
never writes any data on its own.

All data send over the serial port assume big-endian byte ordering.

Each command is initiated with two bytes identifying the particular command
followed by another two bytes indicating command payload length (which may
be 0) followed by the command specific payload.

Each response start with two bytes indicating response length (not including
the these bytes). Response length is 0 for commands which have no response.

RPI never writes more than 64 bytes of pending data (data of commands which
have not been responded yet), this is to avoid Arduino serial buffer overflow.

.. _serial.commands:

List of Commands
================

The commands are divided into groups. First command byte (most significant)
indicates group and second command byte (least significant) indicates the
command.

LED (0x00)
----------

See :ref:`hw.leds`.

* ``0x00`` (turn an LED off/on) -- this command turns on or off LEDs. Has one
  byte payload which is a bit LED on/off bit mask (1 for on, 0 for off). Most
  significant bit corresponds to LED number 0.

* ``0x01`` (read current LED mask) -- this command has no payload. Its response
  is one byte with current LED on/off states bit mask. See command ``0x00``.

.. _serial.commands.motor:

Motor (0x01)
------------

* ``0x00`` (set motors power ratio) -- this command sets left and right motor
  power to a value between -32,768 (max power in backward direction) to 32,767
  (max power in forward direction). 0 means that the motor is off. Note that
  maximum motor power is regulated by Arduino so a single motor pair never
  draws too much current.

  The command payload has 4 bytes, first two bytes (i16) are left motor power
  and the other two bytes are right motor power.

Examples
========

The following command sets left motor to maximum forward direction and right
motor to 25% backward direction.

* Command: ``01 00 00 04 7f ff df ff``

  #. Byte ``01`` indicates that the command belongs to
     :ref:`motor <serial.commands.motor>` group.
  #. Byte ``00`` indicates that this command sets motor power ratio.
  #. Bytes ``00 04`` indicates payload to be 4 bytes long.
  #. Bytes ``7f ff`` (32,767 in decimal) set the left motor to maximum forward
     power.
  #. Bytes ``df ff`` (-8193 in decimal) set the right motor 25% backward power.

* Response: ``00 00`` -- response with no payload.
