use std::net::IpAddr;

use crate::error::AppError;
use crate::model::co::failed_attempts::{FailedAttemptsBanCo, FailedAttemptsBanCoIdGen};
use crate::model::po::failed_attempts::FailedAttemptsPo;
use crate::storage::cache::{Cache, CacheIdGenerator, CacheSetMode};
use crate::storage::db::DbConn;
use crate::util::time::UnixTimestampSecs;

#[derive(Debug)]
pub struct FailedAttemptsBanBo;

impl FailedAttemptsBanBo {
    pub const SCENE_ARTICLE_UNLOCK: &'static str = "article_unlock";
    pub const SCENE_LOGIN: &'static str = "login";

    pub async fn ban(scene: &str, ip: IpAddr, target_id: &str) -> Result<(), AppError> {
        let id = FailedAttemptsBanCoIdGen {
            scene,
            ip,
            target_id,
        };
        let co = Cache::builder(FailedAttemptsBanCo)
            .id(&id)
            .ttl(crate::config::get().article.unlock_ban_ttl)
            .build()?;
        co.set(CacheSetMode::Overwrite).await?;
        Ok(())
    }

    pub async fn is_banned(scene: &str, ip: IpAddr, target_id: &str) -> Result<bool, AppError> {
        let id = FailedAttemptsBanCoIdGen {
            scene,
            ip,
            target_id,
        };
        Cache::<FailedAttemptsBanCo>::exists(id.generate_id().as_ref())
            .await
            .map_err(From::from)
    }

    pub async fn record_failed(
        scene: impl Into<String>,
        ip: IpAddr,
        target_id: impl Into<String>,
        db: &mut DbConn,
    ) -> Result<u32, AppError> {
        let now = UnixTimestampSecs::now();
        let po = FailedAttemptsPo {
            scene: scene.into(),
            ip: ip.to_string(),
            target_id: target_id.into(),
            count: 1,
            created_at: now.as_i64(),
            expires_at: now
                .saturating_add(crate::config::get().article.unlock_try_window)
                .as_i64(),
        };

        crate::storage::db::failed_attempts::remove_single_expired(
            &po.scene,
            &po.ip,
            &po.target_id,
            db,
        )
        .await?;

        crate::storage::db::failed_attempts::incr_count(&po, db)
            .await
            .map_err(From::from)
    }

    pub async fn record_failed_with_ban(
        scene: impl Into<String>,
        ip: IpAddr,
        target_id: impl Into<String>,
        db: &mut DbConn,
    ) -> Result<(u32, bool), AppError> {
        let scene = scene.into();
        let target_id = target_id.into();

        let count = Self::record_failed(&scene, ip, &target_id, db).await?;

        // 计算剩余次数
        let remaining_times = crate::config::get()
            .article
            .unlock_try_max_times
            .saturating_sub(count);

        if remaining_times == 0 {
            FailedAttemptsBanBo::ban(&scene, ip, &target_id).await?;
            Ok((remaining_times, true))
        } else {
            Ok((remaining_times, false))
        }
    }
}
