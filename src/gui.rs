use std::{collections::HashMap, sync::mpsc};

use egui::{Color32, Context, Ui, Vec2b};
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, Points};

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
    idx: u64,
    snt_time: u128,
    rcv_time: Option<u128>,
}

#[derive(Default)]
struct MessageMap {
    msgs: HashMap<u64, MessageState>,
}

impl MessageMap {
    fn add_sent(&mut self, idx: u64, snt_time: u128) {
        self.msgs.insert(
            idx,
            MessageState {
                idx,
                snt_time,
                rcv_time: None,
            },
        );
    }

    fn add_received(&mut self, idx: u64, rcv_time: u128) {
        if let Some(msg) = self.msgs.get_mut(&idx) {
            msg.rcv_time = Some(rcv_time);
        }
    }

    fn iter(&self) -> impl Iterator<Item = &MessageState> {
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
        let current_time = utils::get_timestamp();
        let points: PlotPoints = self
            .messages
            .iter()
            .filter(|m| m.rcv_time.is_some())
            .filter(|m| {
                return (current_time - m.snt_time) < 60000 * 5;
            })
            .map(|msg| {
                [
                    msg.snt_time as f64,
                    (msg.rcv_time.unwrap() - msg.snt_time) as f64,
                ]
            })
            .collect();
        let plot_points = Points::new(points).color(Color32::RED);

        Plot::new("delay_plot")
            .show_grid(true)
            .x_axis_label("Time")
            .y_axis_label("Delay in ms")
            .x_axis_formatter(|a, _, _| utils::format_timestamp_ms(a.value as u128))
            .show_grid(false)
            .show(ui, |plot_ui| {
                plot_ui.add(plot_points);
            });
    }
}
