use sea_orm::FromQueryResult;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub enum Action {
    Allowed,
    Blocked,
}

#[derive(Serialize, Default, Clone, FromQueryResult)]
pub struct TimelinePoint {
    time: u32,
    packets: u32,
}

#[derive(Serialize, Default, Clone, macros::New)]
pub struct Interface {
    packets_in: u32,
    packets_out: u32,
    bytes_in: u32,
    bytes_out: u32,
}

#[derive(Serialize, Default, Clone, macros::New)]
pub struct Stats {
    timeline: Vec<TimelinePoint>,
    protocols: std::collections::BTreeMap<String, u32>,
    actions: std::collections::BTreeMap<String, u32>,
    interfaces: std::collections::BTreeMap<String, Interface>,
}

#[derive(Serialize, Default, Debug, Clone, FromQueryResult)]
pub struct Top {
    key: String,
    proto: Option<u8>,
    count: u32,
    bytes: u32,
}

#[derive(Serialize, Default, Clone, macros::New)]
pub struct Totals {
    packets: u32,
    bytes: u32,
    src: u32,
    dst: u32,
}

#[derive(Serialize, Clone, Default, macros::New)]
pub struct Summary {
    src_ips: Vec<Top>,
    dst_ips: Vec<Top>,
    dst_ports: Vec<Top>,
    totals: Totals,
}

#[derive(Serialize, Clone, Default, macros::New)]
pub struct Options {
    iifaces: Vec<String>,
    oifaces: Vec<String>,
    protocols: Vec<String>,
}

impl From<String> for Action {
    fn from(value: String) -> Self {
        if value.ends_with("drop") {
            return Action::Blocked;
        }
        return Action::Allowed;
    }
}

impl Interface {
    pub fn update_inputs(&mut self, inputs: (u32, u32)) {
        self.packets_in += inputs.0;
        self.bytes_in += inputs.1;
    }

    pub fn update_outputs(&mut self, outputs: (u32, u32)) {
        self.packets_out += outputs.0;
        self.bytes_out += outputs.1;
    }
}
