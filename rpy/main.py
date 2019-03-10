#! /usr/bin/env python3

import io
import os
import socket
import time
from subprocess import PIPE, Popen
from threading import Thread

from picamera import PiCamera


class StreamEnded(Exception):
    pass


class UdpStream(Thread):

    def __init__(self, target_ip, target_port, mux):
        super().__init__()

        self.target_ip = target_ip
        self.target_port = target_port
        self.mux = mux
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    def run(self):
        with open('/home/indy/downloads/test2.h264', 'wb') as fp:
            print('Started...')
            while True:
                try:
                    buf = self.mux.read_chunk()
                except StreamEnded:
                    print('ended')
                    break

                print(f'Sending {len(buf)} bytes...')
                self.sock.sendto(buf, (self.target_ip, self.target_port))
                fp.write(buf)

        # TODO: self.converter.stdout.close()


class Mux:

    def __init__(self, framerate):
        # ffmpeg -f h264 -r 30 -i - -f matroska -vcodec copy - > test2.mkv
        print('Spawning background conversion process')
        self.converter = Popen([
            'ffmpeg',
            '-f', 'h264',
            '-r', str(framerate),
            '-i', '-',
            '-f', 'matroska',
            '-vcodec', 'copy',
            '-'],
            stdin=PIPE, stdout=PIPE, stderr=PIPE,
            shell=False, close_fds=True)

        self.exited = False

    def write(self, b):
        self.converter.stdin.write(b)

    def flush(self):
        print('Waiting for background conversion process to exit 22.')
        self.converter.stdin.close()
        self.converter.wait()

    def read_chunk(self):
        #print('Reading chunk: ')
        #print(self.converter.stderr.read1(100000))
        buf = self.converter.stdout.read1(32000)

        ret_code = self.converter.poll()
        if not self.exited and ret_code is not None:
            self.exited = True

        if ret_code:
            raise Exception(f'Process exited with {ret_code}.')

        if not buf and self.exited:
            raise StreamEnded()
        return buf


def main():
    framerate = 30

    mux = Mux(framerate)
    stream = UdpStream('192.168.0.199', 5005, mux)
    stream.start()

    with PiCamera() as camera:
        camera.resolution = (640, 480)
        camera.framerate = 30

        # Git the camera time to warm up.
        time.sleep(1)

        camera.start_recording(mux, format='h264', quality=23)
        print('1')
        camera.wait_recording(120)
        camera.stop_recording()


if __name__ == '__main__':
    main()
