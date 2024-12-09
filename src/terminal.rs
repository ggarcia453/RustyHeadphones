use rodio::{OutputStream, Sink};
use rustyline::{CompletionType, Config, Editor};
use rustyline::error::ReadlineError;
use tokio::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex};
use crate::backend::AudioCommand;
use crate::backend::player_thread;
use crate::helpers;
use std::io::{stdout, Write};
use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    QueueableCommand,
    ExecutableCommand,
};

pub async fn terminal_main(defpath:String) -> Result<(), ReadlineError>{
    let path :String = defpath;
    let pptath = path.clone();
    let (tx, rx) : (Sender<AudioCommand>, Receiver<AudioCommand>) = mpsc::channel(32);
    let (txx,mut rxx):(Sender<Option<String>>, Receiver<Option<String>>) = mpsc::channel(32);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let stream_handle = Arc::new(Mutex::new(stream_handle));
    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle.lock().unwrap()).unwrap()));
    let audio_handle = tokio::spawn({
        let stream_handle = stream_handle.clone();
        let sink = sink.clone();
        async move {
            player_thread(rx, txx, stream_handle, sink, pptath).await;
        }
    });
    let printer_task = tokio::spawn(async move {
        while let Some(response) = rxx.recv().await {
            match response {
                Some(s) => {
                    if !s.is_empty() && s != (String::from("  ")){
                        for i in s.split("\n"){
                            println!();
                            stdout().execute(cursor::SavePosition).unwrap();
                            stdout()
                                .queue(cursor::MoveUp(1)).unwrap()
                                .queue(Clear(ClearType::CurrentLine)).unwrap();
                            println!("{}", i);
                            stdout().execute(cursor::RestorePosition).unwrap();
                        }
                        print!(">>");
                        stdout().flush().unwrap();
                    }
                },
                None=> break,
            }
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
        let p = String::from(">>");
        let readline = rl.readline(&p);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let command = line.trim().to_string();
                let tx = tx.clone();
                let cmd = match command.split_once(' ').unwrap_or((&command, "")) {
                    ("exit", _) => Some(AudioCommand::Exit),
                    ("stop", _) => Some(AudioCommand::Stop),
                    ("pause", _) => Some(AudioCommand::Pause),
                    ("play", arg) => Some(AudioCommand::Play(if arg.is_empty() { None } else { Some(arg.to_string()) })),
                    ("shuffle", _) => Some(AudioCommand::Shuffle),
                    ("skip", _) => Some(AudioCommand::Skip),
                    ("queue", arg) => Some(AudioCommand::Queue(arg.to_owned())),
                    ("volume", arg) => Some(AudioCommand::VolumeChanger(arg.split_whitespace().map(|e| e.to_owned()).collect())),
                    ("loop", arg) => Some(AudioCommand::SetLoop(if arg.is_empty() { None } else { Some(arg.to_string()) })),
                    ("restart", _) => Some(AudioCommand::Restart),
                    ("back", _) => Some(AudioCommand::Back),
                    ("mute", _) => Some(AudioCommand::Mute),
                    ("unmute", _) => Some(AudioCommand::Unmute),
                    ("help", _) => Some(AudioCommand::Help),
                    ("speed", arg) => Some(AudioCommand::SetSpeed(arg.split_whitespace().map(|e| e.to_owned()).collect()))
                    ,
                    _ => None,
                };
                if let Some(cmd) = cmd{
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
    if let Err(e) = printer_task.await{
        println!("Printer thread error: {}", e);
    }
    println!("Goodbye! :)");
    rl.append_history("history.txt")
}