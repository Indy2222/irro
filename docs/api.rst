===
API
===

Irro is running REST API on port 8080. Irro is regularly broadcasting a UDP
datagram on port 34254, which could be used for its discovery on a LAN.


.. http:get:: /low/led

   Retrieve current LED on/off states.

   **Example request**:

   .. sourcecode:: http

      PUT /low/led HTTP/1.1
      Host: irro.local
      Accept: application/json

   **Example response**:

   .. sourcecode:: http

      HTTP/1.1 200 OK
      Content-Type: application/json

      [true, false, false, false, false, false, false, false]

   :>json list: List of booleans.


.. http:put:: /low/led/(int:led_id)

   Turn on/off an LED `led_id`.

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
