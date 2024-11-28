use std::fs::File;
use std::io::BufReader;
use tokio::time::sleep;
use rodio::{Decoder, Sink, OutputStreamHandle};
use std::time::Duration;
use rand::thread_rng;
use std::sync::{Arc, Mutex};
use crate::operations;
use tokio::sync::mpsc::{Sender, Receiver};
use rand::prelude::SliceRandom;

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Play(Vec<String>),
    Pause,
    Stop,
    Skip,
    Back,
    Queue(Vec<String>),
    Shuffle,
    VolumeChanger(Vec<String>),
    Mute,
    Unmute,
    Restart,
    SetLoop(Option<String>),
    SetSpeed(Vec<String>),
    Help,
    Exit,
}
fn play_song( s: & String, sink: &Sink)-> Result<(), Box<dyn std::error::Error>>{
   let f = File::open(s).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
   let file = BufReader::new(f);
   let source = Decoder::new(file).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
   sink.append(source);
   Ok(())
}
pub struct RustyHeadphones{
    sender: Sender<AudioCommand>,
}
impl RustyHeadphones {
    pub fn new (sender: Sender<AudioCommand>) -> Self{
        Self{
            sender
        }
    }
    pub async fn send_command(&self, cmd: AudioCommand) {
        if let Err(e) = self.sender.send(cmd).await {
            println!("Failed to send command: {}", e);
        }
    }
}

pub async fn player_thread(mut receiver: Receiver<AudioCommand>,_stream_handle: Arc<Mutex<OutputStreamHandle>>, sink: Arc<Mutex<Sink>>, defpath:String){
    let mut handler = operations::Handler::new(defpath);
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) => {
                if let Ok(sink) = sink.lock() {
                    if sink.empty(){
                        if handler.cur_song.is_some(){
                            handler.stack.push(handler.cur_song.as_ref().unwrap().clone());
                        }
                        match handler.islooping{
                            operations::Loop::NoLoop => {
                                if !handler.queue.is_empty(){
                                    let next_song = handler.queue.get(0);
                                    match play_song(next_song.as_ref().unwrap(), &sink){
                                        Ok(_) => {
                                            handler.cur_song = Some(next_song.unwrap().to_owned());
                                        },
                                        Err(_)=>{
                                            handler.cur_song = None;
                                            println!("Error playing song :/")
                                        }
                                    };
                                    handler.queue.remove(0);
                                }
                                else{
                                    handler.cur_song = None;
                                }
                            },
                            operations::Loop::LoopSong => {
                                if handler.cur_song.is_some(){
                                    match play_song(handler.cur_song.as_ref().unwrap(), &sink){
                                        Err(_)=>{
                                            handler.cur_song = None;
                                            println!("Error playing song :/");
                                        }, 
                                        _ => (),
                                    };
                                }
                                else if !handler.queue.is_empty(){
                                    let nextsong = handler.queue.get(0).unwrap().to_owned();
                                    match play_song(&nextsong, &sink){
                                        Ok(_) =>{
                                            handler.cur_song = Some(nextsong);
                                        },
                                        Err(_) =>{
                                            handler.cur_song = None;
                                            println!("Error playing song :/")
                                        }
                                    }
                                    handler.queue.remove(0);
                                    
                                }
                            },
                            operations::Loop::LoopQueue => {
                                if handler.cur_song.is_some(){
                                    let s = handler.cur_song.clone().unwrap();
                                    handler.queue.push(s);
                                }
                                if !handler.queue.is_empty(){
                                    let nextsong = handler.queue.get(0).unwrap().to_owned();
                                    match play_song(&nextsong, &sink){
                                        Ok(_) => {
                                            handler.cur_song = Some(nextsong);
                                        },
                                        Err(_)=>{
                                            handler.cur_song = None;
                                            println!("Error playing song :/")
                                        }
                                    };
                                    handler.queue.remove(0);
                                }
                            },
                        }
                        if handler.cur_song.as_ref().is_some() &&  handler.islooping != operations::Loop::LoopSong{
                            println!("Now Playing {}", handler.cur_song.as_ref().unwrap().clone());
                        }
                    }
                }
            },
            Some(cmd) = receiver.recv() => {
                match cmd {
                    AudioCommand::Exit => break,
                    AudioCommand::Stop => {
                        if let Ok(sink) = sink.lock(){
                            sink.stop();
                            handler.cur_song = None;
                            handler.queue.clear();
                            handler.stack.clear();
                        }
                    },
                    AudioCommand::Pause=>{
                        if let Ok(sink) = sink.lock(){
                            sink.pause();
                        }
                    },
                    AudioCommand::Play(options)=>{
                        let s: Vec<&str>= options.iter().map(|s| s.as_str()).collect();
                        if let Ok(sink) = sink.lock(){
                            handler.play_handle(&sink, s);
                        }
                    },
                    AudioCommand::Shuffle=>{
                        let mut rng = thread_rng(); 
                        handler.queue.shuffle(& mut rng);
                    },
                    AudioCommand::Skip=>{
                        if let Ok(sink) = sink.lock(){
                            handler.skip_handle(&sink);
                        }
                    },
                    AudioCommand::Queue(song_opt)=>{
                        let s: Vec<&str>= song_opt.iter().map(|s| s.as_str()).collect();
                        match handler.queue_handle(s) {
                            Err(_) =>{
                                println!("Could not find \"{}\". Verify it exists or path is correct\nIf you want to queue the default directory put \".\" as the path. ", song_opt.join(" "));
                            },
                            _ => ()
                        }
                    },
                    AudioCommand::VolumeChanger(options)=>{
                        if let Ok(sink) = sink.lock(){
                            match options.first(){
                                Some(op)=>{
                                    match op.as_str() {
                                        "view" => println!("Playing at {:.0}", sink.volume() * 100.0 ),
                                        "set" => {
                                            if options.get(1).is_some(){
                                                let val: Result<f32,_> = options.get(1).unwrap().parse();
                                                match val{
                                                    Err(_) => println!("Error: volume should be set to between 0 and 100"),
                                                    Ok(volume) => {
                                                        if 0.0 <= volume && volume <= 100.0{
                                                            sink.set_volume(volume/100.0 as f32)
                                                        }
                                                        else{
                                                            println!("Error: volume should be set to between 0 and 100")
                                                        }
                                                    },
                                                }
                                            }else{
                                                println!("Error: volume should be set to between 0 and 100");
                                            }
                                            let volume: f32 = sink.volume() as f32;
                                            println!("Volume is at {:.0}", 100.0 * volume);
                                        },
                                        "down" | "Down" => {
                                            let cur_volume = sink.volume();
                                            if cur_volume > 0.05{
                                                sink.set_volume(cur_volume - 0.05);
                                            }
                                            else{
                                                sink.set_volume(0.0);
                                            }
                                            let volume: f32 = sink.volume() as f32;
                                            println!("Volume is at {:.0}", 100.0 * volume);
                                        }
                                        "up" | "Up" => {
                                            let cur_volume = sink.volume();
                                            if cur_volume < 0.95{
                                                sink.set_volume(cur_volume + 0.05);
                                            }
                                            else{
                                                sink.set_volume(1.0);
                                            }
                                            let volume: f32 = sink.volume() as f32;
                                            println!("Volume is at {:.0}", 100.0 * volume);
                                        }
                                        x => println!("Cannot find volume {}", x),
                                    }
                                }, 
                                _ => println!("Requires second option"),
                            }
                        }
                    }
                    AudioCommand::SetLoop(newloop)=>{
                        match newloop{
                            Some(i)=> handler.loop_handle(&i),
                            _ => println!("No Loop option asked for!")
                        }
                    }
                    AudioCommand::Restart=>{
                        if let Ok(sink) = sink.lock(){
                            if handler.cur_song.is_some(){
                                sink.skip_one();
                                let s = handler.cur_song.clone();
                                match play_song(s.as_ref().unwrap(), &sink){
                                    Ok(_)=> println!("Restarting"),
                                    Err(_) =>{
                                        handler.cur_song = None;
                                        println!("Error playing song :/")
                                    }
                                };
                            }
                            else{
                                println!("No song to restart");
                            }
                        }
                    },
                    AudioCommand::Mute=>{
                        if let Ok(sink) = sink.lock(){
                            handler.mute(&sink);
                        }
                    },
                    AudioCommand::Unmute=>{
                        if let Ok(sink) = sink.lock(){
                            handler.unmute(&sink);
                        }
                    },
                    AudioCommand::Back=>{
                        if let Ok(sink) = sink.lock(){
                            handler.back_handle(&sink);
                        }
                    },
                    AudioCommand::SetSpeed(options) =>{
                        if let Ok(sink) = sink.lock(){
                            match options.first(){
                                Some(op) => {
                                    match op.as_str(){
                                        "view" => println!("Playing at {:.2}x speed", sink.speed()),
                                        "set" => {
                                            if options.get(1).is_some(){
                                                let val: Result<f32,_> = options.get(1).unwrap().parse();
                                                match val{
                                                    Err(_) => println!("Error: volume should be set to between 0 and 100"),
                                                    Ok(speed) => {
                                                        if 0.01 <= speed{
                                                            sink.set_speed(speed);
                                                        }
                                                        else{
                                                            println!("Error: volume should be set to between 0 and 100");
                                                        }
                                                    },
                                                }
                                            }else{
                                                println!("Error: volume should be set to between 0 and 100");
                                            }
                                            let speed: f32 = sink.speed() as f32;
                                            println!("Now at {:.2}x speed", speed);
                                        },
                                        "down" | "Down" => {
                                            let cur_speed = sink.speed();
                                            if cur_speed > 0.05{
                                                sink.set_speed(cur_speed - 0.05);
                                            }
                                            else{
                                                sink.set_speed(0.01);
                                            }
                                            let speed: f32 = sink.speed();
                                            println!("Now at {:.2}x speed", speed);
                                        }
                                        "up" | "Up" => {
                                            let cur_speed = sink.speed();
                                            sink.set_speed(cur_speed + 0.05);
                                            let speed: f32 = sink.speed();
                                            println!("Now at {:.2}x speed", speed);
                                        }
                                        x => println!("Cannot find speed {}", x)
                                    }
                                },
                                None => ()
                            }
                        }
                    }
                    AudioCommand::Help=>{
                        println!("Here is a comphrensive list of all the commands you can use!");
                        println!("exit -> exits the program");
                        println!("stop -> stops playing music, resets queue and song history");
                        println!("pause -> pauses the music");
                        println!("play -> plays the music");
                        println!("play <file/directory> -> plays a file or directory as requested. It skips the current queue and is played immedaitely");
                        println!("shuffle -> shuflles current queue");
                        println!("skip -> skips current song");
                        println!("queue [shuffle] <file/directory> -> queues a file or directory as requested. It places it at the end of the current queue. You can optionally add shuffle to shuffle the directory being queued");
                        println!("volume view -> view volume 0 - 100");
                        println!("volume set <number> -> sets volume to number (0-100)");
                        println!("volume <up/down> -> move volume up or down");
                        println!("loop <song/queue/cancel> -> set the loop option. loop song -> loop song only, loop queue -> loop song + queue, loop cancel =-> cancels all looping");
                        println!("restart -> restarts current song from beginning");
                        println!("back -> play previous song");
                        println!("mute -> mute music");
                        println!("unmute -> unmute music");
                        println!("help -> Display help menu");
                    }
                }
            }
        }
    }
}
