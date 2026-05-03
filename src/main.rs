use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use rust_embed::Embed;
// use anyhow;
use std::{collections::HashMap, fs::{self, File}, path::{Path, PathBuf}, rc::Rc, sync::Mutex};
use iced::{self, Element, Length, widget::{svg, Space, button, column, row, scrollable, svg::Handle, text}};

fn main()->Result<(), iced::Error>{
    iced::application(boot,update, view).run()
}

fn boot()->State{
    let handle=DeviceSinkBuilder::open_default_sink().expect("Failed to connect to audio");
    let player=rodio::Player::connect_new(&handle.mixer());
    let mut entries=HashMap::new();
    State {_handle:handle, player: player, top_folder:scan_songs("/home/agiller/Music",&mut entries), entries:entries}
}

#[derive(Embed)]
#[folder = "icons/"]
struct Icon;
impl Icon{
    fn get_handle(name:&str)->Option<Handle>{
        Self::get(name).map(|file|{Handle::from_memory(file.data)})
    }
}

#[derive(Debug, Clone)]
enum Message{
    TogglePlay,
    Start(PathBuf),
    OpenFolder(PathBuf),
    CloseFolder(PathBuf)
}

struct State{
    _handle:MixerDeviceSink,
    player:Player,
    top_folder:Vec<Rc<Mutex<MenuEntry>>>,
    entries:HashMap<PathBuf,Rc<Mutex<MenuEntry>>>
}

struct MenuEntry{
    path:PathBuf,
    status:ButtonStatus,
}

enum ButtonStatus{
    // ClosedFolder,
    // CachedFolder(Vec<Rc<Mutex<MenuEntry>>>),
    Folder(bool,Vec<Rc<Mutex<MenuEntry>>>),
    File
}

// fn generate_folder_button<'a>(entry:&MenuEntry,name:&'a str)->Element<'a, Message>{
    
// }

fn get_folder_entries<'a>(entries:&Vec<Rc<Mutex<MenuEntry>>>)->Element<'a, Message>{
    column(
        entries.iter().map(|entry|{
            let entry=entry.lock().expect("error accesing");
            let name= String::from(entry.path.file_stem().expect("empty file name").to_str().expect("invalid filename"));
            match &entry.status{
                ButtonStatus::File=>{
                    button(
                        row![
                            svg(Icon::get_handle("song.svg").expect("couldn't find icon")).width(16),
                            text(name)
                        ]
                    )
                    .on_press(Message::Start(entry.path.clone())).into()
                },
                ButtonStatus::Folder(false,_)=>{
                    button(
                        row![
                            svg(Icon::get_handle("folder.svg").expect("couldn't find icon")).width(16),
                            text(name)
                        ]
                    )
                    .on_press(Message::OpenFolder(entry.path.clone())).into()
                },
                ButtonStatus::Folder(true,sub_entries)=>{
                    column![
                        button(
                            row![
                                svg(Icon::get_handle("folder.svg").expect("couldn't find icon")).width(16),
                                text(name)
                            ]
                        )
                        .on_press(Message::CloseFolder(entry.path.clone())),
                        row![
                            Space::new().width(32),
                            get_folder_entries(sub_entries)
                        ]
                    ].spacing(4).into()
                }
            }
        })
    ).spacing(4).width(Length::Fill).into()
}

fn view(state:&State) -> Element<'_, Message>{
    // let mut song_entries=Vec::new();
    println!("updating layout");
    column![
        scrollable(get_folder_entries(&state.top_folder)).height(Length::Fill),
        button(svg(Icon::get_handle("play.svg").expect("couldn't file play icon")))
            .on_press(Message::TogglePlay)
            .height(64),
    ].into()
}

fn update(state:&mut State,msg:Message){
    match msg{
        Message::TogglePlay=>{
            if state.player.is_paused(){
                state.player.play();
            }else{
                state.player.pause();
            }
        }
        Message::Start(song)=>{
            match File::open(song){
                Ok(file)=>{
                    let source= Decoder::try_from(file).unwrap();
                    state.player.append(source);
                }
                Err(err)=>{
                    println!("{err}")
                }
            }
        }
        Message::OpenFolder(path)=>{
            let mut scan_files=true;
            if let Ok(mut entry)=state.entries[&path].lock()
            && let ButtonStatus::Folder(ref mut open,_) =entry.status{
                    scan_files=!*open;
                    *open=true;
            }
            if scan_files{
                let new_entries =scan_songs(&path,&mut state.entries);
                if let Ok(mut entry)=state.entries[&path].lock()
                && let ButtonStatus::Folder(_,ref mut entries) =entry.status{
                    *entries =new_entries;
            }
            }
        }
        Message::CloseFolder(path)=>{
            if let Ok(mut entry)=state.entries[&path].lock()
            && let ButtonStatus::Folder(ref mut opened, _)=entry.status{
                *opened=false;
            }
        }
    }
}

fn scan_songs<P:AsRef<Path>>(folder:P, entries:&mut HashMap<PathBuf,Rc<Mutex<MenuEntry>>>)->Vec<Rc<Mutex<MenuEntry>>>{
    const VALID_EXTENTIONS:[&str;2]=["mp3","m4a"];
    println!("scanning songs");
    match fs::read_dir(folder){
        Ok(dir_entries)=>{
            let mut out=Vec::new();
            for entry in dir_entries{
                if let Ok(entry)=entry{
                    let path=entry.path();
                    if let Ok(file_type)=entry.file_type(){
                        if file_type.is_file(){
                            if let Some(Some(extention))=path.extension().map(|extention|{extention.to_str()})
                            && VALID_EXTENTIONS.contains(&extention){
                                let entry=Rc::new(Mutex::new(MenuEntry{path:path.clone(),status:ButtonStatus::File}));
                                out.push(Rc::clone(&entry));
                                entries.insert(path,entry);

                            }
                        }else if file_type.is_dir(){
                            let entry=Rc::new(Mutex::new(MenuEntry{path:path.clone(),status:ButtonStatus::Folder(false,Vec::new())}));
                            out.push(Rc::clone(&entry));
                            entries.insert(path,entry);
                        }
                    }
                }
            }
            out
        },
        Err(err)=>{
            println!("{err}");
            Vec::new()
        }
    }
}