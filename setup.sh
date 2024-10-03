#!/bin/bash

sudo apt-get install -y -qq libasound2-dev libssl-dev libpulse-dev libdbus-1-dev portaudio19-dev librust-alsa-sys-dev
echo "Please set up a defualt path for your music"
read musicpath
if [[ "${musicpath: -1}" != "\\" && "${musicpath: -1}" != "/" ]]; then
    if [[ "$os_type" == "CYGWIN_NT" ]]; then
        musicpath="${musicpath}\\\\"
    else
        musicpath="${musicpath}/"
    fi
fi

echo "MUSICPATH=\"$musicpath\"" > .env
cargo build
