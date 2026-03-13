use std::time::Duration;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::util::time::UnixTimestampSecs;

pub fn encode<T>(claims: &T, secret: &[u8]) -> anyhow::Result<String>
where
    T: Serialize,
{
    jsonwebtoken::encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(Into::into)
}

pub fn decode<T>(token: &str, secret: &[u8]) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    jsonwebtoken::decode(token, &DecodingKey::from_secret(secret), &validation())
        .map(|data| data.claims)
        .map_err(Into::into)
}

fn validation() -> Validation {
    let mut validation = Validation::default();
    validation.validate_exp = false;
    validation
}

pub trait JwtClaimsData: DeserializeOwned + Serialize {
    fn kind() -> &'static str;

    fn with_ttl(self, ttl: Duration) -> JwtClaims<Self> {
        JwtClaims::with_ttl(self, ttl)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtClaims<T> {
    /// 签发时间
    pub iat: i64,
    /// 过期时间
    pub exp: i64,
    /// 令牌类型
    pub kind: String,
    /// 令牌数据
    pub data: T,
}

impl<T> JwtClaims<T>
where
    T: JwtClaimsData,
{
    pub fn new(data: T, iat: i64, exp: i64) -> Self {
        JwtClaims {
            iat,
            exp,
            kind: T::kind().to_owned(),
            data,
        }
    }

    pub fn with_ttl(data: T, ttl: Duration) -> Self {
        let now = UnixTimestampSecs::now();
        Self::new(data, now.as_i64(), now.saturating_add(ttl).as_i64())
    }

    pub fn encode(&self, secret: &[u8]) -> anyhow::Result<String> {
        self::encode(&self, secret)
    }

    pub fn decode(token: &str, secret: &[u8]) -> anyhow::Result<JwtClaims<T>> {
        let claims = self::decode::<JwtClaims<T>>(token, secret)?;
        if claims.kind != T::kind() {
            return Err(anyhow::anyhow!(
                "令牌类型不匹配：期望 `{}`, 实际 `{}`",
                T::kind(),
                claims.kind
            ));
        }
        Ok(claims)
    }

    pub fn is_expired(&self) -> bool {
        UnixTimestampSecs::now().as_i64() > self.exp
    }

    pub fn is_effective(&self) -> bool {
        UnixTimestampSecs::now().as_i64() >= self.iat
    }
}
