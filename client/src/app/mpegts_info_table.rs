use super::is_mpegts_stream_visible;
use crate::streams::RefStreams;
use egui_extras::{Column, TableBody, TableBuilder};
use ewebsock::WsSender;
use netpix_common::mpegts::descriptors::Descriptors;
use netpix_common::mpegts::header::PIDTable;
use netpix_common::mpegts::psi::pat::pat_buffer::PatBuffer;
use netpix_common::mpegts::psi::pat::ProgramAssociationTable;
use netpix_common::mpegts::psi::pmt::pmt_buffer::PmtBuffer;
use netpix_common::mpegts::psi::pmt::ProgramMapTable;
use netpix_common::mpegts::psi::psi_buffer::PsiBuffer;
use netpix_common::MpegtsStreamKey;
use std::cmp::{max, Ordering};
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::time::Duration;

const LINE_HEIGHT: f32 = 20.0;

struct MpegTsInfo {
    pat: Option<ProgramAssociationTable>,
    pmt: Option<ProgramMapTable>,
}

#[derive(Hash, Eq, PartialEq, Ord, Clone)]
struct RowKey {
    pid: PIDTable,
    alias: String,
}

impl PartialOrd for RowKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if !self.alias.eq(&other.alias) {
            return self.alias.partial_cmp(&other.alias);
        }
        self.pid.partial_cmp(&other.pid)
    }
}

pub struct MpegTsInformationTable {
    streams: RefStreams,
    ws_sender: WsSender,
    streams_visibility: HashMap<MpegtsStreamKey, bool>,
}

impl MpegTsInformationTable {
    pub fn new(streams: RefStreams, ws_sender: WsSender) -> Self {
        Self {
            streams,
            ws_sender,
            streams_visibility: HashMap::default(),
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.options_ui(ui);
            self.build_table(ui);
        });
    }
    fn options_ui(&mut self, ui: &mut egui::Ui) {
        let mut aliases = Vec::new();
        let streams = &self.streams.borrow().mpeg_ts_streams;
        let keys: Vec<_> = streams.keys().collect();

        keys.iter().for_each(|&key| {
            let alias = streams.get(key).unwrap().alias.to_string();
            aliases.push((*key, alias));
        });
        aliases.sort_by(|(_, a), (_, b)| a.cmp(b));

        ui.horizontal_wrapped(|ui| {
            ui.label("Filter by: ");
            aliases.iter().for_each(|(key, alias)| {
                let selected = is_mpegts_stream_visible(&mut self.streams_visibility, *key);
                ui.checkbox(selected, alias);
            });
        });
        ui.vertical(|ui| {
            ui.add_space(5.0);
        });
    }

    fn build_table(&mut self, ui: &mut egui::Ui) {
        let header_labels = [
            ("Stream alias", "Stream alias"),
            ("Type", "Type of mpegts packet"),
            ("Packet count", "Number of packets in mpegts packet"),
            ("Addition information", "Additional information"),
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .stick_to_bottom(true)
            .columns(Column::remainder().at_least(100.0).at_most(200.0), 3)
            .column(Column::remainder().at_least(1000.0))
            .header(30.0, |mut header| {
                header_labels.iter().for_each(|(label, desc)| {
                    header.col(|ui| {
                        ui.heading(label.to_string())
                            .on_hover_text(desc.to_string());
                    });
                });
            })
            .body(|body| {
                self.build_table_body(body);
            });
    }

    fn build_pmt_table(&mut self, ui: &mut egui::Ui, pmt: &ProgramMapTable) {
        let header_labels = [
            "Stream type",
            "Elementary PID",
            "Descriptors",
        ];
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .columns(Column::remainder().at_least(100.0), 3)
            .header(20.0, |mut header| {
                header_labels.iter().for_each(|label| {
                    header.col(|ui| {
                        ui.heading(label.to_string());
                    });
                });
            })
            .body(|body| {
                self.build_pmt_table_body(body, pmt);
            });
    }

    fn build_table_body(&mut self, body: TableBody) {
        let streams = &self.streams.borrow();
        let mut mpegts_rows: BTreeMap<RowKey, MpegTsInfo> = BTreeMap::default();
        let mut row_height: BTreeMap<RowKey, f32> = BTreeMap::default();
        streams.mpeg_ts_streams.iter().for_each(|(_key, stream)| {
            if let Some(pat) = &stream.stream_info.pat {
                let info = MpegTsInfo {
                    pat: Some(pat.clone()),
                    pmt: None,
                };
                let key = RowKey {
                    pid: PIDTable::ProgramAssociation,
                    alias: stream.alias.clone(),
                };
                mpegts_rows.insert(key.clone(), info);
                row_height.insert(key, pat.programs.len() as f32 * LINE_HEIGHT);
            }
            stream.stream_info.pmt.iter().for_each(|(key, pmt)| {
                let info = MpegTsInfo {
                    pat: None,
                    pmt: Some(pmt.clone()),
                };
                let key = RowKey {
                    pid: key.clone(),
                    alias: stream.alias.clone(),
                };
                mpegts_rows.insert(key.clone(), info);
                row_height.insert(key, (pmt.elementary_streams_info.len() + 1) as f32 * LINE_HEIGHT);
            });
        });

        let keys = mpegts_rows.keys().collect::<Vec<_>>();
        body.heterogeneous_rows(
            row_height.values().map(|height| *height).into_iter(),
            |mut row| {
                let key = keys.get(row.index()).unwrap();
                let info = mpegts_rows.get(key).unwrap();

                row.col(|ui| {
                    let mut binding = key.alias.clone();
                    let text_edit = egui::TextEdit::singleline(&mut binding).frame(false);
                    ui.add(text_edit);
                });
                row.col(|ui| {
                    let label = match key.pid {
                        PIDTable::ProgramAssociation => key.pid.to_string(),
                        PIDTable::PID(pid) => format!("Program map ({})", pid),
                        _ => String::default(),
                    };
                    ui.label(label);
                });
                row.col(|ui| {
                    if let Some(pat) = &info.pat {
                        ui.label(pat.fragment_count.to_string());
                    } else if let Some(pmt) = &info.pmt {
                        ui.label(pmt.fragment_count.to_string());
                    }
                });
                row.col(|ui| {
                    if let Some(pat) = &info.pat {
                        let mut programs = String::new();
                        pat.programs.iter().for_each(|program| {
                            programs +=
                                format!("Program number: {} ", program.program_number).as_str();
                            if let Some(network_pid) = program.network_pid {
                                programs += format!("Network PID: {}\n", network_pid).as_str();
                            } else if let Some(program_map_pid) = program.program_map_pid {
                                programs +=
                                    format!("Program map PID: {}\n", program_map_pid).as_str();
                            }
                        });
                        ui.label(programs);
                    } else if let Some(pmt) = &info.pmt {
                        if pmt.elementary_streams_info.len() > 0 {
                            let mut streams_info = String::default();
                            pmt.elementary_streams_info.iter().for_each(|stream_info| {
                                streams_info +=
                                    format!("Stream type: {}, ", stream_info.stream_type).as_str();
                                streams_info +=
                                    format!("Elementary PID: {}, ", stream_info.elementary_pid)
                                        .as_str();
                                if stream_info.descriptors.len() > 0 {
                                    // streams_info += "Descriptors: ";
                                    // let descriptors = stream_info
                                    //     .clone()
                                    //     .descriptors
                                    //     .into_iter()
                                    //     .filter(|desc| !desc.eq(&Descriptors::Unknown))
                                    //     .map(|desc| desc.to_string())
                                    //     .collect::<Vec<_>>();
                                    // streams_info += descriptors.join(", ").as_str();
                                }
                                streams_info += "\n";
                            });
                            ui.label(streams_info);
                        }
                    }
                });
            },
        )
    }

    fn build_pmt_table_body(&mut self, body: TableBody, pmt: &ProgramMapTable) {
        body.rows(20.0, pmt.elementary_streams_info.len(), |mut row| {
            let stream_info = pmt.elementary_streams_info.get(row.index()).unwrap();
            let has_descriptors = stream_info.descriptors.len() > 0;
            row.col(|ui| {
                ui.label(stream_info.stream_type.to_string());
            });
            row.col(|ui| {
                ui.label(stream_info.elementary_pid.to_string());
            });
            row.col(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let button = egui::Button::new("i Descriptors");
                });
            });
        })
    }
}
