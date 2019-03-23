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

* ``0x00`` (turn an LED off/on) -- this command turns on or off LEDs. Has one
  byte payload which is a bit LED on/off bit mask (1 for on, 0 for off). Most
  significant bit corresponds to LED number 0.

.. TODO link to documentation of which LED is which
