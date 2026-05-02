use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use rust_embed::Embed;
// use anyhow;
use std::{fs::{self, File}, path::{Path, PathBuf}, time::Duration};
use iced::{self, Element, Length, widget::{button, column, row, scrollable, svg::{Handle}, svg, text}};

fn main()->Result<(), iced::Error>{
    iced::application(boot,update, view).run()
}

fn boot()->State{
    let handle=DeviceSinkBuilder::open_default_sink().expect("Failed to connect to audio");
    let player=rodio::Player::connect_new(&handle.mixer());
    State {_handle:handle, player: player, songs:scan_songs("/home/agiller/Music")}
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
    Start(PathBuf)
}

struct State{
    _handle:MixerDeviceSink,
    player:Player,
    songs:Vec<MenuEntry>
}

struct MenuEntry{
    path:PathBuf,
    folder:bool
}

fn view(state:&State) -> Element<'_, Message>{
    println!("updating layout");
    column![
        scrollable(column(
            state.songs.iter().filter_map(|entry|{
                entry.path.file_stem().expect("empty file name").to_str().map(|name|{
                    button(
                        row![
                            svg(Icon::get_handle(if entry.folder {"folder.svg"} else {"song.svg"}).expect("couldn't find icon")).width(16),
                            text(name)
                        ]
                    )
                    .on_press(Message::Start(entry.path.clone()))
                    .into()
                })
            })
        ).spacing(4).width(Length::Fill)).height(Length::Fill),
        button(svg(Icon::get_handle("play.svg").expect("couldn't file play icon")))
            .on_press(Message::TogglePlay)
            .height(64),
    ].into()
}

fn update(state:&mut State,msg:Message){
    match msg{
        Message::TogglePlay=>{
            if state.player.is_paused(){
                state.player.pause();
            }else{
                state.player.play();
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
    }
}

fn scan_songs<P:AsRef<Path>>(folder:P)->Vec<MenuEntry>{
    const VALID_EXTENTIONS:[&str;2]=["mp3","m4a"];
    println!("scanning songs");
    match fs::read_dir(folder){
        Ok(entries)=>{
            let mut out=Vec::new();
            for entry in entries{
                if let Ok(entry)=entry{
                    let path=entry.path();
                    if let Ok(file_type)=entry.file_type(){

                        if file_type.is_file(){
                            if let Some(Some(extention))=path.extension().map(|extention|{extention.to_str()})
                            && VALID_EXTENTIONS.contains(&extention){
                                out.push(MenuEntry{path:path,folder:false});

                            }
                        }else if file_type.is_dir(){
                            out.push(MenuEntry{path:path,folder:true});
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