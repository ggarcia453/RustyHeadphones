use std::{fmt, fs::File};
use std::io::BufReader;
use tokio::time::sleep;
use rodio::{Decoder, Sink, OutputStreamHandle};
use std::time::Duration;
use rand::thread_rng;
use std::sync::{Arc, Mutex};
use crate::operations;
use tokio::sync::mpsc::{Receiver, Sender};
use rand::prelude::SliceRandom;

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Play(Option<String>),
    Pause,
    Stop,
    Skip,
    Back,
    Queue(String),
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
impl fmt::Display for AudioCommand{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


fn play_song( s: & String, sink: &Sink)-> Result<(), Box<dyn std::error::Error>>{
   let f = File::open(s).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
   let file = BufReader::new(f);
   let source = Decoder::new(file).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
   sink.append(source);
   Ok(())
}
pub async fn player_thread(mut receiver: Receiver<AudioCommand>, sender: Sender<Option<String>>, _stream_handle: Arc<Mutex<OutputStreamHandle>>, sink: Arc<Mutex<Sink>>, defpath:String){
    fn sendprint(sender: & Sender<Option<String>>, s: String){
        if s.is_empty(){
            match sender.try_send(None) {
                _ => (),
            }
        }
        else{
            match sender.try_send(Some(s)){
            _ => (),
            }
        }
    }
    let mut handler = operations::Handler::new(defpath);
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) => {
                if let Ok(sink) = sink.lock() {
                    if sink.empty(){
                        sendprint(&sender, String::from("  "));
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
                                            sendprint(&sender,"Error playing song :/".to_string());
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
                                            sendprint(&sender,"Error playing song :/".to_string());
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
                                            sendprint(&sender,"Error playing song :/".to_string());
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
                                            sendprint(&sender,"Error playing song :/".to_string());
                                        }
                                    };
                                    handler.queue.remove(0);
                                }
                            },
                        }
                        if handler.cur_song.as_ref().is_some(){
                            sendprint(&sender,format!("Now Playing {}", handler.cur_song.as_ref().unwrap().clone()) );
                        }
                    }
                }
            },
            Some(cmd) = receiver.recv() => {
                match cmd {
                    AudioCommand::Exit => {
                        match sender.try_send(None){
                            _ => (),
                        };
                        break;
                    },
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
                        if let Ok(sink) = sink.lock(){
                            let ouput:String = handler.play_handle(&sink, options);
                            if !ouput.is_empty(){
                                sendprint(&sender, ouput);
                            }
                        }
                    },
                    AudioCommand::Shuffle=>{
                        let mut rng = thread_rng(); 
                        handler.queue.shuffle(& mut rng);
                    },
                    AudioCommand::Skip=>{
                        if let Ok(sink) = sink.lock(){
                            sendprint(&sender, handler.skip_handle(&sink));
                        }
                    },
                    AudioCommand::Queue(song_opt)=>{
                        let c = song_opt.clone();
                        match handler.queue_handle(song_opt) {
                            Err(_) =>{
                                sendprint(&sender, format!("Could not find \"{}\". Verify it exists or path is correct\nIf you want to queue the default directory put \".\" as the path. ", c));
                            },
                            Ok(s) => sendprint(&sender, s),
                        }
                    },
                    AudioCommand::VolumeChanger(options)=>{
                        if let Ok(sink) = sink.lock(){
                            match options.first(){
                                Some(op)=>{
                                    match op.as_str() {
                                        "view" => {
                                            let volume: f32 = sink.volume() as f32;
                                            sendprint(&sender, format!("Volume is at {:.0}", 100.0 * volume));
                                        },
                                        "set" => {
                                            if options.get(1).is_some(){
                                                let val: Result<f32,_> = options.get(1).unwrap().parse();
                                                match val{
                                                    Err(_) => {
                                                        sendprint(&sender, "Error: volume should be set to between 0 and 100".to_string());
                                                    },
                                                    Ok(volume) => {
                                                        if 0.0 <= volume && volume <= 100.0{
                                                            sink.set_volume(volume/100.0 as f32)
                                                        }
                                                        else{
                                                            sendprint(&sender, "Error: volume should be set to between 0 and 100".to_string());
                                                        }
                                                    },
                                                }
                                            }else{
                                                sendprint(&sender, "Error: volume should be set to between 0 and 100".to_string());
                                            }
                                            let volume: f32 = sink.volume() as f32;
                                            sendprint(&sender, format!("Volume is at {:.0}", 100.0 * volume));
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
                                            sendprint(&sender, format!("Volume is at {:.0}", 100.0 * volume));
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
                                            sendprint(&sender, format!("Volume is at {:.0}", 100.0 * volume));
                                        }
                                        x => {
                                            sendprint(&sender, format!("Cannot find volume {}", x));
                                        },
                                    }
                                }, 
                                _ =>{
                                    sendprint(&sender, "Requires second option".to_string());
                                } ,
                            }
                        }
                    }
                    AudioCommand::SetLoop(newloop)=>{
                        match newloop{
                            Some(i)=> sendprint(&sender, handler.loop_handle(&i)),
                            _ => sendprint(&sender,"No Loop option asked for!".to_string())
                        }
                    }
                    AudioCommand::Restart=>{
                        if let Ok(sink) = sink.lock(){
                            if handler.cur_song.is_some(){
                                sink.skip_one();
                                let s = handler.cur_song.clone();
                                match play_song(s.as_ref().unwrap(), &sink){
                                    Ok(_)=> sendprint(&sender, "Restarting".to_string()),
                                    Err(_) =>{
                                        handler.cur_song = None;
                                        sendprint(&sender,"Error playing song :/".to_string());
                                    }
                                };
                            }
                            else{
                                sendprint(&sender,"No song to restart".to_string());
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
                                        "view" => sendprint(&sender, format!("Playing at {:.2}x speed", sink.speed())),
                                        "set" => {
                                            if options.get(1).is_some(){
                                                let val: Result<f32,_> = options.get(1).unwrap().parse();
                                                match val{
                                                    Err(_) => sendprint(&sender, "Error: speed should be set to a number".to_string()),
                                                    Ok(speed) => {
                                                        if 0.01 <= speed{
                                                            sink.set_speed(speed);
                                                        }
                                                        else{
                                                            sendprint(&sender, "Error: speed should be set to a number".to_string());
                                                        }
                                                    },
                                                }
                                            }else{
                                                sendprint(&sender, "Error: speed should be set to a number".to_string());
                                            }
                                            let speed: f32 = sink.speed() as f32;
                                            sendprint(&sender, format!("Now at {:.2}x speed", speed));
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
                                            sendprint(&sender, format!("Now at {:.2}x speed", speed));
                                        }
                                        "up" | "Up" => {
                                            let cur_speed = sink.speed();
                                            sink.set_speed(cur_speed + 0.05);
                                            let speed: f32 = sink.speed();
                                            sendprint(&sender, format!("Now at {:.2}x speed", speed));
                                        }
                                        x => sendprint(&sender, format!("Cannot find speed {}", x)),
                                    }
                                },
                                None => ()
                            }
                        }
                    }
                    AudioCommand::Help=>{
                        sendprint(&sender, "Here is a comphrensive list of all the commands you can use!".to_string());
                        sendprint(&sender, "exit -> exits the program".to_string());
                        sendprint(&sender, "stop -> stops playing music, resets queue and song history".to_string());
                        sendprint(&sender, "pause -> pauses the music".to_string());
                        sendprint(&sender, "play -> plays the music".to_string());
                        sendprint(&sender, "play <file/directory> -> plays a file or directory as requested. It skips the current queue and is played immedaitely".to_string());
                        sendprint(&sender, "shuffle -> shuflles current queue".to_string());
                        sendprint(&sender, "skip -> skips current song".to_string());
                        sendprint(&sender, "queue [shuffle] <file/directory> -> queues a file or directory as requested. It places it at the end of the current queue. You can optionally add shuffle to shuffle the directory being queued".to_string());
                        sendprint(&sender, "volume view -> view volume 0 - 100".to_string());
                        sendprint(&sender, "volume set <number> -> sets volume to number (0-100)".to_string());
                        sendprint(&sender, "volume <up/down> -> move volume up or down".to_string());
                        sendprint(&sender, "loop <song/queue/cancel> -> set the loop option. loop song -> loop song only, loop queue -> loop song + queue, loop cancel =-> cancels all looping".to_string());
                        sendprint(&sender, "restart -> restarts current song from beginning".to_string());
                        sendprint(&sender, "back -> play previous song".to_string());
                        sendprint(&sender, "mute -> mute music".to_string());
                        sendprint(&sender, "unmute -> unmute music".to_string());
                        sendprint(&sender, "help -> Display help menu".to_string());
                    }
                }
            }
        }
    }
}
