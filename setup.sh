#!/bin/bash
read -p "Would you like to install the necessary packages? (y/n): " input

if [[ "$input" == "y" || "$input" == "Y" ]]; then
    sudo apt-get install -y -qq libasound2-dev libssl-dev libpulse-dev libdbus-1-dev portaudio19-dev librust-alsa-sys-dev
    sudo apt-get install -y -qq libgl1-mesa-glx libegl1-mesa xorg-dev
else
    echo "Installation skipped."
fi
cargo build --release
