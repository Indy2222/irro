===
API
===

Irro is running REST API on port 8080. Irro is regularly broadcasting a UDP
datagram on port 34254, which could be used for its discovery on a LAN.


.. http:get:: /low/led

   Retrieve current LED on/off states. See :ref:`hw.leds`.

   **Example request**:

   .. sourcecode:: http

      GET /low/led HTTP/1.1
      Host: irro.local
      Accept: application/json

   **Example response**:

   .. sourcecode:: http

      HTTP/1.1 200 OK
      Content-Type: application/json

      [true, false, false, false, false, false, false, false]

   :>json list: List of booleans.


.. http:put:: /low/led/(int:led_id)

   Turn on/off an LED `led_id`. See :ref:`hw.leds`.

   **Example request**:

   .. sourcecode:: http

      PUT /low/led/0 HTTP/1.1
      Host: irro.local
      Accept: application/json

      true

   **Example response**:

   .. sourcecode:: http

      HTTP/1.1 200 OK
      Content-Type: application/json

      null

   :param led_id: ID of the LED
   :<json boolean: Request body is a single boolean. When ``true`` the LED is
                   turned on, it is turned off otherwise.


.. http:post:: /low/motor/power/ratio

   Set output power ratio to left and right motors.

   **Example request**:

   .. sourcecode:: http

      POST /low/motor/ratio HTTP/1.1
      Host: irro.local
      Accept: application/json

      {
          "left": 0.2,
          "right: 0.15
      }

   **Example response**:

   .. sourcecode:: http

      HTTP/1.1 200 OK
      Content-Type: application/json

      null

   :>json float left: A number between -1 (full power backwards) and 1 (full
       power forward).
   :>json float right: See left.
