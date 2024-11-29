use std::env;
use std::error::Error;
use std::fmt;
use clap::Parser;
mod operations;
mod helpers;
mod backend;
mod terminal;
mod gui;

#[derive(Parser)]
struct ArgumentTracker{
    #[arg(short, long, default_value_t=env::current_dir().expect("Could Not Grab Directory to use").to_string_lossy().into_owned())]
    defpath: String,
    #[arg(short, long, default_value_t=String::new())]
    token: String,
    #[arg(short, long, default_value_t=String::new())]
    mode: String
}

#[derive(Debug)]
struct ModeError {
    message: String,
}
impl fmt::Display for ModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ModeError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = ArgumentTracker::parse();
    if args.mode.is_empty()  || args.mode == "terminal"{
        terminal::terminal_main(args.defpath, args.token).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
    else if args.mode == "gui"{
        gui::gui_main(args.defpath,args.token).await.map_err(|e |Box::new(e) as Box<dyn std::error::Error>)
    }
    else{
        Err(Box::new(ModeError {
            message: "Invalid mode specified".to_string(),
        }) as Box<dyn std::error::Error>) 
    }
}
