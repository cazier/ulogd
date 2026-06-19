use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Default, Clone, FromQueryResult, ToSchema)]
pub struct Bucket {
    /// Beginning timestamp of the bucket
    time: u32,
    /// Packets in that bucket
    packets: u32,
}

#[derive(Serialize, Default, Clone, macros::New, ToSchema)]
pub struct Interface {
    /// Packets received on the interface
    packets_in: u32,
    /// Packets transmitted out of the interface
    packets_out: u32,
    /// Bytes received on the interface
    bytes_in: u32,
    /// Bytes transmitted out of the interface
    bytes_out: u32,
}

#[derive(Serialize, Default, Clone, macros::New, ToSchema)]
pub struct Stats {
    /// Array of the packets seen in each time-spanned bucket
    buckets: Vec<Bucket>,
    /// Mapping of the packets transmitted of each protocol
    protocols: std::collections::BTreeMap<String, u32>,
    /// Mapping of the packets transmitted with each action performed
    actions: std::collections::BTreeMap<String, u32>,
    /// Mapping of the packets transmitted on each interface
    interfaces: std::collections::BTreeMap<String, Interface>,
}

#[derive(Serialize, Default, Debug, Clone, FromQueryResult, ToSchema)]
pub struct Top {
    /// Identifier for the top statistic (i.e., a source or destination IP address)
    key: String,
    /// Traffic protocol integer value
    proto: Option<u8>,
    /// Packets transmitted
    count: u32,
    /// Bytes transmitted
    bytes: u32,
}

#[derive(Serialize, Default, Clone, macros::New, ToSchema)]
pub struct Totals {
    /// Total packets transmitted
    packets: u32,
    /// Total bytes transmitted
    bytes: u32,
    /// Total source IP addresses seen
    src: u32,
    /// Total destination IP addresses seen
    dst: u32,
}

#[derive(Serialize, Clone, Default, macros::New, ToSchema)]
pub struct Summary {
    /// Source IPv4 or IPv6 addresses
    src_ips: Vec<Top>,
    /// Destination IPv4 or IPv6 addresses
    dst_ips: Vec<Top>,
    /// Destination ports
    dst_ports: Vec<Top>,
    /// Total traffic stats
    totals: Totals,
}

#[derive(Serialize, Clone, Default, macros::New, ToSchema)]
pub struct Options {
    /// Input interfaces
    iifaces: Vec<String>,
    /// Output interfaces
    oifaces: Vec<String>,
    /// Protocols
    protocols: Vec<String>,
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
