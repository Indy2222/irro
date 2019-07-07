============
Raspberry Pi
============

 * Install_ Ubuntu 18.04 on the Raspberry Pi 3 B+.

.. _Install: https://wiki.ubuntu.com/ARM/RaspberryPi

 * Add my ssh key to ``/home/ubuntu/.ssh/authorized_keys``

 * Disable password login in ``/etc/ssh/sshd_config`` by setting
   ``PasswordAuthentication no``.

 * Setup Irro user

.. code-block:: bash

   adduser irro
   # So the user has access to Arduino serial port.
   usermod -a -G dialout irro

 * Add ``/raspberry/irro.service`` to ``/etc/systemd/system/irro.service`` and
   call ``systemctl enable irro``.

 * Make journald logs persistent: open ``/etc/systemd/journald.conf`` and set
   ``Storage=persistent``.
