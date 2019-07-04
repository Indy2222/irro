FROM ubuntu:18.04
LABEL maintainer="Martin Indra <martin.indra@mgn.cz>"

RUN apt-get update \
        && apt-get install -y \
        build-essential \
        curl \
        python3 \
        python3-pip \
        gcc-arm-linux-gnueabihf \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH "/root/.cargo/bin/:$PATH"
RUN rustup component add rustfmt clippy-preview
RUN rustup target add armv7-unknown-linux-gnueabihf
ENV HOME /root
RUN echo "[target.armv7-unknown-linux-gnueabihf]\nlinker = \"arm-linux-gnueabihf-gcc\"" >> $HOME/.cargo/config

RUN mkdir /root/build
WORKDIR /root/build

ADD docs/requirements.txt .
RUN pip3 install -r requirements.txt
