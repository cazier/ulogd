use std::sync::atomic::{AtomicI64, Ordering};

use rocket::FromForm;
use sea_orm::{
    ColumnTrait, DatabaseConnection, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{Asterisk, Expr, Func, SimpleExpr},
};
use utoipa::IntoParams;

use super::{
    models::{Bucket, Options, Stats, Summary, Top, Totals},
    tables::{HasTimestamp, filters, log},
};
use crate::models::models::Interface;

#[derive(FromForm, Debug, Clone, IntoParams)]
pub struct FilterForm {
    #[field(default = 3600)]
    /// Seconds in the past to query
    pub time_range: u32,
    /// Source IPv4 or IPv6 address
    pub src_ip: Option<String>,
    /// Destination IPv4 or IPv6 address
    pub dst_ip: Option<String>,
    /// Source Port
    pub src_port: Option<u16>,
    /// Destination Port
    pub dst_port: Option<u16>,
    /// Layer 4 Protocol
    pub protocol: Option<u32>,
    /// Input interface name
    pub iiface: Option<String>,
    /// Output interface name
    pub oiface: Option<String>,
    /// Whether traffic was allowed through (was not blocked/dropped)
    pub allowed: Option<bool>,
    #[field(default = 100)]
    /// Number of results to return
    pub limit: u16,
    #[field(default = 0)]
    /// Offset to start for results
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
        if let Some(allowed) = &self.allowed {
            let prefix = match allowed {
                true => "allow",
                false => "drop",
            };
            q = q.filter(log::Column::Prefix.contains(prefix));
        }
        return q;
    }

    pub async fn live(
        &self,
        db: &DatabaseConnection,
        cursor: &AtomicI64,
    ) -> Result<Vec<log::Model>, sea_orm::DbErr> {
        let cursor_id = cursor.load(Ordering::Relaxed);

        let mut query = self
            ._base()
            .filter(log::Column::Rowid.gt(cursor_id))
            .order_by_asc(log::Column::Rowid);

        if cursor_id != 0 {
            query = query.limit(self.limit as u64).offset(self.offset as u64);
        };

        query.all(db).await.inspect(|result| {
            if let Some(last) = result.last() {
                cursor.store(last.rowid, Ordering::Relaxed);
            }
        })
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

    async fn generate_distributions<V>(
        &self,
        db: &DatabaseConnection,
        column: log::Column,
    ) -> Result<std::collections::BTreeMap<String, u32>, sea_orm::DbErr>
    where
        (V, i64): sea_orm::TryGetableMany,
        V: std::fmt::Display,
    {
        let rows: Vec<(V, i64)> = self
            ._base()
            .select_only()
            .column(column)
            .expr_as(Expr::col(Asterisk).count(), "count")
            .filter(column.is_not_null())
            .group_by(column)
            .into_tuple()
            .all(db)
            .await?;

        Ok(rows
            .into_iter()
            .map(|(val, count)| (format!("{val}"), count as u32))
            .collect())
    }

    async fn generate_timeline(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Vec<Bucket>, sea_orm::DbErr> {
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
            .into_model::<Bucket>()
            .all(db)
            .await?;

        Ok(timeline)
    }

    async fn generate_interfaces(
        &self,
        db: &DatabaseConnection,
    ) -> Result<std::collections::BTreeMap<String, Interface>, sea_orm::DbErr> {
        let rows: Vec<(Option<String>, Option<String>, i64, i64)> = self
            ._base()
            .select_only()
            .column(log::Column::Iiface)
            .column(log::Column::Oiface)
            .expr_as(Expr::col(Asterisk).count(), "packets")
            .column_as(log::Column::Length.sum(), "bytes")
            .group_by(log::Column::Iiface)
            .group_by(log::Column::Oiface)
            .into_tuple()
            .all(db)
            .await?;

        let mut map: std::collections::BTreeMap<String, Interface> =
            std::collections::BTreeMap::new();

        for (iiface, oiface, packets, bytes) in rows {
            if let Some(name) = iiface
                && name != ""
            {
                let e = map.entry(name).or_default();
                e.update_inputs((packets as u32, bytes as u32));
            }
            if let Some(name) = oiface
                && name != ""
            {
                let e = map.entry(name).or_default();
                e.update_outputs((packets as u32, bytes as u32));
            }
        }

        Ok(map)
    }

    pub async fn stats(&self, db: &DatabaseConnection) -> Result<Stats, sea_orm::DbErr> {
        Ok(Stats::new(
            self.generate_timeline(db).await?,
            self.generate_distributions::<u8>(db, log::Column::Proto)
                .await?,
            self.generate_distributions::<String>(db, log::Column::Prefix)
                .await?,
            self.generate_interfaces(db).await?,
        ))
    }

    pub async fn summary(&self, db: &DatabaseConnection) -> Result<Summary, sea_orm::DbErr> {
        let (packets, bytes) = self.count(db).await?;
        Ok(Summary::new(
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

#[derive(FromForm, Debug, Clone, IntoParams)]
pub struct OptionsForm {
    #[field(default = 3600)]
    /// Seconds in the past to query
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
                filters::Kind::Protocol => protocols.push(row.value),
            }
        }

        Ok(Options::new(iifaces, oifaces, protocols))
    }
}
