# Overview 
Welcome to Rusty Headphones! This is a terminal based audio player created by me in Rust. It allows you to play local audio files from the terminal with all of the functionality of an audio player such as Groovy or Spotify. 
# Setup 
To setup rusty headphones, we have provided a setup shell script. Here you can enter a working directory that serves as the default for your music libraries. From there, RustyHeadphones will be installed and you can begin using it 
# Libraries
This is a list of installed libarires needed for the rust crate [rodio](https://docs.rs/rodio/latest/rodio/) for audio processing. They may be installed via the setup shell script provided. 
```
libasound2-dev 
libssl-dev 
libpulse-dev 
libdbus-1-dev 
portaudio19-dev 
librust-alsa-sys-dev
```

# Future Plans
Features planning to be added
1. Path Sense/Autocomplete (Similar to other terminals)
2. Integrate song streaming with Spotify or some other service
3. GUI mode