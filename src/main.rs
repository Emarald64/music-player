use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use rust_embed::Embed;
// use anyhow;
use std::{collections::HashMap, fs::{self, DirEntry, File}, path::{Path, PathBuf}, rc::Rc, sync::Mutex};
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
    CloseFolder(PathBuf),
    Skip
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

fn get_folder_entries<'a>(entries:&Vec<Rc<Mutex<MenuEntry>>>)->Element<'a, Message>{
    column(
        entries.iter().map(|entry|{
            let entry=entry.lock().expect("error accesing");
            let name= String::from(entry.path.file_stem().expect("empty file name").to_str().expect("invalid filename"));
            const ICON_SIZE:u32=20;
            match &entry.status{
                ButtonStatus::File=>{
                    button(
                        row![
                            svg(Icon::get_handle("song.svg").expect("couldn't find icon")).width(ICON_SIZE),
                            text(name)
                        ]
                    )
                    .on_press(Message::Start(entry.path.clone())).into()
                },
                ButtonStatus::Folder(false,_)=>{
                    button(
                        row![
                            svg(Icon::get_handle("folder.svg").expect("couldn't find icon")).width(ICON_SIZE),
                            text(name)
                        ]
                    )
                    .on_press(Message::OpenFolder(entry.path.clone())).into()
                },
                ButtonStatus::Folder(true,sub_entries)=>{
                    column![
                        button(
                            row![
                                svg(Icon::get_handle("folder.svg").expect("couldn't find icon")).width(ICON_SIZE),
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
    ).spacing(4).into()
}

fn view(state:&State) -> Element<'_, Message>{
    // let mut song_entries=Vec::new();
    println!("updating layout");
    column![
        row![
            scrollable(
                get_folder_entries(&state.top_folder)
            ).height(Length::Fill),

        ],
        row![
            button(svg(Icon::get_handle(if state.player.is_paused() || state.player.empty() {"play.svg"} else {"pause.svg"}).expect("couldn't file play icon")))
                .on_press(Message::TogglePlay)
                .height(64),
            button(svg(Icon::get_handle("skip.svg").expect("counldn't find skip icon")))
                .on_press_maybe(
                    match state.player.len(){
                        2.. =>Some(Message::Skip),
                        _=>None
                    }
                )
        ],
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
        },
        Message::Skip=>{
            state.player.skip_one();
        },
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
        },
        Message::OpenFolder(path)=>{
            let mut scan_files=true;
            if let Ok(mut entry)=state.entries[&path].lock()
            && let ButtonStatus::Folder(ref mut open,ref entries) =entry.status{
                    scan_files=entries.is_empty();
                    *open=true;
            }
            if scan_files{
                let new_entries =scan_songs(&path,&mut state.entries);
                if let Ok(mut entry)=state.entries[&path].lock()
                && let ButtonStatus::Folder(_,ref mut entries) =entry.status{
                    *entries =new_entries;
            }
            }
        },
        Message::CloseFolder(path)=>{
            if let Ok(mut entry)=state.entries[&path].lock()
            && let ButtonStatus::Folder(ref mut opened, _)=entry.status{
                *opened=false;
            }
        },
    }
}

fn is_file(entry:&DirEntry)->bool{
    entry.file_type().map(|ft|{ft.is_file()}).unwrap_or(true)
}

fn scan_songs<P:AsRef<Path>>(folder:P, entries:&mut HashMap<PathBuf,Rc<Mutex<MenuEntry>>>)->Vec<Rc<Mutex<MenuEntry>>>{
    const VALID_EXTENTIONS:[&str;3]=["mp3","m4a","ogg"];
    println!("scanning songs");
    match fs::read_dir(folder){
        Ok(dir_entries)=>{
            let mut dir_entries:Vec<fs::DirEntry>=dir_entries.filter_map(|entry|{entry.ok()}).collect();
            dir_entries.sort_by(|e1,e2|{
                is_file(e1).cmp(&is_file(e2)).then_with(||{e1.file_name().cmp(&e2.file_name())})
            });
            dir_entries.iter().filter_map(|entry|{
                let file_type=entry.file_type().ok()?;
                let path=entry.path();
                if file_type.is_file()
                && let Some(Some(extention))=path.extension().map(|extention|{extention.to_str()})
                && VALID_EXTENTIONS.contains(&extention){
                    let entry=Rc::new(Mutex::new(MenuEntry{path:path.clone(),status:ButtonStatus::File}));
                    entries.insert(path,Rc::clone(&entry));
                    Some(entry)
                }else if file_type.is_dir(){
                    let entry=Rc::new(Mutex::new(MenuEntry{path:path.clone(),status:ButtonStatus::Folder(false,Vec::new())}));
                    entries.insert(path,Rc::clone(&entry));
                    Some(entry)
                }else{
                    None
                }
            }).collect()
        },
        Err(err)=>{
            println!("{err}");
            Vec::new()
        }
    }
}