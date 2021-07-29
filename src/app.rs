use std::{
    iter::Map,
    sync::mpsc::{Receiver, Sender},
};

use eframe::{egui::{self, Color32, FontDefinitions, Vec2}, epi};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
// if we add new fields, give them default values when deserializing old state
use std::collections::HashMap;
#[derive(Debug, Clone)]
struct SongMap {
    download_url: String,
    author: String,
    name: String,
    source_name:String,
    selected: bool,
}
#[derive(Debug, Clone)]
struct Song {
    name: String,
    artists: Vec<String>,
    maps: Vec<SongMap>,
    searched: bool,
}
pub struct MicroApp {
    label: String,
    songlist_id: HashMap<String, usize>,
    songlist: Vec<Song>,
    selected_song: String,
    selected_maps:HashMap<String,SongMap>,
    value: f32,
    channel: (std::sync::mpsc::Sender<SongMap>, Receiver<SongMap>),
}
use std::sync::Arc;

use std::sync::mpsc;
use std::thread;
impl Default for MicroApp {
    fn default() -> Self {
        Self {
            selected_song: "".to_owned(),
            songlist_id: HashMap::new(),
            songlist: vec![],
            label: "Hello World!".to_owned(),
            value: 2.7,
            channel: mpsc::channel(),
            selected_maps:HashMap::new()
        }
    }
}

impl epi::App for MicroApp {
    fn name(&self) -> &str {
        "egui template"
    }
    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    fn load(&mut self, storage: &dyn epi::Storage) {
        *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            selected_song,
            label,
            value,
            songlist,
            channel,
            songlist_id,
            selected_maps
        } = self;
        if let Ok(result) = channel.1.try_recv() {
            let m = songlist_id.get(&result.source_name).unwrap();
            songlist[*m].searched = true;
            songlist[*m].maps.push(result);
        }
        {
            //Set Fonts
            use egui::FontFamily;
            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert(
                "AliPH".to_owned(),
                std::borrow::Cow::Borrowed(include_bytes!("./Fonts/Alibaba-PuHuiTi-Medium.ttf")),
            );
            fonts
                .fonts_for_family
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "AliPH".to_owned());

            fonts
                .fonts_for_family
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .push("AliPH".to_owned());
            ctx.set_fonts(fonts);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "导入", |ui| {
                    if ui.button("从网易云Playlist").clicked() {
                        println!(
                            "Playlist Addr:{}",
                            format!(
                                "{}{}",
                                std::env::var("LOCALAPPDATA").unwrap().as_str(),
                                "\\Netease\\CloudMusic\\webdata\\file\\queue"
                            )
                        );
                        let playlist = json::parse(
                            std::fs::read_to_string(format!(
                                "{}{}",
                                std::env::var("LOCALAPPDATA").unwrap().as_str(),
                                "\\Netease\\CloudMusic\\webdata\\file\\queue"
                            ))
                            .unwrap()
                            .as_str(),
                        )
                        .unwrap();
                        for index in 0..playlist.len() {
                            let mut artists = vec![];
                            for artist in 0..playlist[index]["track"]["artists"].len() {
                                artists.push(playlist[index]["track"]["artists"][artist]["name"].to_string());
                            }
                            songlist_id.insert(
                                playlist[index]["track"]["name"].to_string(),
                                songlist.len(),
                            );
                            songlist.push(Song {
                                name: playlist[index]["track"]["name"].to_string(),
                                maps: vec![],
                                artists: artists,
                                searched: false,
                            })
                        }
                    }
                    // if ui.button("退出").clicked() {
                    //     frame.quit();
                    // }
                });
            });
        });
        let sl=songlist.clone();
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("歌曲列表");

            egui::ScrollArea::auto_sized().show(ui,|ui|{
                for i in songlist{
                    println!("{}",selected_song);
                    let selected=selected_song==&i.name;
                    if ui.selectable_label( selected,format!("{} - {}",i.name.as_str(),i.artists.join("/"))).clicked(){
                        if selected{
                            *selected_song="".to_owned();
                        }else{
                            *selected_song=i.name.clone();
                            if !i.searched{
                                let name=i.name.clone();
                                let sendere=channel.0.clone();
                                thread::spawn(move || {
                                    use urlencoding::encode;
                                    let sender=sendere;
                                    let client = reqwest::blocking::Client::new();
                                    let result= client.get(format!("https://beatsaver.com/api/search/text/0?q={}",encode(name.as_str())))
                                     .header("sec-ch-ua","\" Not;A Brand\";v=\"99\", \"Google Chrome\";v=\"91\", \"Chromium\";v=\"91\"")
                                    .header("Cache-Control","max-age=0")
                                    .header("sec-ch-ua-mobile","?0")
                                    .header("Upgrade-Insecure-Requests","1")
                                    .header("User-Agent","Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.164 Safari/537.36")
                                    .header("Accept","text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*;q=0.8,application/signed-exchange;v=b3;q=0.9")
                                    .header("Sec-Fetch-Site","none")
                                    .header("Sec-Fetch-Mode","navigate")
                                    .header("Sec-Fetch-User","?1")
                                    .header("Sec-Fetch-Dest","document")
                                    .header("Accept-Language","zh-CN,zh;q=0.9")
                                    .send().unwrap().text().unwrap();
                                    let result=json::parse(result.as_str()).unwrap();
                                    for i in result["docs"].members(){
                                        sender.send(SongMap{
                                            source_name:name.clone(),
                                            author:i["metadata"]["levelAuthorName"].to_string(),
                                            download_url:i["directDownload"].to_string(),
                                            name:i["name"].to_string(),
                                            selected:false
                                        }).unwrap();
                                    }
                                });
                            }
    
                        }
                    }
                    if selected{
                        if i.searched{
                            ui.vertical(|ui|{
                                ui.set_max_height(200.0);
                                egui::ScrollArea::auto_sized().show(ui,|ui|{
                                    for index in 0..i.maps.len() {
                                        if ui.selectable_label(i.maps[index].selected, format!("{}\n创作者：{}",i.maps[index].name,i.maps[index].author)).clicked(){
                                            i.maps[index].selected=!i.maps[index].selected;
                                        };
                                    }
                                });
                            });
                            // ui.add(eg)
                        }else{
                            ui.label("加载中……");
                        }
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
            });
        });

        
        egui::CentralPanel::default().show(ctx, |ui| {
            for i in sl{
                for map in &i.maps{
                    if map.selected{
                        ui.add(egui::Label::new(format!("{}({})\n{}",map.name,map.source_name,map.download_url)).text_color(Color32::from_rgb(255, 255, 255)));
                    }
                }
            }
            ui.heading("egui template");
            ui.hyperlink("https://github.com/emilk/egui_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/egui_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        egui::Window::new("使用教程").show(ctx, |ui| {
            ui.label("1.请打开 网易云音乐");
            ui.label("2.用您要导出的歌单替换播放列表（点击【播放全部】即可）");
            ui.label("3.退出（完全退出，不是关窗口！）网易云");
            ui.label("4.点击 顶部栏-导入-从网易云Playlist");
        });
    }
}
 