use std::{borrow::BorrowMut, collections::HashMap, sync::mpsc};

use egui::{Color32, Context, Ui, Vec2b};
use egui_plot::{Plot, PlotPoints, Points};
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

use crate::utils;

pub enum NetupEvent {
    MessageSent(MessageSentEvent),
    MessageReceived(MessageReceivedEvent),
}

pub struct MessageSentEvent {
    idx: u64,
    snt_time: u128,
}

impl MessageSentEvent {
    pub fn new(idx: u64, snt_time: u128) -> Self {
        Self { idx, snt_time }
    }
}

pub struct MessageReceivedEvent {
    idx: u64,
    rcv_time: u128,
}

impl MessageReceivedEvent {
    pub fn new(idx: u64, rcv_time: u128) -> Self {
        Self { idx, rcv_time }
    }
}

struct MessageState {
    #[allow(dead_code)]
    idx: u64,
    snt_time: u128,
    rcv_time: Option<u128>,
}

#[derive(Default)]
struct MessageMap {
    msgs: HashMap<u64, MessageState>,
}

impl MessageMap {
    pub fn add_sent(&mut self, idx: u64, snt_time: u128) {
        self.msgs.insert(
            idx,
            MessageState {
                idx,
                snt_time,
                rcv_time: None,
            },
        );
    }

    pub fn add_received(&mut self, idx: u64, rcv_time: u128) {
        if let Some(msg) = self.msgs.get_mut(&idx) {
            msg.rcv_time = Some(rcv_time);
        }
    }

    pub fn last_x_ms(&self, window: u128) -> impl Iterator<Item = &MessageState> {
        let current_time = utils::get_timestamp();
        self.msgs
            .values()
            .filter(move |m| (current_time - m.snt_time) < window)
    }

    pub fn iter(&self) -> impl Iterator<Item = &MessageState> {
        self.msgs.values()
    }
}

struct NetupApp {
    messages: MessageMap,
    channel: mpsc::Receiver<NetupEvent>,
}

pub fn run_gui(channel: mpsc::Receiver<NetupEvent>) {
    let app = NetupApp {
        messages: MessageMap::default(),
        channel,
    };
    let options = eframe::NativeOptions::default();
    _ = eframe::run_native("Netup", options, Box::new(|_| Box::<NetupApp>::new(app)));
}

impl eframe::App for NetupApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.collect_events();
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_ui(ui);
        });
        ctx.request_repaint();
    }
}

/// The maximum delay in milliseconds before a message is considered lost
/// A message is also considered lost if it is not received within this time
const MAX_DELAY: u128 = 500;

/// The amount of time between two packets before a period of silence is detected
const MAX_SILENCE: u128 = 50;

/// How far back the graphs should go
const LOOKBACK_PERIOD: u128 = 60000 * 5;

/// The amount of jitter to add to the points
const JITTER_STRENGTH: f64 = 0.1;

impl NetupApp {
    fn collect_events(&mut self) {
        while let Ok(event) = self.channel.try_recv() {
            match event {
                NetupEvent::MessageSent(msg) => {
                    self.messages.add_sent(msg.idx, msg.snt_time);
                }
                NetupEvent::MessageReceived(msg) => {
                    self.messages.add_received(msg.idx, msg.rcv_time);
                }
            }
        }
    }

    fn draw_ui(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.draw_delay_plot(ui);
        });
    }

    fn jitter_points(x: u128, y: u128, id: u64, strength: f64) -> [f64; 2] {
        if JITTER_STRENGTH == 0.0 {
            return [x as f64, y as f64];
        }

        let mut buf = [0; 32];
        buf[..8].copy_from_slice(&id.to_le_bytes());
        let mut rng = StdRng::from_seed(buf);
        let x_offset = rng.gen_range(-strength..strength);
        let y_offset = rng.gen_range(-strength..strength);

        [x as f64 + x_offset, y as f64 + y_offset]
    }

    fn draw_delay_plot(&mut self, ui: &mut Ui) {
        let now = utils::get_timestamp();
        let lost_points: PlotPoints = self
            .messages
            .last_x_ms(LOOKBACK_PERIOD)
            .filter(|m| match m.rcv_time {
                Some(rcv_time) => rcv_time - m.snt_time > MAX_DELAY,
                None => now - m.snt_time > MAX_DELAY,
            })
            .map(|m| {
                let x = m.snt_time;
                let y = -2.0;
                NetupApp::jitter_points(x, y as u128, m.idx, 1.)
            })
            .collect();

        let lost_points = Points::new(lost_points).color(Color32::DARK_BLUE);

        let points: PlotPoints = self
            .messages
            .last_x_ms(LOOKBACK_PERIOD)
            .filter(|m| m.rcv_time.is_some())
            .map(|m| {
                let x = m.snt_time;
                let y = m.rcv_time.unwrap() - m.snt_time;
                let id = m.idx;
                NetupApp::jitter_points(x, y, id, 0.3)
            })
            .collect();
        let plot_points = Points::new(points).color(Color32::RED);

        Plot::new("delay_plot")
            .link_axis("packet_plots", true, false)
            .show_grid(true)
            .x_axis_label("Time")
            .y_axis_label("Delay in ms")
            .x_axis_formatter(|a, _, _| utils::format_timestamp_ms(a.value as u128))
            .show_grid(false)
            .show(ui, |plot_ui| {
                plot_ui.add(plot_points);
                plot_ui.add(lost_points);
            });
    }
}
