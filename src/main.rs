use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
// use anyhow;
use std::{fs::{self, File}, io::BufReader, path::PathBuf, time::Duration};
use iced::{self,Element, widget::{column,scrollable,row,button,svg,text},};

fn main()->Result<(), iced::Error>{
    iced::application(boot,update, view).run()
}

fn boot()->State{
    let handle=DeviceSinkBuilder::open_default_sink().expect("Failed to connect to audio");
    let player=rodio::Player::connect_new(&handle.mixer());
    State {handle:handle, player: player, songs:Vec::new()}
}

#[derive(Debug, Clone)]
enum Message{
    TogglePlay,
    Start(String)
}

// #[derive(Default)]
struct State{
    handle:MixerDeviceSink,
    player:Player,
    songs:Vec<String>
}


fn view(state:&State) -> Element<'_, Message>{
    column![
        scrollable(row(
            state.songs.iter().map(|file|{
                button(text(file))
                .on_press(Message::Start(file.clone()))
                .into()
            })
        )),
        button(svg("play.svg"))
            .on_press(Message::TogglePlay),
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

fn scan_songs(folder:PathBuf)->Vec<PathBuf>{
    const VALID_EXTENTIONS:[&str;2]=["mp3","m4a"];
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
                                out.push(path);
                            }
                        }else if file_type.is_dir(){
                            out.extend(scan_songs(path));
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