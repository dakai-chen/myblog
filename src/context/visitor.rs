use std::sync::Arc;

use boluo::data::Extension;
use boluo::extract::FromRequest;
use boluo::http::header::SET_COOKIE;
use boluo::request::Request;
use boluo::response::{IntoResponseParts, ResponseParts};
use boluo_extra::cookie::{Cookie, CookieJar, SameSite};
use time::OffsetDateTime;

use crate::error::{AppError, AppErrorMeta};
use crate::util::time::UnixTimestampSecs;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisitorId {
    visitor_id: Arc<str>,
}

impl VisitorId {
    const COOKIE_KEY: &str = "x-visitor-id";

    pub fn visitor_id(&self) -> &str {
        &self.visitor_id
    }

    async fn from_cookies(jar: &CookieJar) -> anyhow::Result<Self> {
        let visitor_id = match jar.get(Self::COOKIE_KEY).map(|cookie| cookie.value()) {
            Some(visitor_id) => crate::service::visitor::keep_or_create(visitor_id).await?,
            None => crate::service::visitor::create().await?,
        };
        Ok(VisitorId {
            visitor_id: Arc::from(visitor_id),
        })
    }

    fn build_cookie(&self) -> anyhow::Result<Cookie<'_>> {
        let mut cookie = Cookie::new(Self::COOKIE_KEY, self.visitor_id());
        cookie.set_expires(Self::cookie_expires_at()?);
        cookie.set_http_only(true);
        cookie.set_path("/");
        cookie.set_same_site(SameSite::Lax);
        if crate::config::get().security.cookie_secure {
            cookie.set_secure(true);
        }
        Ok(cookie)
    }

    fn cookie_expires_at() -> anyhow::Result<OffsetDateTime> {
        Ok(OffsetDateTime::from_unix_timestamp(
            UnixTimestampSecs::now()
                .saturating_add(crate::config::get().visitor.session_ttl)
                .as_i64(),
        )?)
    }
}

impl FromRequest for VisitorId {
    type Error = AppError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        if let Some(visitor) = Option::<Extension<Self>>::from_request(request).await? {
            return Ok(Extension::into_inner(visitor));
        }
        let jar = <CookieJar as FromRequest>::from_request(request)
            .await
            .map_err(|e| {
                AppErrorMeta::BadRequest
                    .with_message("Cookie 解析错误")
                    .with_source(e)
            })?;
        let visitor = VisitorId::from_cookies(&jar).await?;
        request.extensions_mut().insert(visitor.clone());
        Ok(visitor)
    }
}

impl IntoResponseParts for VisitorId {
    type Error = AppError;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        if let Ok(value) = self.build_cookie()?.encoded().to_string().parse() {
            parts.headers.append(SET_COOKIE, value);
        }
        Ok(parts)
    }
}
