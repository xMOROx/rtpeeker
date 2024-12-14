use super::filters::{self, parse_filter, FilterContext, FilterType};
use super::types::PacketInfo;
use crate::app::common::{TableBase, TableConfig};
use crate::app::utils::{FilterHelpContent, FilterInput};
use crate::declare_table;
use crate::filter_system::FilterExpression;
use crate::streams::rtpStream::RtpInfo;
use crate::streams::RefStreams;
use eframe::epaint::Color32;
use egui::RichText;
use egui_extras::TableBuilder;
use egui_extras::{Column, TableBody};
use netpix_common::packet::SessionPacket;
use std::collections::HashMap;

declare_table!(RtpPacketsTable, FilterType, {
    height(30.0);
    striped(true);
    resizable(true);
    stick_to_bottom(true);
    columns(
        column(None, 40.0, Some(50.0), false, true),
        column(None, 80.0, Some(80.0), false, true),
        column(None, 130.0, None, false, true),
        column(None, 130.0, None, false, true),
        column(None, 80.0, None, false, true),
        column(None, 80.0, None, false, true),
        column(None, 80.0, None, false, true),
        column(None, 80.0, None, false, true),
        column(None, 80.0, None, false, true),
        column(None, 80.0, None, false, true),
        column(None, 80.0, None, false, true),
        column(None, 50.0, None, false, true),
        column(None, 80.0, None, false, true),
        column(None, 80.0, None, false, true),
    )
});

pub struct RtpPacketsTable {
    streams: RefStreams,
    filter_input: FilterInput,
    config: TableConfig,
}

impl TableBase for RtpPacketsTable {
    fn new(streams: RefStreams) -> Self {
        let help = FilterHelpContent::builder("RTP Packet Filters")
            .filter("source", "Filter by source IP address")
            .filter("dest", "Filter by destination IP address")
            .filter("alias", "Filter by stream alias")
            .filter("padding", "Filter by padding presence (+: true, -: false)")
            .filter("extension", "Filter by extension presence (+: true, -: false)")
            .filter("marker", "Filter by marker presence (+: true, -: false)")
            .filter("seq", "Filter by sequence number")
            .filter("timestamp", "Filter by RTP timestamp")
            .filter("payload", "Filter by payload size")
            .example("source:10.0.0 AND payload:>1000")
            .example("(dest:192.168 OR dest:10.0.0) AND NOT seq:0")
            .example("padding:+ AND timestamp:>1000000")
            .example("padding:+ AND extension:-")

            .build();

        Self {
            streams,
            filter_input: FilterInput::new(help),
            config: TableConfig::default(),
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        if self.filter_input.show(ctx) {
            self.check_filter();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.build_table(ui);
        });
    }

    fn check_filter(&mut self) {
        let filter = self.filter_input.get_filter();
        if filter.is_empty() {
            self.filter_input.set_error(None);
            return;
        }

        let result = parse_filter(&filter.to_lowercase());
        self.filter_input.set_error(result.err());
    }
}

impl RtpPacketsTable {
    fn build_header(&mut self, header: &mut egui_extras::TableRow) {
        let headers = [
            ("No.", "Packet number (including skipped packets)"),
            ("Time", "Packet arrival timestamp"),
            ("Source", "Source IP address and port"),
            ("Destination", "Destination IP address and port"),
            ("Padding", "RTP packet contains additional padding"),
            ("Extension", "RTP packet contains additional header extensions"),
            ("Marker", "RTP marker\nFor audio type it might say that it is first packet after silence\nFor video, marker might say that it is last packet of a frame"),
            ("Payload Type", "RTP payload type informs the receiver about the codec or encoding"),
            ("Sequence Number", "RTP sequence number ensures correct order and helps detect packet loss"),
            ("Timestamp", "RTP timestamp is the sender time of generating packet"),
            ("SSRC", "RTP SSRC (Synchronization Source Identifier) identifies the source of an RTP stream"),
            ("Alias", "Locally assigned SSRC alias to make differentiating streams more convenient"),
            ("CSRC", "RTP CSRC (Contributing Source Identifier)\nSSRC identifiers of the sources that have contributed to a composite RTP packet"),
            ("Payload Length", "RTP payload length (Excluding header and extensions)"),
        ];

        for (label, desc) in headers {
            header.col(|ui| {
                ui.heading(label).on_hover_text(desc);
            });
        }
    }

    fn build_table_body(&mut self, body: TableBody) {
        let streams = &self.streams.borrow();
        let mut rtp_packets: Vec<_> = streams
            .packets
            .values()
            .filter(|packet| matches!(packet.contents, SessionPacket::Rtp(_)))
            .collect();

        // Apply filtering if filter is set
        rtp_packets.retain(|packet| {
            let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                return false;
            };

            let key = (
                packet.source_addr,
                packet.destination_addr,
                packet.transport_protocol,
                rtp_packet.ssrc,
            );

            let stream_alias = streams
                .rtp_streams
                .get(&key)
                .map(|stream| stream.alias.to_string());

            let ctx = FilterContext {
                packet: rtp_packet,
                source_addr: &packet.source_addr.to_string(),
                destination_addr: &packet.destination_addr.to_string(),
                alias: &stream_alias.unwrap_or_default(),
            };

            self.packet_matches_filter(&ctx)
        });

        if rtp_packets.is_empty() {
            return;
        }

        let mut ssrc_to_display_name = HashMap::new();
        streams.rtp_streams.iter().for_each(|(key, stream)| {
            ssrc_to_display_name.insert(*key, stream.alias.to_string());
        });

        let first_ts = rtp_packets.first().unwrap().timestamp;

        body.rows(self.config.row_height, rtp_packets.len(), |mut row| {
            let row_ix = row.index();
            let packet = rtp_packets.get(row_ix).unwrap();

            let SessionPacket::Rtp(ref rtp_packet) = packet.contents else {
                unreachable!();
            };

            let key = (
                packet.source_addr,
                packet.destination_addr,
                packet.transport_protocol,
                rtp_packet.ssrc,
            );

            // ID column
            row.col(|ui| {
                ui.label(packet.id.to_string());
            });

            // Time column
            row.col(|ui| {
                let timestamp = packet.timestamp - first_ts;
                ui.label(format!("{:.4}", timestamp.as_secs_f64()));
            });

            // Source/Destination columns
            row.col(|ui| {
                ui.label(packet.source_addr.to_string());
            });
            row.col(|ui| {
                ui.label(packet.destination_addr.to_string());
            });

            // RTP-specific columns
            row.col(|ui| {
                ui.label(format_boolean(rtp_packet.padding));
            });
            row.col(|ui| {
                ui.label(format_boolean(rtp_packet.extension));
            });
            row.col(|ui| {
                ui.label(format_boolean(rtp_packet.marker));
            });

            // Payload type column with hover
            let payload_type = &rtp_packet.payload_type;
            let (_, resp) = row.col(|ui| {
                ui.label(payload_type.id.to_string());
            });
            resp.on_hover_text(rtp_packet.payload_type.to_string());

            // Sequence, timestamp, SSRC columns
            row.col(|ui| {
                ui.label(rtp_packet.sequence_number.to_string());
            });
            row.col(|ui| {
                ui.label(rtp_packet.timestamp.to_string());
            });
            row.col(|ui| {
                ui.label(format!("{:x}", rtp_packet.ssrc));
            });
            row.col(|ui| {
                ui.label(ssrc_to_display_name.get(&key).unwrap().to_string());
            });

            // CSRC column
            row.col(|ui| {
                if rtp_packet.csrc.len() <= 1 {
                    if let Some(csrc) = rtp_packet.csrc.first() {
                        ui.label(csrc.to_string());
                    }
                    return;
                }

                let formatted_csrc = rtp_packet
                    .csrc
                    .iter()
                    .map(|num| num.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");

                ui.label(format!("{:?}, ...", rtp_packet.csrc.first().unwrap()))
                    .on_hover_text(formatted_csrc);
            });

            // Payload length column
            row.col(|ui| {
                ui.label(rtp_packet.payload_length.to_string());
            });
        });
    }

    fn packet_matches_filter(&self, ctx: &FilterContext) -> bool {
        if self.filter_input.get_filter().is_empty() {
            return true;
        }

        let filter = self.filter_input.get_filter().trim();

        parse_filter(&filter)
            .map(|filter_type| filter_type.matches(&ctx))
            .unwrap_or(true)
    }
}

fn format_boolean(value: bool) -> RichText {
    if value {
        RichText::from("✔").color(Color32::GREEN)
    } else {
        RichText::from("❌").color(Color32::RED)
    }
}