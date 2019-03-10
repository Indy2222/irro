# !/usr/bin/env bash

set -e

# Manual Steps
# ============

# lsb_release is install in Python 3.5 version and supposes that `python3`
# points to python3.5 which we are going to change. This can be easily fixed by
# modifying shebang (first line) of `/usr/bin/lsb_release` to point to
# `python3.5` explicitly.

# So we can install apt packages from fresh index later.
sudo apt-get update

# Python 3.7.2
# ============

cd ~/downloads
wget https://www.python.org/ftp/python/3.7.2/Python-3.7.2.tar.xz
tar xf Python-3.7.2.tar.xz
rm Python-3.7.2.tar.xz

# Install build dependencies of Python 3.7.2. Note that some of the
# dependencies are needed for build only. They are not cleared because they are
# generally handy and a lot of other software depends on them.
sudo apt-get install -y \
     build-essential tk-dev libncurses5-dev libncursesw5-dev libreadline6-dev \
     libdb5.3-dev libgdbm-dev libsqlite3-dev libssl-dev libbz2-dev \
     libexpat1-dev liblzma-dev zlib1g-dev libffi-dev

cd Python-3.7.2
./configure
make -j 4
sudo make altinstall
sudo rm /usr/bin/python3
sudo ln -s /usr/local/bin/python3.7 /usr/bin/python3

# Make sure that it was actually installed.
python3 --version | grep 3.7.2

# Clean-up after yourself!
cd ..
# Some of the files were created during `make install` stage which is why it
# must be removed with sudo.
sudo rm -rf Python-3.7.2

# pip3
# ====

sudo python3 -m easy_install pip

# VLC
# ===

# This will install VLC without X dependencies.
sudo apt-get install -y vlc-nox
sudo apt-get install -y ffmpeg
