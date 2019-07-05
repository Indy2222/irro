============
Raspberry Pi
============

 * Setup Irro user

.. code-block:: bash

   adduser irro
   # So the user has access to Arduino serial port.
   usermod -a -G dialout irro

 * Add ``/raspberry/irro.service`` to ``/etc/systemd/system/irro.service`` and
   call ``systemctl enable irro``.

 * Make journald logs persistent: open ``/etc/systemd/journald.conf`` and set
   ``Storage=persistent``.
