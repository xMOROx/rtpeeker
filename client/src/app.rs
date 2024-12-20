use crate::app::common::table::TableBase;
use eframe::egui;
use egui::{ComboBox, Label, TextWrapMode, Ui, Widget};
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use log::{error, warn};
use netpix_common::{MpegtsStreamKey, Request, Response, RtpStreamKey, Source};

use packets_table::PacketsTable;
use rtcp_packets_table::RtcpPacketsTable;
use rtp_packets_table::RtpPacketsTable;
use rtp_streams_table::RtpStreamsTable;
use std::collections::HashMap;
use std::sync::Arc;

use mpegts_info_table::MpegTsInformationTable;
use mpegts_packets_table::MpegTsPacketsTable;
use mpegts_streams_table::MpegTsStreamsTable;

use tab::Tab;

use crate::app::tab::{MpegTsSection, RtpSection};
use crate::streams::RefStreams;
use rtp_streams_plot::RtpStreamsPlot;

mod packets_table;
mod rtcp_packets_table;
mod rtp_packets_table;
mod rtp_streams_plot;
mod rtp_streams_table;

mod mpegts_info_table;
mod mpegts_packets_table;
mod mpegts_streams_table;

mod utils;

mod tab;

mod common;

const SOURCE_KEY: &str = "source";
const TAB_KEY: &str = "tab";

pub struct App {
    ws_sender: WsSender,
    ws_receiver: WsReceiver,
    is_capturing: bool,
    // some kind of sparse vector would be the best
    // but this will do
    streams: RefStreams,
    sources: Vec<Source>,
    selected_source: Option<Source>,
    tab: Tab,
    // would rather keep this in `Tab` enum
    // but it proved to be inconvinient
    packets_table: PacketsTable,
    rtp_packets_table: RtpPacketsTable,
    rtcp_packets_table: RtcpPacketsTable,
    rtp_streams_table: RtpStreamsTable,
    rtp_streams_plot: RtpStreamsPlot,

    mpegts_packets_table: MpegTsPacketsTable,
    mpegts_streams_table: MpegTsStreamsTable,
    mpegts_info_table: MpegTsInformationTable,
    discharged_count: usize,
    overwritten_count: usize,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.is_capturing {
            self.receive_packets()
        }

        self.build_side_panel(ctx);
        self.build_top_bar(ctx, frame);
        self.build_bottom_bar(ctx);

        match self.tab {
            Tab::Packets => self.packets_table.ui(ctx),
            Tab::RtpSection(section) => match section {
                RtpSection::Packets => self.rtp_packets_table.ui(ctx),
                RtpSection::RtcpPackets => self.rtcp_packets_table.ui(ctx),
                RtpSection::Streams => self.rtp_streams_table.ui(ctx),
                RtpSection::Plot => self.rtp_streams_plot.ui(ctx),
            },
            Tab::MpegTsSection(section) => match section {
                MpegTsSection::Packets => self.mpegts_packets_table.ui(ctx),
                MpegTsSection::Streams => self.mpegts_streams_table.ui(ctx),
                MpegTsSection::Information => self.mpegts_info_table.ui(ctx),
            },
        };
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let host = &cc.integration_info.web_info.location.host;
        let uri = format!("ws://{}/ws", host);

        let ctx = cc.egui_ctx.clone();
        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message

        let (ws_sender, ws_receiver) =
            ewebsock::connect_with_wakeup(uri, wakeup).expect("Unable to connect to WebSocket");

        let streams = RefStreams::default();
        let packets_table = PacketsTable::new_with_sender(streams.clone(), ws_sender.clone());
        let rtp_packets_table = RtpPacketsTable::new(streams.clone());
        let rtcp_packets_table = RtcpPacketsTable::new(streams.clone());
        let rtp_streams_table =
            RtpStreamsTable::new_with_sender(streams.clone(), ws_sender.clone());
        let rtp_streams_plot = RtpStreamsPlot::new(streams.clone());

        let mpegts_packets_table = MpegTsPacketsTable::new(streams.clone());
        let mpegts_streams_table = MpegTsStreamsTable::new(streams.clone());
        let mpegts_info_table = MpegTsInformationTable::new(streams.clone());

        let (tab, selected_source) = get_initial_state(cc);

        Self {
            ws_sender,
            ws_receiver,
            is_capturing: true,
            streams,
            sources: Vec::new(),
            selected_source,
            tab,
            packets_table,
            rtp_packets_table,
            rtcp_packets_table,
            rtp_streams_table,
            rtp_streams_plot,
            mpegts_packets_table,
            mpegts_streams_table,
            mpegts_info_table,
            discharged_count: 0,
            overwritten_count: 0,
        }
    }

    fn build_side_panel(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = (0.0, 8.0).into();
        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size = 20.0;
        }

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .default_width(32.0)
            .show(ctx, |ui| {
                ui.set_style(style);
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);

                    let button = side_button("▶");
                    let resp = ui
                        .add_enabled(!self.is_capturing, button)
                        .on_hover_text("Resume packet capturing");
                    if resp.clicked() {
                        self.is_capturing = true
                    }

                    let button = side_button("⏸");
                    let resp = ui
                        .add_enabled(self.is_capturing, button)
                        .on_hover_text("Stop packet capturing");
                    if resp.clicked() {
                        self.is_capturing = false
                    }

                    let button = side_button("🗑");
                    let resp = ui
                        .add(button)
                        .on_hover_text("Discard previously captured packets");
                    if resp.clicked() {
                        self.streams.borrow_mut().clear();
                    }

                    //TODO: implement more optimal way to do that - with lots of packages it is too much for wasm to handle this
                    let button = side_button("↻");
                    let resp = ui
                        .add(button)
                        .on_hover_text("Refetch all previously captured packets");
                    if resp.clicked() {
                        self.streams.borrow_mut().clear();
                        self.refetch_packets()
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);

                    egui::widgets::global_theme_preference_switch(ui);
                });
            });
    }

    fn build_top_bar(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let selected = self.tab.display_name();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                self.build_dropdown_source(ui, frame);
                ui.separator();
                self.build_menu_button(ui, frame);
                Label::new(selected).ui(ui);
            });
        });
    }

    fn build_menu_button(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        ui.menu_button("📑 Open tabs", |ui| {
            ui.heading("Tabs");

            let menu_sections = Tab::sections();

            for (label, sections) in menu_sections {
                ui.menu_button(label, |ui| {
                    for tab in sections {
                        let resp = ui.selectable_value(&mut self.tab, tab, tab.display_name());
                        if resp.clicked() {
                            if let Some(storage) = frame.storage_mut() {
                                storage.set_string(TAB_KEY, tab.to_string());
                            }
                        }
                    }
                });
            }
        });
    }

    fn build_dropdown_source(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let selected = match self.selected_source {
            Some(ref source) => source.to_string(),
            None => "Select packets source...".to_string(),
        };

        ComboBox::from_id_salt("source_picker")
            .width(300.0)
            .wrap_mode(TextWrapMode::Extend)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                let mut was_changed = false;

                for source in self.sources.iter() {
                    let resp = ui.selectable_value(
                        &mut self.selected_source,
                        Some(source.clone()),
                        source.to_string(),
                    );
                    if resp.clicked() {
                        was_changed = true;
                    }
                }

                if was_changed {
                    self.streams.borrow_mut().clear();
                    self.change_source_request();
                    if let Some(storage) = frame.storage_mut() {
                        let source = self.selected_source.as_ref().unwrap();
                        storage.set_string(SOURCE_KEY, source.to_string());
                    }
                }
            });
    }

    fn build_bottom_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                let streams = self.streams.borrow();
                let count = streams.packets.id_count();
                let count_label = format!("Packets: {}", count);

                let captured_count = streams.packets.len();
                let captured_label = format!("Captured: {}", captured_count);

                let discharged_label = format!("Discharged: {}", self.discharged_count);
                let overwritten_label = format!("Overwritten: {}", self.overwritten_count);
                let label = format!(
                    "{} • {} • {} • {}",
                    count_label, captured_label, discharged_label, overwritten_label
                );
                ui.label(label);
            });
        });
    }

    fn receive_packets(&mut self) {
        while let Some(msg) = self.ws_receiver.try_recv() {
            let WsEvent::Message(msg) = msg else {
                warn!("Received special message: {:?}", msg);
                continue;
            };

            let WsMessage::Binary(msg) = msg else {
                warn!("Received unexpected message: {:?}", msg);
                continue;
            };

            // Handle single message at a time
            let Ok(response) = Response::decode(&msg) else {
                error!("Failed to decode response message");
                continue;
            };

            match response {
                Response::Packet(packet) => {
                    let mut streams = self.streams.borrow_mut();
                    streams.add_packet(packet);
                }
                Response::Sources(sources) => {
                    if let Some(ref source) = self.selected_source {
                        if !sources.contains(source) {
                            self.selected_source = None;
                        } else {
                            self.change_source_request();
                        }
                    }
                    self.sources = sources;
                }
                Response::Sdp(stream_key, sdp) => {
                    let mut streams = self.streams.borrow_mut();
                    if let Some(stream) = streams.rtp_streams.get_mut(&stream_key) {
                        stream.add_sdp(sdp);
                    }
                }
                Response::PacketsStats(stats) => {
                    self.discharged_count = stats.discharged;
                    self.overwritten_count = stats.overwritten;
                }
            }
        }
    }

    fn refetch_packets(&mut self) {
        let request = Request::FetchAll;
        let Ok(msg) = request.encode() else {
            error!("Failed to encode a request message");
            return;
        };
        let msg = WsMessage::Binary(msg);

        self.ws_sender.send(msg);
    }

    fn change_source_request(&mut self) {
        let selected = self.selected_source.as_ref().unwrap().clone();
        let request = Request::ChangeSource(selected);
        let Ok(msg) = request.encode() else {
            log::error!("Failed to encode a request message");
            return;
        };
        let msg = WsMessage::Binary(msg);
        self.ws_sender.send(msg);
    }
}

fn get_initial_state(cc: &eframe::CreationContext<'_>) -> (Tab, Option<Source>) {
    if let Some(storage) = cc.storage {
        let tab = match storage.get_string(TAB_KEY) {
            Some(tab_str) => Tab::from_string(tab_str).unwrap_or(Tab::Packets),
            _ => Tab::Packets,
        };

        let source = match storage.get_string(SOURCE_KEY) {
            Some(src_str) => Source::from_string(src_str),
            _ => None,
        };

        (tab, source)
    } else {
        (Tab::Packets, None)
    }
}

fn side_button(text: &str) -> egui::Button {
    egui::Button::new(text)
        .min_size((30.0, 30.0).into())
        .rounding(egui::Rounding::same(9.0))
}

pub fn is_rtp_stream_visible(
    streams_visibility: &mut HashMap<RtpStreamKey, bool>,
    key: RtpStreamKey,
) -> &mut bool {
    streams_visibility.entry(key).or_insert(true)
}
