use rodio::OutputStreamHandle;
use rodio::{OutputStream, Sink};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Sender, Receiver};
use dotenv::dotenv;
use crate::backend::AudioCommand;
use crate::backend::player_thread;

struct RustyHeadphonesGUI{
    tx :Sender<AudioCommand>,
    sink : Arc<std::sync::Mutex<Sink>>,
    _stream : OutputStream, 
    _stream_handle: Arc<Mutex<OutputStreamHandle>>,
    filename: String
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
            sink,
            _stream,
            _stream_handle :stream_handle,
            filename : "".to_string()
        }
    }

}

impl eframe::App for RustyHeadphonesGUI{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let la = ui.label("File to queue:");
                let response = ui.add(egui::TextEdit::singleline(&mut self.filename)).labelled_by(la.id);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    match self.tx.try_send(AudioCommand::Queue(self.filename.clone().split_whitespace().map(str::to_string).collect())) {
                        Err(e) => println!("{:?}", e),
                        Ok(_) => (),
                    }
                    self.filename = "".to_string();
                }
            });
            if ui.button("Queue").clicked(){
                match self.tx.try_send(AudioCommand::Queue(self.filename.clone().split_whitespace().map(str::to_string).collect())) {
                    Err(e) => println!("{:?}", e),
                    Ok(_) => (),
                }
                self.filename = "".to_string();
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui |{
                ui.add_space(200.0);
                let back_image = egui::include_image!("../assets/back.png");
                if ui.add_sized([100.0, 100.0],egui::ImageButton::new(back_image)).clicked(){
                    if let Ok(sink) = self.sink.lock(){
                        if  sink.speed() * sink.get_pos().as_secs_f32() > 2.0 {
                            match self.tx.try_send(AudioCommand::Restart){
                                Err(e) => println!("{:?}", e),
                                Ok(_) => (),
                            }
                        }
                        else{
                            match self.tx.try_send(AudioCommand::Back){
                                Err(e) => println!("{:?}", e),
                                Ok(_) => (),
                            }
                        }
                    }
                }
                let play_image = egui::include_image!("../assets/play.png");
                if ui.add_sized([100.0, 100.0], egui::ImageButton::new(play_image)).clicked(){
                    match self.tx.try_send(AudioCommand::Play(Vec::new())){
                        Err(e) => println!("{:?}", e),
                        Ok(_) => (),
                    }
                }
                let pause_image = egui::include_image!("../assets/pause.png");
                if ui.add_sized([100.0, 100.0], egui::ImageButton::new(pause_image)).clicked(){
                    match self.tx.try_send(AudioCommand::Pause){
                        Err(e) => println!("{:?}", e),
                        Ok(_) => (),
                    }
                }
                let forward_image = egui::include_image!("../assets/forward.png");
                if ui.add_sized([100.0, 100.0], egui::ImageButton::new(forward_image)).clicked(){
                    match self.tx.try_send(AudioCommand::Skip){
                        Err(e) => println!("{:?}", e),
                        Ok(_) => (),
                    }
                }
            });
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
    eframe::run_native("RustyHeadphonesGUI", options, 
    Box::new(|cc|{
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Box::new(RustyHeadphonesGUI::new(cc, ppath))
    })
    )
}