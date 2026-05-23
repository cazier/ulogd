use rocket::FromForm;
use sea_orm::sea_query::{Asterisk, Expr, Func, SimpleExpr};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};

use super::log::{self, Entity as Log};
use super::models::{Stats, Top};

#[derive(FromForm, Debug, Clone)]
pub struct FilterForm {
    #[field(default = 3600)]
    pub time_range: u32,
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub protocol: Option<u32>,
    pub iiface: Option<String>,
    pub oiface: Option<String>,
    pub prefix: Option<String>,
    #[field(default = 100)]
    pub limit: u16,
    #[field(default = 0)]
    pub offset: u16,
}

enum TopKind {
    SrcIp,
    DstIp,
    DstPort,
}

impl FilterForm {
    fn min_time(&self) -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
            - self.time_range
    }

    fn _base(&self) -> Select<log::Entity> {
        let mut q = Log::find().filter(log::Column::OobTimeSec.gte(self.min_time()));

        if let Some(src_ip) = &self.src_ip {
            q = q.filter(log::Column::SrcIp.eq(src_ip));
        }
        if let Some(dst_ip) = &self.dst_ip {
            q = q.filter(log::Column::DstIp.eq(dst_ip));
        }
        if let Some(src_port) = self.src_port {
            q = q.filter(
                Condition::any()
                    .add(log::Column::TcpSport.eq(src_port))
                    .add(log::Column::UdpSport.eq(src_port)),
            );
        }
        if let Some(dst_port) = self.dst_port {
            q = q.filter(
                Condition::any()
                    .add(log::Column::TcpDport.eq(dst_port))
                    .add(log::Column::UdpDport.eq(dst_port)),
            );
        }
        if let Some(protocol) = self.protocol {
            q = q.filter(log::Column::Proto.eq(protocol as u8));
        }
        if let Some(iiface) = &self.iiface {
            q = q.filter(log::Column::Iiface.eq(iiface));
        }
        if let Some(oiface) = &self.oiface {
            q = q.filter(log::Column::Oiface.eq(oiface));
        }
        if let Some(prefix) = &self.prefix {
            q = q.filter(log::Column::Prefix.eq(prefix));
        }
        q
    }

    pub async fn query(&self, db: &DatabaseConnection) -> Result<Vec<log::Model>, DbErr> {
        self._base()
            .limit(self.limit as u64)
            .offset(self.offset as u64)
            .all(db)
            .await
    }

    fn top(
        &self,
        db: &DatabaseConnection,
        kind: TopKind,
    ) -> impl Future<Output = Result<Vec<Top>, DbErr>> {
        let query = match kind {
            TopKind::SrcIp => self
                ._base()
                .select_only()
                .column_as(log::Column::SrcIp, "key"),
            TopKind::DstIp => self
                ._base()
                .select_only()
                .column_as(log::Column::DstIp, "key"),
            TopKind::DstPort => {
                let _kind: SimpleExpr = Func::coalesce([
                    Expr::col(log::Column::TcpDport).into(),
                    Expr::col(log::Column::UdpDport).into(),
                ])
                .into();

                self._base()
                    .select_only()
                    .column_as(_kind.cast_as("text"), "key")
                    .column_as(log::Column::Proto, "proto")
                    .filter(
                        Condition::any()
                            .add(log::Column::TcpDport.is_not_null())
                            .add(log::Column::UdpDport.is_not_null()),
                    )
            }
        };

        query
            .expr_as(Expr::col(Asterisk).count(), "count")
            .column_as(log::Column::Length.sum(), "bytes")
            .limit(10)
            .group_by(Expr::col("key"))
            .order_by_desc(Expr::col("bytes"))
            .into_model::<Top>()
            .all(db)
    }

    pub async fn stats(&self, db: &DatabaseConnection) -> Result<Stats, DbErr> {
        Ok(Stats {
            src_ips: self.top(db, TopKind::SrcIp).await?,
            dst_ips: self.top(db, TopKind::DstIp).await?,
            dst_ports: self.top(db, TopKind::DstPort).await?,
            ..Stats::default()
        })
    }
}
