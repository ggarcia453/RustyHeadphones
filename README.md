# Overview 
Welcome to Rusty Headphones! This is a terminal based audio player created by me in Rust. It allows you to play local audio files from the terminal with all of the functionality of an audio player such as Groovy or Spotify. 
# Setup 
To setup rusty headphones, we have provided a setup shell script. Here you can enter a working directory that serves as the default for your music libraries. From there, RustyHeadphones will be installed and you can begin using it 
# Libraries
This is a list of installed libarires needed for the rust crate 
<a href ="https://docs.rs/rodio/latest/rodio/" target="_blank">rodio</a>
 for audio processing. They may be installed via the setup shell script provided. 
```
libasound2-dev 
libssl-dev 
libpulse-dev 
libdbus-1-dev 
portaudio19-dev 
librust-alsa-sys-dev
```

There are also some libraries needed for the rust crate 
<a href="https://docs.rs/egui/0.29.1/egui/index.html" target= "_blank">egui</a>
 for gui creation. They are listed below.
 ```
 libgl1-mesa-glx
 libegl1-mesa
 xorg-dev
 ```

# Users Guide
The following section details how to use RustyHeadphones once all requirements have been installed and configured. 
## Default Path 
The default path can be configured with either the ```setup.sh``` script or manually by editing .env. If you choose not
to use the .env file or if you cargo install Rustyheadphones it will default to the current working directory. 
## Commands
You can find a list of commands below. For reference ```[a]``` indicates an optional argument "a", while ```(b)``` indicates a required argument "b".
|Command     |Action     |
|:------------|:------------|
| exit | This exits and quits the program.|
| stop | This will stop all music being played and clear the queue and listening history for the session.| 
| pause | This will pause the music or audio file currently being played.|
| play [file/directory] | Play has two modes. Without a file or directory, it will play the music if it is currently paused. With a file or directory, it will begin playing them immedialtey, skipping the current queue.|
| shuffle | Shuffle will shuffle the current queue.|
| skip | Skip will skip the current song being plaued.|
| queue [shuffle] (file/directory) | Queue adds the requested file or directory to the end of the queue. You may shuffle the directory order.|
| queue view | This allows you to view the current queue.|
| volume view | This will print the current volume (0 -100)|
| volume set (number) | Volume sets a new volume, which must be between 0 - 100.|
| volume (up/down) | Move the volume up or down as requested.|
| loop (song/queue/cancel) | This sets the looping option for the player. Loop song will loop the current/next song. Loop Queue will loop the current queue. Loop cancel will cancel all looping.|
| restart | Restarts the current song.|
| back | Plays the previous song.|
| mute | Mute the audio player.|
| unmute | Unmute the audio player.|
| volume set (number) | Will set the speed of the player to whatever new speed is.|
| speed (up/down) | This moves the speed up or down as requested.|
| speed view | This will print the current speed.|
| help | This will print the help version of the command list.|


You can also use help in RustyHeadphones to view a condensed version of the commands and their actions. 

# Features to be added
Features planning to be added
1. Setting default path outside of .env
2. Integrate song streaming with Spotify or some other service
3. GUI mode
4. Integrate support for more file formats (ex. m4a)

# Known Issues
This is a list of known issues currently with RustyHeadphones.
1. Printing Errors (Output does not print as intended).
2. Decoding errors with certain mp3 files.
3. Audio player glitches if computer enters sleep mode and must be reset.