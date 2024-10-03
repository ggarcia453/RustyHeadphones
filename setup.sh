#!/bin/bash
read -p "Would you like to install the necessary packages? (y/n): " input

if [[ "$input" == "y" || "$input" == "Y" ]]; then
    sudo apt-get install -y -qq libasound2-dev libssl-dev libpulse-dev libdbus-1-dev portaudio19-dev librust-alsa-sys-dev
else
    echo "Installation skipped."
fi

read  -p "Please set up a defualt path for your music: " musicpath
if [[ "${musicpath: -1}" != "\\" && "${musicpath: -1}" != "/" ]]; then
    if [[ "$os_type" == "CYGWIN_NT" || "$os_type" == "MINGW32_NT" || "$musicpath" == *"\\\\"* ]]; then
        musicpath="${musicpath}\\\\"
    else
        musicpath="${musicpath}/"
    fi
fi

echo "MUSICPATH=\"$musicpath\"" > .env
cargo build
