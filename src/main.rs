use std::fs::File;
use std::io::BufReader;
use rand::seq::SliceRandom;
use tokio::time::sleep;
use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use std::time::Duration;
use dotenv::dotenv;
extern crate rand;
use rand::thread_rng;
use rustyline::{CompletionType, Config, Editor};
use rustyline::error::ReadlineError;
use tokio::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
mod operations;
mod helpers;

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
    Exit,
}
struct RustyHeadphones{
    sender: Sender<AudioCommand>,
}
impl RustyHeadphones {
    fn new (sender: Sender<AudioCommand>) -> Self{
        Self{
            sender
        }
    }
    async fn send_command(&self, cmd: AudioCommand) {
        if let Err(e) = self.sender.send(cmd).await {
            println!("Failed to send command: {}", e);
        }
    }
}

fn play_song( s: & String, sink: &Sink)-> Result<(), Box<dyn std::error::Error>>{
   let f = File::open(s).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
   let file = BufReader::new(f);
   let source = Decoder::new(file).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
   sink.append(source);
   Ok(())
}

async fn player_thread(mut receiver: Receiver<AudioCommand>,_stream_handle: Arc<Mutex<OutputStreamHandle>>, sink: Arc<Mutex<Sink>>, defpath:String){
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
                                            handler.queue.remove(0);
                                        },
                                        Err(_)=>{
                                            handler.cur_song = None;
                                            println!("Error playing song :/")
                                        }
                                    };
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
                                            println!("Error playing song :/")
                                        }, 
                                        _ => (),
                                    };
                                }
                                else if !handler.queue.is_empty(){
                                    let nextsong = handler.queue.get(0).unwrap().to_owned();
                                    match play_song(&nextsong, &sink){
                                        Ok(_) =>{
                                            handler.cur_song = Some(nextsong);
                                            handler.queue.remove(0);
                                        },
                                        Err(_) =>{
                                            handler.cur_song = None;
                                            println!("Error playing song :/")
                                        }
                                    }
                                    
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
                                            handler.queue.remove(0);
                                        },
                                        Err(_)=>{
                                            handler.cur_song = None;
                                            println!("Error playing song :/")
                                        }
                                    };    
                                }
                            },
                        }
                        if handler.cur_song.as_ref().is_some() &&  handler.islooping != operations::Loop::LoopSong{
                            print!("\x1B[1A");
                            print!("\x1B[2K");
                            println!("Now Playing {}", handler.cur_song.as_ref().unwrap().clone());
                            print!("\x1B[1E");
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
                                        "view" => println!("Playing at {}", sink.volume()),
                                        "set" => {
                                            if options.get(1).is_some(){
                                                let val: Result<f32,_> = options.get(1).unwrap().parse();
                                                match val{
                                                    Err(_) => println!("Error: volume should be set to between 0 and 1"),
                                                    Ok(volume) => {
                                                        if 0.0 <= volume && volume <= 1.0{
                                                            sink.set_volume(volume)
                                                        }
                                                        else{
                                                            println!("Error: volume should be set to between 0 and 1")
                                                        }
                                                    },
                                                }
                                            }else{
                                                println!("Error: volume should be set to between 0 and 1");
                                            }
                                            println!("Volume is at {}", sink.volume());
                                        },
                                        "down" | "Down" => {
                                            let cur_volume = sink.volume();
                                            if cur_volume > 0.05{
                                                sink.set_volume(cur_volume - 0.05);
                                            }
                                            else{
                                                sink.set_volume(0.0);
                                            }
                                            println!("Volume is at {}", sink.volume());
                                        }
                                        "up" | "Up" => {
                                            let cur_volume = sink.volume();
                                            if cur_volume < 0.95{
                                                sink.set_volume(cur_volume + 0.05);
                                            }
                                            else{
                                                sink.set_volume(1.0);
                                            }
                                            println!("Volume is at {}", sink.volume());
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
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), ReadlineError>{
    dotenv().ok();
    let path = std::env::var("MUSICPATH").expect("MUSICPATH must be set.");
    let pptath = path.clone();
    let (tx, rx) : (Sender<AudioCommand>, Receiver<AudioCommand>) = mpsc::channel(32);
    let player = Arc::new(RustyHeadphones::new(tx.clone())); 
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let stream_handle = Arc::new(Mutex::new(stream_handle));
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle.lock().unwrap()).unwrap()));
    let audio_handle = tokio::spawn({
        let stream_handle = stream_handle.clone();
        let sink = sink.clone();
        async move {
            player_thread(rx, stream_handle, sink, pptath).await;
        }
    });
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .build();
    
    let h = helpers::HeadphoneHelper::new(path);
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(h));
    let _ =  rl.load_history("history.txt");
    let mut last_command_handle: Option<tokio::task::JoinHandle<()>> = None;
    loop {
        if let Some(handle) = last_command_handle.take() {
            if let Err(e) = handle.await {
                println!("Previous command failed: {}", e);
            }
        }
        let p = format!(">>");
        let readline = rl.readline(&p);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let command = line.trim().to_string();
                let tx = tx.clone();
                let s:Vec<&str>  = command.split(" ").collect();
                let command = match s.clone().get(0) {
                    Some(&"exit") =>{
                        Some(AudioCommand::Exit)
                    },
                    Some(&"stop")=>
                        Some(AudioCommand::Stop)
                    ,
                    Some(&"pause")=>{
                        Some(AudioCommand::Pause)
                    },
                    Some(&"play")=>{
                        Some(AudioCommand::Play(s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e.to_owned()).collect()))
                    }
                    Some(&"shuffle")=>{
                        Some(AudioCommand::Shuffle)
                    },
                    Some(&"skip") =>{
                        Some(AudioCommand::Skip)
                    },
                    Some(&"queue") =>{
                        Some(AudioCommand::Queue(s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e.to_owned()).collect()))
                    },
                    Some(&"volume")=>{
                        Some(AudioCommand::VolumeChanger(s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e.to_owned()).collect()))
                    }
                    Some(&"loop")=>{
                        Some(AudioCommand::SetLoop(s.get(1).map(|&x| x.to_string())))
                    },
                    Some(&"restart")=>{
                        Some(AudioCommand::Restart)
                    }
                    Some(&"back")=>{
                        Some(AudioCommand::Back)
                    },
                    Some(&"mute")=>{
                        Some(AudioCommand::Mute)
                    },
                    Some(&"unmute")=>{
                        Some(AudioCommand::Unmute)
                    },
                    _ => {
                        println!("Testing: Cannot do {} right now", &s.join(" "));
                        None
                    },
                };
                if let Some(cmd) = command{
                    let tx = tx.clone();
                    let cmd_copy = cmd.clone();
                    last_command_handle = Some(tokio::spawn(async move {
                    if let Err(e) = tx.send(cmd_copy).await {
                        println!("Failed to send command: {}", e);
                    }}));
                    match cmd {
                        AudioCommand::Exit=> break,
                        _ => ()
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                player.send_command(AudioCommand::Exit).await;
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                player.send_command(AudioCommand::Exit).await;
                break;
            }
        }
    }
    if let Err(e) = audio_handle.await {
        println!("Audio thread error: {}", e);
    }
    println!("Goodbye! :)");
    rl.append_history("history.txt")
}
