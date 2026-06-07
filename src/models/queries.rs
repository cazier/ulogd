use super::{
    models::{Options, Stats, TimelinePoint, Top, Totals},
    tables::{HasTimestamp, filters, log},
};
use crate::utils::get_protocol_from_number;
use rocket::FromForm;
use sea_orm::{
    ColumnTrait, DatabaseConnection, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{Asterisk, Expr, Func, SimpleExpr},
};

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

#[cfg(not(debug_assertions))]
fn from_time<E: HasTimestamp>(param: u32) -> sea_orm::Select<E> {
    E::find().filter(
        E::timestamp_column().gte(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32
                - param,
        ),
    )
}

#[cfg(debug_assertions)]
fn from_time<E: HasTimestamp>(_: u32) -> sea_orm::Select<E> {
    E::find().filter(E::timestamp_column().gte(0u32))
}

impl FilterForm {
    fn _base(&self) -> sea_orm::Select<log::Entity> {
        let mut q = from_time(self.time_range);

        if let Some(src_ip) = &self.src_ip {
            q = q.filter(log::Column::SrcIp.eq(src_ip));
        }
        if let Some(dst_ip) = &self.dst_ip {
            q = q.filter(log::Column::DstIp.eq(dst_ip));
        }
        if let Some(src_port) = self.src_port {
            q = q.filter(
                sea_orm::Condition::any()
                    .add(log::Column::TcpSport.eq(src_port))
                    .add(log::Column::UdpSport.eq(src_port)),
            );
        }
        if let Some(dst_port) = self.dst_port {
            q = q.filter(
                sea_orm::Condition::any()
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

    pub async fn query(&self, db: &DatabaseConnection) -> Result<Vec<log::Model>, sea_orm::DbErr> {
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
    ) -> impl Future<Output = Result<Vec<Top>, sea_orm::DbErr>> {
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
                        sea_orm::Condition::any()
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

    async fn distinct(
        &self,
        db: &DatabaseConnection,
        kind: TopKind,
    ) -> Result<u32, sea_orm::DbErr> {
        let count = self
            ._base()
            .select_only()
            .column(match kind {
                TopKind::SrcIp => log::Column::SrcIp,
                TopKind::DstIp => log::Column::DstIp,
                _ => panic!("Ahhhh"),
            })
            .distinct()
            .count(db)
            .await?;
        return Ok(count as u32);
    }

    async fn count(&self, db: &DatabaseConnection) -> Result<(u32, u32), sea_orm::DbErr> {
        let row: Option<(i64, i64)> = self
            ._base()
            .select_only()
            .expr_as(Expr::col(Asterisk).count(), "count")
            .column_as(log::Column::Length.sum(), "bytes")
            .into_tuple()
            .one(db)
            .await?;

        let (packets, bytes) = row.unwrap();
        Ok((packets as u32, bytes as u32))
    }

    async fn generate_timeline(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Vec<TimelinePoint>, sea_orm::DbErr> {
        // time
        // packets

        let bucket = match self.time_range {
            0..=300 => 10,
            301..=900 => 30,
            901..=3600 => 60,
            3601..=21600 => 300,
            _ => 900,
        };

        let timeline = self
            ._base()
            .select_only()
            .expr_as(Expr::col(Asterisk).count(), "packets")
            .expr_as(
                Expr::cust(format!("(oob_time_sec / {bucket}) * {bucket}")),
                "time",
            )
            .group_by(Expr::cust("time"))
            .order_by_asc(Expr::cust("time"))
            .into_model::<TimelinePoint>()
            .all(db)
            .await?;

        Ok(timeline)
    }

    pub async fn stats(&self, db: &DatabaseConnection) -> Result<Stats, sea_orm::DbErr> {
        let (packets, bytes) = self.count(db).await?;
        Ok(Stats::new(
            self.generate_timeline(db).await?,
            self.top(db, TopKind::SrcIp).await?,
            self.top(db, TopKind::DstIp).await?,
            self.top(db, TopKind::DstPort).await?,
            Totals::new(
                packets,
                bytes,
                self.distinct(db, TopKind::SrcIp).await?,
                self.distinct(db, TopKind::DstIp).await?,
            ),
        ))
    }
}

#[derive(FromForm, Debug, Clone)]
pub struct OptionsForm {
    #[field(default = 3600)]
    pub time_range: u32,
}

impl OptionsForm {
    pub async fn query(&self, db: &DatabaseConnection) -> Result<Options, sea_orm::DbErr> {
        let rows = from_time::<filters::Entity>(self.time_range)
            .all(db)
            .await?;

        let (mut iifaces, mut oifaces, mut protocols) = (vec![], vec![], vec![]);

        for row in rows {
            match row.kind {
                filters::Kind::InputInterface => iifaces.push(row.value),
                filters::Kind::OutputInterface => oifaces.push(row.value),
                filters::Kind::Protocol => {
                    let name = row
                        .value
                        .parse::<u8>()
                        .ok()
                        .and_then(get_protocol_from_number)
                        .unwrap_or(row.value);
                    protocols.push(name);
                }
            }
        }

        Ok(Options::new(iifaces, oifaces, protocols))
    }
}
