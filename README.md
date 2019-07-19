Irro
=====

[![Travis status](https://travis-ci.org/Indy2222/irro.svg?branch=master)](https://travis-ci.org/Indy2222/irro)
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This is my personal robot project and a very first robot I am building. Please
visit [irro.cz](https://irro.cz) to see this project's documentation.

This project is free software, see its [license](/LICENSE).

Repository Structure
--------------------

* [**/.travis.yml**](/.travis.yml) – [Travis CI](https://travis-ci.org/)
  configuration.
* [**/android**](/android) – Client to the robot (remote control) written as an
  [Android](https://www.android.com/) application in
  [Kotlin](https://kotlinlang.org/).
* [**/arduino**](/arduino) – [Arduino](https://www.arduino.cc/) source codes.
* [**/ci**](/ci) – continuous integration related files.
* [**/docs**](/docs) – [Sphinx](https://www.sphinx-doc.org) documentation.
* [**/irroctl**](/irroctl) – Command line client and test toolkit for Irro
  written in [Rust](https://www.rust-lang.org/).
* [**/raspberry**](/raspberry) – setup scripts, systemd unit files, and similar
  for Robot's onboard Raspberry Pi.
* [**/rust**](/rust) – [Rust](https://www.rust-lang.org/) source codes, most of
  which is running on the robot's onboard
  [Raspberry Pi](https://www.raspberrypi.org/).
* [**/version.txt**](/version.txt) – a single-line file with repository wide
  version of the robot.
