use rodio::{OutputStream, Sink};
use dotenv::dotenv;
use rustyline::{CompletionType, Config, Editor};
use rustyline::error::ReadlineError;
use tokio::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use crate::backend::AudioCommand;
use crate::backend::player_thread;
use crate::helpers;

pub async fn terminal_main(defpath:String, token:String) -> Result<(), ReadlineError>{
    dotenv().ok();
    let path :String; 
    if let Ok(result) = std::env::var("MUSICPATH") {
        path = result;
    }
    else{
        path = defpath;
    }
    let pptath = path.clone();
    let (tx, rx) : (Sender<AudioCommand>, Receiver<AudioCommand>) = mpsc::channel(32);
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
    println!("Welcome to RustyHeadphones! For help on how to use it use the help command or reference the README :)");
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .build();
    
    let h = helpers::HeadphoneHelper::new(path);
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(h));
    let _ =  rl.load_history("history.txt");
    loop {
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
                    Some(&"help")=>{
                        Some(AudioCommand::Help)
                    },
                    Some(&"spotify")=>{
                        if token.is_empty(){
                            println!("YOu need to pass in a token");
                            None
                        }
                        else{
                            println!("SPotify Setup");
                            None
                        }
                    },
                    Some(&"speed")=>{
                        Some(AudioCommand::SetSpeed(s.into_iter().enumerate().filter(|&(i, _)| i >0 ).map(|(_, e)| e.to_owned()).collect()))
                    }
                    _ => {
                        println!("Testing: Cannot do {} right now", &s.join(" "));
                        None
                    },
                };
                if let Some(cmd) = command{
                    let tx = tx.clone();
                    let cmd_copy = cmd.clone();
                    if let Err(e) = tx.try_send(cmd_copy){
                        println!("Failed to send command: {}", e);
                    }
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
                if let Err(e) = tx.send(AudioCommand::Exit).await {
                    println!("Failed to send command: {}", e);
                }
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                if let Err(e) = tx.send(AudioCommand::Exit).await {
                    println!("Failed to send command: {}", e);
                }
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