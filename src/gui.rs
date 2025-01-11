use rodio::OutputStreamHandle;
use rodio::{OutputStream, Sink};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Sender, Receiver};
use crate::backend::AudioCommand;
use crate::backend::player_thread;
use rfd::FileDialog;

struct RustyHeadphonesGUI{
    tx :Sender<AudioCommand>,
    rx : Receiver<Option<String>>,
    sink : Arc<std::sync::Mutex<Sink>>,
    _stream : OutputStream, 
    _stream_handle: Arc<Mutex<OutputStreamHandle>>,
    defpath: String, 
    pathqueue: Option<String>,
    file_or_folder:bool,
    feedback : Option<String>,
    shuffle: bool,
    loophandle: usize,
    isplaying:bool,
    volume: f32,
    speed: f32,
    cur_queue: Vec<String>
}

impl RustyHeadphonesGUI{
    fn new(_ctx : &eframe::CreationContext<'_>, ppath: String) -> Self{
        let (tx, rx) : (Sender<AudioCommand>, Receiver<AudioCommand>) = mpsc::channel(32);
        let (txx, rxx):(Sender<Option<String>>, Receiver<Option<String>>) = mpsc::channel(32);
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let stream_handle = Arc::new(Mutex::new(stream_handle));
        let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle.lock().unwrap()).unwrap()));
        let defpath = ppath.clone();
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
            defpath,
            pathqueue : None,
            file_or_folder: false,
            feedback: None,
            shuffle : false,
            loophandle: 0,
            isplaying:true,
            volume: 100.0,
            speed : 1.0,
            cur_queue: Vec::new(),
        }
    }

    fn send(&self, cmd: AudioCommand){
        if let Err(e) = self.tx.try_send(cmd) { 
            println!("{:?}", e) 
        }
    }

}

impl eframe::App for RustyHeadphonesGUI{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Currently Selected:");
                if self.pathqueue.is_some(){
                    ui.label(self.pathqueue.as_ref().unwrap());
                }
                else{
                    ui.label("None");
                }
                if ui.button("Grab File").clicked(){
                    if let Some(path) = FileDialog::new().set_directory(self.defpath.clone()).pick_file() {
                        self.pathqueue = Some(path.display().to_string());
                        self.file_or_folder = false;
                    }
                }
                if ui.button("Grab Folder").clicked(){
                    if let Some(path) = FileDialog::new().set_directory(self.defpath.clone()).pick_folder() {
                        self.pathqueue = Some(path.display().to_string());
                        self.file_or_folder = true;
                    }
                }
                ui.checkbox(& mut self.shuffle, "Shuffle Folder?");
            });
            ui.horizontal(|ui|{
                if ui.button("Queue").clicked(){
                    if self.pathqueue.is_some(){
                        if self.shuffle && self.file_or_folder{
                            let f = "shuffle ".to_owned() + self.pathqueue.as_ref().unwrap();
                            self.send(AudioCommand::Queue(f));
                        }
                        else{
                            self.send(AudioCommand::Queue(self.pathqueue.as_ref().unwrap().to_owned()));
                        }
                    }
                    self.pathqueue = None;
                }
                if ui.button("Play").clicked(){
                    if self.pathqueue.is_some(){
                        self.send(AudioCommand::Play(Some(self.pathqueue.as_ref().unwrap().to_owned())));
                    }
                    self.pathqueue = None;
                }
                if ui.button("Shuffle Queue").clicked(){
                    self.send(AudioCommand::Shuffle);
                    self.send(AudioCommand::Queue("view".to_string()));
                }
            });
            if self.feedback.is_some(){
                ui.label(self.feedback.as_ref().unwrap());
            }
            if !self.cur_queue.is_empty(){
                ui.label(self.cur_queue.join("\n"));
            }
            if let Ok(s) =  self.rx.try_recv() {
                if s.is_some(){
                        let k = s.unwrap();
                        if k == *"  "{
                            self.feedback = None;
                        }
                        else if k.starts_with("Now Playing "){
                            self.send(AudioCommand::Queue("view".to_string()));
                            self.isplaying = true;
                            self.feedback = Some(k);
                            self.send(AudioCommand::Play(None));
                        }
                        else if k.starts_with("Currently Playing"){
                            self.cur_queue = k.split("\n").map(|x |x.to_string()).collect();
                            self.cur_queue.remove(0);
                        }
                    }
                                
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
                let p_image = match self.isplaying {
                    true => pause_image,
                    false => play_image,
                };
                if ui.add_sized([100.0, 100.0], egui::ImageButton::new(p_image)).clicked(){
                    match self.isplaying {
                        true => {
                            self.send(AudioCommand::Pause);
                        },
                        false => {
                            self.send(AudioCommand::Play(None));
                        },
                    }
                    self.isplaying = !self.isplaying;
                    
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
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui|{
                    ui.horizontal(|ui|{
                        ui.label("Volume:");
                        if ui.add(egui::Slider::new(&mut self.volume, 0.0 ..= 100.0)).changed(){
                            self.send(AudioCommand::VolumeChanger(vec!["set".to_string(), self.volume.to_string()]));
                    }
                    });
                    ui.horizontal(|ui|{
                        ui.label("Speed:");
                        if ui.add(egui::Slider::new(&mut self.speed, 0.01 ..= 10.0)).changed(){
                            self.send(AudioCommand::SetSpeed(vec!["set".to_string(), self.speed.to_string()]));
                        }
                    });
                });
            });
        });
    }
}

pub async fn gui_main(defpath:String) -> Result<(), eframe::Error>{
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