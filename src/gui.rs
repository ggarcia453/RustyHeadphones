use rodio::OutputStreamHandle;
use rodio::{OutputStream, Sink};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Sender, Receiver};
use crate::backend::AudioCommand;
use crate::backend::player_thread;
use crate::operations::is_music_file;

struct RustyHeadphonesGUI{
    tx :Sender<AudioCommand>,
    rx : Receiver<Option<String>>,
    sink : Arc<std::sync::Mutex<Sink>>,
    _stream : OutputStream, 
    _stream_handle: Arc<Mutex<OutputStreamHandle>>,
    filename: String,
    playing : String,
    shuffle: bool,
    loophandle: usize,
    isplaying:bool,
}

impl RustyHeadphonesGUI{
    fn new(_ctx : &eframe::CreationContext<'_>, ppath: String) -> Self{
        let (tx, rx) : (Sender<AudioCommand>, Receiver<AudioCommand>) = mpsc::channel(32);
        let (txx, rxx):(Sender<Option<String>>, Receiver<Option<String>>) = mpsc::channel(32);
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let stream_handle = Arc::new(Mutex::new(stream_handle));
        let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle.lock().unwrap()).unwrap()));
        tokio::spawn({
            let stream_handle = stream_handle.clone();
            let sink = sink.clone();
            async move {
                player_thread(rx, txx, stream_handle, sink, ppath).await;
            }
        });
        Self{
            tx,
            rx: rxx,
            sink,
            _stream,
            _stream_handle :stream_handle,
            filename : "".to_string(),
            playing: String::from("Now Playing ___"),
            shuffle : false,
            loophandle: 0,
            isplaying:true
        }
    }

    fn send(&self, cmd: AudioCommand){
        match self.tx.try_send(cmd) {
            Err(e) => println!("{:?}", e),
            Ok(_) => (),
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
                    if self.shuffle && !is_music_file(&self.filename){
                        let f = "shuffle ".to_owned() + &self.filename;
                        self.send(AudioCommand::Queue(f.split_whitespace().map(str::to_string).collect())) ;
                    }
                    else{
                        self.send(AudioCommand::Queue(self.filename.clone().split_whitespace().map(str::to_string).collect())) ;
                    }
                    self.filename = "".to_string();
                }
                ui.checkbox(& mut self.shuffle, "Shuffle?");
            });
            ui.horizontal(|ui|{
                if ui.button("Queue").clicked(){
                    if self.shuffle && !is_music_file(&self.filename){
                        let f = "shuffle ".to_owned() + &self.filename;
                        self.send(AudioCommand::Queue(f.split_whitespace().map(str::to_string).collect())) ;
                    }
                    else{
                        self.send(AudioCommand::Queue(self.filename.clone().split_whitespace().map(str::to_string).collect())) ;
                    }
                    self.filename = "".to_string();
                }
                if ui.button("Play").clicked(){
                    self.send(AudioCommand::Play(self.filename.clone().split_whitespace().map(str::to_string).collect())) ;
                    self.filename = "".to_string();
                }
            });
            ui.add(egui::Label::new(&self.playing));
            match self.rx.try_recv() {
                Ok(s) => {
                    if s.is_some(){
                        let k = s.unwrap();
                        if (&k) == (&String::from("  ")){
                            self.playing = String::from("Now Playing ___")
                        }
                        else if (&k).starts_with("Now Playing ") && !(&k).contains("\n"){
                            self.playing = k;
                        }
                    }
                },
                _ => (),
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui |{
                let back_image = egui::include_image!("../assets/back.png");
                if ui.add_sized([100.0, 100.0],egui::ImageButton::new(back_image)).clicked(){
                    if let Ok(sink) = self.sink.lock(){
                        if  sink.speed() * sink.get_pos().as_secs_f32() > 2.0 {
                            self.send(AudioCommand::Restart);
                        }
                        else{
                            self.send(AudioCommand::Back);
                        }
                    }
                }
                let play_image = egui::include_image!("../assets/play.png");
                let pause_image = egui::include_image!("../assets/pause.png");
                if self.playing.ends_with("___"){
                    self.isplaying = true;
                }
                let p_image = match self.isplaying {
                    true => pause_image,
                    false => play_image,
                };
                if ui.add_sized([100.0, 100.0], egui::ImageButton::new(p_image)).clicked(){
                    match self.isplaying {
                        true => {
                            if !self.playing.ends_with("___"){
                                self.send(AudioCommand::Pause);
                                self.isplaying = !self.isplaying
                            }
                        },
                        false => {
                            self.send(AudioCommand::Play(Vec::new()));
                            self.isplaying = !self.isplaying;
                        },
                    }
                    
                }
                let forward_image = egui::include_image!("../assets/forward.png");
                if ui.add_sized([100.0, 100.0], egui::ImageButton::new(forward_image)).clicked(){
                    self.send(AudioCommand::Skip);
                }
                let noloop_image = egui::include_image!("../assets/noloop.png");
                let loopqueue_image = egui::include_image!("../assets/loopqueue.png");
                let loopsong_image = egui::include_image!("../assets/loopsong.png");
                let current_image = match self.loophandle {
                    0 => noloop_image, 
                    1 => loopqueue_image,
                    _ => loopsong_image
                };
                if ui.add_sized([100.0, 100.0], egui::ImageButton::new(current_image)).clicked(){
                    match self.loophandle {
                        0 => self.send(AudioCommand::SetLoop(Some(String::from("queue")))),
                        1 => self.send(AudioCommand::SetLoop(Some(String::from("song")))),
                        2 => self.send(AudioCommand::SetLoop(Some(String::from("cancel")))),
                        _ => (),
                    }
                    self.loophandle = (self.loophandle + 1)%3;
                }
            });
        });
    }
}

pub async fn gui_main(defpath:String, _token:String) -> Result<(), eframe::Error>{
    let path :String = defpath;
    let ppath = path.clone();
    let options = eframe::NativeOptions::default();
    eframe::run_native("RustyHeadphonesGUI", options, 
    Box::new(|cc|{
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Box::new(RustyHeadphonesGUI::new(cc, ppath))
    })
    )
}