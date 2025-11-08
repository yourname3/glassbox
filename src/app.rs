use std::io::Read;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::{path::PathBuf, time::Duration};
use std::sync::Arc;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use egui::{Color32, ViewportId};
use rfd::FileDialog;
use rodio::Source;

use std::fmt::Write;

use crate::color_identifier::{self, Palette};

const DISPLAY_BUFFER_SIZE: usize = 2048;

pub struct TapOutputChannel {
    contents: [AtomicU32; DISPLAY_BUFFER_SIZE],
    write_ptr: AtomicUsize,
}

pub struct TapOutput {
    channels: [TapOutputChannel; 2]
}

impl TapOutputChannel {
    pub fn new() -> Self {
        Self {
            contents: core::array::from_fn(|_| AtomicU32::new(f32::to_bits(0.0))),
            write_ptr: 0.into(),
        }
    }

    pub fn write(&self, sample: f32) {
        let as_u32: u32 = f32::to_bits(sample);

        let dest = self.write_ptr.load(Ordering::Relaxed);
        self.write_ptr.store((dest + 1) % DISPLAY_BUFFER_SIZE, Ordering::Relaxed);

        self.contents[dest].store(as_u32, Ordering::Relaxed);
    }

    pub fn read_in_order(&self) -> Vec<f32> {
        let mut output = Vec::new();
        // The oldest value that was written is the one right after the write_ptr.
        let start = self.write_ptr.load(Ordering::Relaxed) + 1 % DISPLAY_BUFFER_SIZE;

        for i in 0..DISPLAY_BUFFER_SIZE {
            let as_bits = self.contents[(start + i) % DISPLAY_BUFFER_SIZE].load(Ordering::Relaxed);
            output.push(f32::from_bits(as_bits));
        }

        output
    }
}

impl TapOutput {
    pub fn new() -> Self {
        TapOutput {
            channels: [TapOutputChannel::new(), TapOutputChannel::new()]
        }
    }

    pub fn write(&self, sample: f32, channel: usize) {
        if channel > self.channels.len() { return; }
        self.channels[channel].write(sample);
    }

    pub fn read_in_order(&self) -> [Vec<f32>; 2] {
        [self.channels[0].read_in_order(), self.channels[1].read_in_order()]
    }
}

pub struct Tap<S> {
    inner: S,
    output: Arc<TapOutput>,

    cur_channel: usize,
}

impl<S> Tap<S> {
    fn new(inner: S, output: Arc<TapOutput>) -> Self {
        Tap {
            inner,
            output,

            cur_channel: 0,
        }
    }
}

impl<S> Iterator for Tap<S>
where
    S: rodio::Source + Iterator<Item = rodio::Sample>,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;
        
        self.output.write(sample, self.cur_channel);
        self.cur_channel = (self.cur_channel + 1) % self.channels() as usize;

        Some(sample)
    }
}

impl<S: rodio::Source> rodio::Source for Tap<S> {
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }
}

pub struct Song {
    path: PathBuf,

    title: String,
    album: Option<String>,
    artist: Option<String>,

    // String representing the concatenated artist and album, e.g.
    // Musician - The Song
    artist_album: Option<String>,
}

impl Song {
    pub fn new(path: PathBuf) -> Self {
        let mut title = path.file_name().map(|t| { t.to_string_lossy().to_string() });
        let mut album = None;
        let mut artist = None;

        if let Ok(tags) = taglib::File::new(&path) {
            if let Ok(tags) = tags.tag() {
                if let Some(tags_title) = tags.title() {
                    title = Some(tags_title);
                }
                album = tags.album();
                artist = tags.artist();
            }
        }

        let title = title.unwrap_or_else(|| "<unknown>".into());
        
        let artist_album = match (&artist, &album) {
            (None, None) => None,
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (Some(a), Some(b)) => Some(format!("{a} - {b}")),
        };

        Song {
            path,
            title,
            album,
            artist,
            artist_album,
        }
    }
}

pub struct Album {
    songs: Vec<Song>,

    // The cover.png for the folder, if there is one.
    album_cover: Option<egui::ColorImage>,

    // Colors for the album cover, if there was an album cover.
    palette: Palette,
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
                let state = song.artist_album.as_ref().map(|x| x.clone()).unwrap_or_else(|| "".into());

                let _ = self.client.set_activity(activity::Activity::new()
                    .details(&song.title)
                    .state(&state)
                    // TODO: Add .timestamps() for more.

                    // Show the song name as the status display
                    .status_display_type(activity::StatusDisplayType::Details)
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

    tap_output: Arc<TapOutput>,

    current_album_art: Option<egui::TextureHandle>,
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

            tap_output: Arc::new(TapOutput::new()),
            current_album_art: None,
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

            let tap = Tap::new(decoder, self.tap_output.clone());

            total_duration += tap.total_duration().unwrap();
            playback.duration_map.push(total_duration);

            playback.sink.append(tap);
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
                let supported_exts = ["ogg", "flac", "mp3", "wav"];

                for candidate in supported_exts {
                    if ext == candidate {
                        songs.push(Song::new(entry.path()));
                    }
                }
            }
        }

        // Clear the loaded texture so that we will reload it.
        self.current_album_art = None;

        let mut album_cover = None;

        let possible_covers = ["cover.png", "cover.jpg"];
        for candidate in possible_covers {
            let possible_cover_path = path.join(candidate);
            
            log::info!("testing for cover image @ {:?}", possible_cover_path);

            let img = image::ImageReader::open(possible_cover_path);
            if let Ok(img) = img {
                if let Ok(decode) = img.decode() {
                    let rgba = decode.to_rgba8();
                    let size = [rgba.width() as _, rgba.height() as _];
                    let pixels = rgba.as_flat_samples();

                    // Load the album art into a ColorImage so that we can process
                    // its pixel data.
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size,
                        pixels.as_slice());

                    album_cover = Some(color_image);
                    break;
                }
            }
        }

        let palette = color_identifier::identify_colors(album_cover.as_ref());

        self.current_album = Some(Album {
            songs,
            album_cover,
            palette,
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

        let monitor_size = ctx.input(|i| i.viewport().monitor_size);
        if let Some(size) = monitor_size {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(size.x, 200.0)));
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default()
                //.fill(Color32::from_rgba_unmultiplied(176, 134, 189, 127))
                .fill(Color32::TRANSPARENT)
                .inner_margin(3.0)
            )
            .show(ctx, |ui| {
                // I think this will work?
                let total_width = ui.available_width();

                let mut album_cover = None;
                let mut palette = color_identifier::default();

                // Allocate space for the album art so that the label()s go
                // below it.
                ui.allocate_space(egui::vec2(100.0, 100.0));

                ctx.style_mut(|style| {
                    style.visuals.override_text_color = Some(Color32::WHITE);
                });
                if let Some(album) = self.current_album.as_ref() {
                    palette = album.palette;

                    if let None = self.current_album_art && let Some(cover) = &album.album_cover {
                        let texture = ctx.load_texture(
                            "album-cover",
                            // TODO: This can probably be some sort of take() instead.
                            cover.clone(),
                            egui::TextureOptions::LINEAR
                        );
                        self.current_album_art = Some(texture);

                        
                    }

                    if let Some(cover) = &self.current_album_art {
                        let cover = egui::Image::from_texture((cover.id(), cover.size_vec2()))
                            .corner_radius(5.0);
                        album_cover = Some(cover);
                    }
                    

                    // if let Some(cover) = &album.album_cover {
                    //     let cover = egui::Image::from_bytes("bytes://album_cover.png", cover.clone())
                    //         .max_size(egui::vec2(100.0, 100.0))
                    //         .fit_to_exact_size(egui::vec2(100.0, 100.0))
                    //         .texture_options(egui::TextureOptions::LINEAR)
                    //         .corner_radius(5.0)
                    //         .show_loading_spinner(true);
                        
                    //     album_cover = Some(cover);
                    // }

                    if current_playing_idx >= 0 && current_playing_idx < album.songs.len() as isize {
                        let song = &album.songs[current_playing_idx as usize];

                        let song_frame_color = Color32::from_hex("#272727c7").unwrap();
                        
                        // TODO: Having some sort of text stroke/outline would be
                        // maybe preferable than this setup.

                        egui::Area::new(egui::Id::new("topwindow_song_info"))
                            .fixed_pos((105.0, 5.0))
                            .show(ctx, |ui| {
                                egui::Frame::default()
                                    .fill(song_frame_color)
                                    .corner_radius(5)
                                    .inner_margin(5)
                                    .outer_margin(0)
                                    
                                    .show(ui, |ui| {
                                        ui.label(&song.title);
                                    }
                                );
                            }
                        );

                        if let Some(artist_album) = &song.artist_album {
                            egui::Area::new(egui::Id::new("topwindow_artist_album"))
                                .pivot(egui::Align2::LEFT_BOTTOM)
                                .fixed_pos((105.0, 95.0))
                                .show(ctx, |ui| {
                                    egui::Frame::default()
                                        .fill(song_frame_color)
                                        .corner_radius(5)
                                        .inner_margin(5)
                                        .outer_margin(0)
                                        .show(ui, |ui| {
                                            ui.label(artist_album);
                                        }
                                    );
                                }
                            );
                        }
                    }
                }

                // TODO: Maybe keep this as a preallocated buffer and re-use it
                // each frame
                let samples = self.tap_output.read_in_order();

                let max_x = total_width - 10.0;
                let min_x = 100.0 + 10.0;
                
                let samples_to_points = |samples: &Vec<_>| {
                    let mut points = Vec::new();
                    let mut points_high = Vec::new();
                    let mut points_low = Vec::new();

                    let x_factor = (max_x - min_x) / (samples.len() as f32);
                    
                    for (idx, sample) in samples.iter().enumerate() {
                        let point = egui::pos2(min_x + idx as f32 * x_factor, sample * 50.0 + 50.0);
                        let high = point + egui::vec2(0.0, -1.5);
                        let low = point + egui::vec2(0.0, 1.5);

                        points.push(point);
                        points_high.push(high);
                        points_low.push(low);
                    }

                    (points, points_high, points_low)
                };

                let left = samples_to_points(&samples[0]);
                let right = samples_to_points(&samples[1]);

                let painter = ui.painter();

                let stroke_bg = egui::epaint::PathStroke::new_uv(6.0, move |rect, uv| {
                    let t = (uv.y - rect.top()) / rect.height();
                    palette.bg_a.lerp_to_gamma(palette.fg_a, t)
                });

                painter.line(left.0.clone(), stroke_bg.clone());
                painter.line(right.0.clone(), stroke_bg);

                //painter.line(left.2, (3.0, Color32::from_gray(140)));
                //painter.line(right.2, (3.0, Color32::from_gray(140)));

                //painter.line(left.1, (3.0, Color32::from_gray(230)));
                //painter.line(right.1, (3.0, Color32::from_gray(230)));

                painter.line(left.0, egui::epaint::PathStroke::new_uv(4.0, move |rect, uv| {
                    let t = (uv.x - rect.left()) / rect.width();
                    palette.fg_a.lerp_to_gamma(palette.fg_b, t)
                }));
                painter.line(right.0, egui::epaint::PathStroke::new_uv(4.0, move |rect, uv| {
                    let t = (uv.x - rect.left()) / rect.width();
                    palette.bg_a.lerp_to_gamma(palette.bg_b, t)
                }));

                if let Some(cover) = album_cover {
                    // Draw the album cover over the big line.
                    // I guess we could also draw the line to the side...
                    cover.paint_at(ui, egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0)));
                }
            }
        );

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
