#! /usr/bin/env python3

import math
import time
from contextlib import contextmanager
from io import BytesIO
from threading import Thread

import numpy as np
import picamera
from flask import Flask, send_file
from PIL import Image

app = Flask(__name__)
camera = None


class CameraThread(Thread):

    def __init__(self, width, height):
        super().__init__()

        self._width = width
        self._height = height

        raw_width = int(math.ceil(self._width / 32) * 32)
        raw_height = int(math.ceil(self._height / 16) * 16)
        self._last_image = np.empty((raw_height, raw_width, 3), dtype=np.uint8)

    def run(self):
        with self._init_camera() as camera:
            while True:
                a = time.time()
                camera.capture(self._last_image, 'rgb')
                b = time.time()
                print(f'dt = {b - a}')

    @property
    def last_image(self):
        if self._last_image is None:
            raise Exception('No image has been taken yet.')

        return self._last_image[:self._height, :self._width, :]

    def _init_camera(self):
        camera = picamera.PiCamera(
            resolution=(self._width, self._height),
            framerate=24,
        )
        time.sleep(2)
        return camera


def main():
    global camera

    camera = CameraThread(720, 480)
    camera.start()

    app.run(host='0.0.0.0', port=8080, debug=False, threaded=False)


@app.route("/camera.png")
def hello():
    a = time.time()
    image_rgb = camera.last_image
    stream = BytesIO()
    Image.fromarray(image_rgb).save(stream, 'PNG', compress_level=0)
    stream.seek(0)
    b = time.time()
    print(f'dt2 = {b - a}')
    return send_file(stream, mimetype='image/png')


if __name__ == '__main__':
    main()
