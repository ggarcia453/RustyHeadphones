#!/bin/bash

sudo apt-get install -y -qq libasound2-dev libssl-dev libpulse-dev libdbus-1-dev portaudio19-dev librust-alsa-sys-dev
echo "Please set up a defualt path for your music"
read musicpath
echo "MUSICPATH=\"$musicpath\"" > .env
cargo build
