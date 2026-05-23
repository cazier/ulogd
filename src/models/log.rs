use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub oob_time_sec: u32,
    pub oob_time_usec: u32,
    #[sea_orm(column_name = "oob_prefix")]
    pub prefix: String,
    #[sea_orm(column_name = "oob_in")]
    pub iiface: String,
    #[sea_orm(column_name = "oob_out")]
    pub oiface: String,
    #[sea_orm(column_name = "ip_saddr_str")]
    pub src_ip: String,
    #[sea_orm(column_name = "ip_daddr_str")]
    pub dst_ip: String,
    #[sea_orm(column_name = "ip_protocol")]
    pub proto: Option<u8>,
    #[sea_orm(column_name = "ip_ttl")]
    pub ttl: Option<u8>,
    pub tcp_sport: Option<u16>,
    pub tcp_dport: Option<u16>,
    pub udp_sport: Option<u16>,
    pub udp_dport: Option<u16>,
    pub icmp_type: Option<u8>,
    pub icmp_code: Option<u8>,
    #[sea_orm(column_name = "raw_pktlen")]
    pub length: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
