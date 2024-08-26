use std::{collections::HashMap, sync::mpsc};

use egui::Context;

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
        channel: channel,
    };
    let options = eframe::NativeOptions::default();
    _ = eframe::run_native("Netup", options, Box::new(|_| Box::<NetupApp>::new(app)));
}

impl eframe::App for NetupApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.collect_events();
        egui::CentralPanel::default().show(ctx, |ui| {
            for msg in self.messages.iter() {
                ui.label(format!(
                    "Message {} sent at {} received at {:?}",
                    msg.idx, msg.snt_time, msg.rcv_time
                ));
            }
        });
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
}
