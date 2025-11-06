use std::{path::PathBuf, time::Duration};
use std::sync::Arc;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use egui::{Color32, ViewportId};
use rfd::FileDialog;
use rodio::Source;

pub struct Song {
    path: PathBuf,

    title: String,
}

impl Song {
    pub fn new(path: PathBuf) -> Self {
        let mut title = path.file_name().map(|t| { t.to_string_lossy().to_string() });

        if let Ok(tags) = taglib::File::new(&path) {
            if let Ok(tags) = tags.tag() {
                if let Some(tags_title) = tags.title() {
                    title = Some(tags_title);
                }
            }
        }

        let title = title.unwrap_or_else(|| "<unknown>".into());

        Song {
            path,
            title,
        }
    }
}

pub struct Album {
    songs: Vec<Song>,
}

pub struct AudioPlayback {
    stream_handle: rodio::OutputStream,
    sink: rodio::Sink,
    /// Keeps track of which durations map to which songs.
    duration_map: Vec<Duration>,
}

impl AudioPlayback {
    pub fn open() -> Option<Self> {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().ok()?;
        let sink = rodio::Sink::connect_new(stream_handle.mixer());
        
        Some(AudioPlayback {
            stream_handle,
            sink,
            duration_map: Vec::new(),
        })
    }
}

pub struct Discord {
    client: DiscordIpcClient,
}

impl Discord {
    const CLIENT_ID: &str = "1436060096294682758";

    pub fn open() -> Option<Self> {
        let mut client = discord_rich_presence::DiscordIpcClient::new(Self::CLIENT_ID);
        // TODO: Button for reconnecting? Maybe just a "enable discord" toggle?
        client.connect().ok()?;

        let _ = client.set_activity(activity::Activity::new()
            .details("Nothing playing")
            .activity_type(activity::ActivityType::Listening)
        );

        Some(Discord { client })
    }

    pub fn update_song(&mut self, song: Option<&Song>) {
        match song {
            Some(song) => {
                let _ = self.client.set_activity(activity::Activity::new()
                    .details(&song.title)
                    .activity_type(activity::ActivityType::Listening)
                );
            }
            None => {
                let _ = self.client.set_activity(activity::Activity::new()
                    .details("Nothing playing")
                    .activity_type(activity::ActivityType::Listening)
                );
            }
        }
    }
}

pub struct App {
    current_album: Option<Album>,
    playback: Option<AudioPlayback>,

    // Index of the last song we were playing.
    last_playing_idx: isize,

    discord: Option<Discord>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // NOTE: I think this stuff requires serde? But it is very cool.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        //     Default::default()
        // }
        App {
            current_album: None,
            playback: AudioPlayback::open(),
            last_playing_idx: -1,
            discord: Discord::open(),
        }
    }

    fn play_album(&mut self) {
        let Some(playback) = self.playback.as_mut() else { return; };
        let Some(album) = self.current_album.as_ref() else { return; };

        playback.sink.clear();

        playback.duration_map.clear();
        let mut total_duration = Duration::ZERO;

        for song in &album.songs {
            // TODO: Report errors somehow?
            let Ok(file) = std::fs::File::open(&song.path) else { continue; };
            let Ok(decoder) = rodio::Decoder::try_from(file) else {
                continue;
            };

            total_duration += decoder.total_duration().unwrap();
            playback.duration_map.push(total_duration);

            playback.sink.append(decoder);
        }

        // Force playing update
        self.last_playing_idx = -1;
    }

    fn open_album(&mut self, path: &PathBuf) -> std::io::Result<()> {
        log::info!("opening album: {:?}", path);
        let dir = std::fs::read_dir(path)?;

        let mut songs: Vec<Song> = Vec::new();

        for entry in dir {
            let Ok(entry) = entry else { continue; };
            if let Some(ext) = entry.path().extension() {
                if ext == "ogg" {
                    songs.push(Song::new(entry.path()));
                }
            }
        }

        self.current_album = Some(Album {
            songs
        });

        self.play_album();

        Ok(())
    }

    fn pause(&mut self) {
        let Some(playback) = self.playback.as_mut() else { return };
        playback.sink.pause();
    }

    fn resume(&mut self) {
        let Some(playback) = self.playback.as_mut() else { return };
        playback.sink.play();
    }

    fn is_paused(&self) -> bool {
        let Some(playback) = self.playback.as_ref() else { return true; };
        playback.sink.is_paused()
    }

    fn compute_playing_idx(&self) -> isize {
        let Some(playback) = self.playback.as_ref() else { return -1 };
        let Some(album) = self.current_album.as_ref() else { return -1 };

        return (album.songs.len() - playback.sink.len()) as isize;
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        let current_playing_idx = self.compute_playing_idx();

        #[cfg(target_os = "windows")]
        crate::os::apply_window_transparency(_frame);

        egui::CentralPanel::default()
            .frame(egui::Frame::default()
                .fill(Color32::from_rgba_unmultiplied(176, 134, 189, 127)))
            .show(ctx, |ui| {

        });

        // TODO:
        // It would be ideal if we could defer the other viewport, but this
        // does really complicate the ownership.

        ctx.show_viewport_immediate(ViewportId::from_hash_of("glassbox_controls"),
            egui::ViewportBuilder::default()
                .with_inner_size((400.0, 300.0)),
            |ctx, class| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Control Window");

                    if ui.button("Open").clicked() {
                        if let Some(album) = FileDialog::new().pick_folder() {
                            self.open_album(&album);
                            
                        }
                    }

                    let paused = self.is_paused();
                    if ui.button(if paused { "Play" } else { "Pause" }).clicked() {
                        if paused { self.resume(); } else { self.pause(); }
                    }

                    if let Some(album) = self.current_album.as_ref() {
                        if current_playing_idx >= 0 && current_playing_idx < album.songs.len() as isize {
                            ui.horizontal(|ui| {
                                ui.label("Now Playing: ");
                                let title = &album.songs[current_playing_idx as usize].title;
                                ui.label(title);
                            });
                        }

                        for song in &album.songs {
                            ui.label(&song.title);
                        }

                        if current_playing_idx != self.last_playing_idx {
                            let song = if current_playing_idx >= 0 && current_playing_idx < album.songs.len() as isize {
                                Some(&album.songs[current_playing_idx as usize])
                            }
                            else { None };

                            if let Some(discord) = self.discord.as_mut() {
                                discord.update_song(song);
                            }
                        }
                    }
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    ctx.send_viewport_cmd_to(ViewportId::ROOT, egui::ViewportCommand::Close);
                }
            });

        ctx.request_repaint();
        self.last_playing_idx = current_playing_idx;
    }
}
