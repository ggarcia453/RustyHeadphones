use rodio::OutputStreamHandle;
use rodio::{OutputStream, Sink};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Sender, Receiver};
use dotenv::dotenv;
use crate::backend::AudioCommand;
use crate::backend::player_thread;

struct RustyHeadphonesGUI{
    tx :Sender<AudioCommand>,
    _sink : Arc<std::sync::Mutex<Sink>>,
    _stream : OutputStream, 
    _stream_handle: Arc<Mutex<OutputStreamHandle>>
}

impl RustyHeadphonesGUI{
    fn new(_ctx : &eframe::CreationContext<'_>, ppath: String) -> Self{
        let (tx, rx) : (Sender<AudioCommand>, Receiver<AudioCommand>) = mpsc::channel(32);
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let stream_handle = Arc::new(Mutex::new(stream_handle));
        let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle.lock().unwrap()).unwrap()));
        tokio::spawn({
            let stream_handle = stream_handle.clone();
            let sink = sink.clone();
            async move {
                player_thread(rx, stream_handle, sink, ppath).await;
            }
        });
        Self{
            tx,
            _sink : sink,
            _stream,
            _stream_handle :stream_handle
        }
    }

}

impl eframe::App for RustyHeadphonesGUI{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Queue").clicked(){
                match self.tx.try_send(AudioCommand::Queue(vec!["Goodbye.mp3".to_string()])){
                    Err(e) => println!("{:?}", e),
                    Ok(_) => (),
                }
            }
        });
    }
}

pub async fn gui_main(defpath:String, _token:String) -> Result<(), eframe::Error>{
    dotenv().ok();
    let path :String; 
    if let Ok(result) = std::env::var("MUSICPATH") {
        path = result;
    }
    else{
        path = defpath;
    }
    let ppath = path.clone();
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "RustyHeadphones GUI",
        options,
        Box::new(|cc| Box::new(RustyHeadphonesGUI::new(cc, ppath)))
    )
}