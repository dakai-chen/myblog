pub mod generator;

use std::collections::HashMap;
use std::time::Duration;
use std::{net::IpAddr, sync::OnceLock};

use serde::{Deserialize, Deserializer, Serialize};

use crate::util::path::PathJoin;

/// 全局应用程序配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// 安全配置
    pub security: SecurityConfig,
    /// HTTP 服务配置
    pub http: HttpConfig,
    /// 日志配置
    pub logger: LoggerConfig,
    /// 请求体大小限制配置
    pub body_limit: BodyLimitConfig,
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 资源配置
    pub resource: ResourceConfig,
    /// 管理员配置
    pub admin: AdminConfig,
    /// JWT 配置
    pub jwt: JwtConfig,
    /// 主题配置
    pub theme: ThemeConfig,
    /// 定时任务配置
    pub cron: CronConfig,
    /// 文章配置
    pub article: ArticleConfig,
    /// 游客配置
    pub visitor: VisitorConfig,
}

impl AppConfig {
    pub fn load(mode: Option<&str>) -> anyhow::Result<Self> {
        if let Some(mode) = mode {
            Self::from_mode(mode)
        } else {
            Self::from_default()
        }
    }

    fn from_default() -> anyhow::Result<Self> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name("config/default.toml").required(true))
            .add_source(config::File::with_name("config/auth.toml"))
            .add_source(config::Environment::default().prefix("APP").separator("."))
            .build()?
            .try_deserialize()?)
    }

    fn from_mode(mode: &str) -> anyhow::Result<Self> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name("config/default.toml").required(true))
            .add_source(config::File::with_name("config/user/auth.toml"))
            .add_source(config::File::with_name(&format!("config/user/{mode}.toml")).required(true))
            .add_source(config::Environment::default().prefix("APP").separator("."))
            .build()?
            .try_deserialize()?)
    }
}

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub fn init(mode: Option<&str>) -> anyhow::Result<()> {
    generator::generate_auth_config("config/user/auth.toml")?;
    APP_CONFIG
        .set(AppConfig::load(mode)?)
        .map_err(|_| anyhow::anyhow!("重复初始化应用程序配置"))
}

pub fn get() -> &'static AppConfig {
    APP_CONFIG.get().expect("应用程序配置未初始化")
}

/// 安全配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    /// 是否启用 Cookie 的 Secure 属性
    pub cookie_secure: bool,
}

/// HTTP 服务配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpConfig {
    /// 服务绑定的 IP 地址
    pub bind_ip: IpAddr,
    /// 服务绑定的端口号
    pub bind_port: u16,
    /// 优雅关机的超时时间
    #[serde(default, with = "humantime_serde")]
    pub shutdown_timeout: Option<Duration>,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggerConfig {
    /// 日志记录级别
    pub level: String,
    /// 启用日志文件输出
    pub enable_file_output: bool,
    /// 日志文件存储目录
    pub file_dir: String,
    /// 日志文件名前缀
    pub file_prefix: String,
    /// 日志文件最大保留数量
    pub max_keep_files: usize,
}

/// 自定义请求体大小限制规则
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BodyLimitRule {
    /// 请求路径
    pub path: String,
    /// 请求方法
    pub method: Option<String>,
    /// 请求体大小限制值
    #[serde(default, with = "crate::util::serde::human_size")]
    pub limit: Option<u64>,
}

/// 请求体大小限制配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BodyLimitConfig {
    /// 默认请求体大小限制
    #[serde(default, with = "crate::util::serde::human_size")]
    pub default_limit: Option<u64>,
    /// 自定义请求体大小限制规则列表
    pub rules: Vec<BodyLimitRule>,
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// 连接字符串
    pub url: String,
    /// 数据库迁移配置
    pub migrations: DatabaseMigrationsConfig,
    /// 数据库日志配置
    pub log: DatabaseLogConfig,
    /// 数据库连接池配置
    pub pool: DatabasePoolConfig,
    /// SQLite 配置
    pub sqlite: DatabaseSqliteConfig,
}

/// SQLite 配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseSqliteConfig {
    /// 扩展文件目录
    pub extensions_dir: String,
}

/// 数据库迁移配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseMigrationsConfig {
    /// 是否自动运行迁移
    pub auto_migrate: bool,
    /// 迁移脚本的扩展名
    pub script_extension: String,
    /// 迁移脚本存放目录
    pub script_dir: String,
}

/// 数据库日志配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseLogConfig {
    /// 获取连接耗时过长时使用的日志级别
    #[serde(default)]
    pub acquire_slow_level: Option<String>,
    /// 当获取连接的耗时超过此阈值时，将使用 acquire_slow_level 对应的日志级别记录
    #[serde(default, with = "humantime_serde")]
    pub acquire_slow_threshold: Option<Duration>,
}

/// 数据库连接池配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabasePoolConfig {
    /// 最小连接数量
    pub min_connections: u32,
    /// 最大连接数量
    pub max_connections: u32,
    /// 连接获取超时时间
    #[serde(with = "humantime_serde")]
    pub acquire_timeout: Duration,
    /// 连接空闲超时时间
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,
    /// 连接最大保持时间
    #[serde(with = "humantime_serde")]
    pub max_lifetime: Duration,
}

/// 资源文件配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceConfig {
    /// 上传文件的存储目录
    pub upload_dir: String,
    /// 上传文件的大小限制
    #[serde(with = "crate::util::serde::human_size")]
    pub upload_file_max_size: u64,
    /// 回收站目录
    pub trash_dir: String,
    /// 孤立文件移入回收站的时间阈值
    #[serde(with = "humantime_serde")]
    pub trash_threshold: Duration,
    /// 公开文件的存储目录
    pub public_dir: String,
}

/// 管理员配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdminConfig {
    /// 登录密码
    pub password: String,
    /// TOTP 工具导入链接（OTPAuth URL格式，用于生成二维码供扫码绑定）
    pub totp_url: String,
    /// 会话有效期
    #[serde(with = "humantime_serde")]
    pub session_ttl: Duration,
    /// 登录尝试次数统计的时间窗口
    #[serde(with = "humantime_serde")]
    pub login_try_window: Duration,
    /// 登录时间窗口内允许的最大尝试次数
    pub login_try_max_times: u32,
    /// 登录尝试次数超限后的封禁时长
    #[serde(with = "humantime_serde")]
    pub login_ban_ttl: Duration,
}

/// JWT 配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    /// JWT 密钥
    pub secret: String,
}

/// 主题配置
#[derive(Debug, Clone, Serialize)]
pub struct ThemeConfig {
    /// 主题文件存储的目录路径
    pub dir: String,
    /// 用户自定义主题目录
    pub custom_dir: String,
    /// 代码语法的来源（可选值：default/theme/custom）
    pub code_syntax_source: ThemeCodeSource,
    /// 代码主题的来源（可选值：default/theme/custom）
    pub code_themes_source: ThemeCodeSource,
    /// 当前使用的页面主题名称
    pub current_page_theme: String,
    /// 当前使用的代码主题名称
    pub current_code_theme: String,
    /// 渲染版本号
    pub render_version: String,
    /// 自定义扩展配置项
    #[serde(default)]
    pub extensions: HashMap<String, String>,
    /// 当前使用的主题配置
    #[serde(skip)]
    current: CurrentThemeConfig,
    /// 自定义的主题配置
    #[serde(skip)]
    custom: CustomThemeConfig,
}

impl ThemeConfig {
    pub fn current(&self) -> &CurrentThemeConfig {
        &self.current
    }

    pub fn custom(&self) -> &CustomThemeConfig {
        &self.custom
    }
}

impl<'de> Deserialize<'de> for ThemeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TempThemeConfig {
            dir: String,
            custom_dir: String,
            code_syntax_source: ThemeCodeSource,
            code_themes_source: ThemeCodeSource,
            current_page_theme: String,
            current_code_theme: String,
            render_version: String,
            #[serde(default)]
            extensions: HashMap<String, String>,
        }

        let temp = TempThemeConfig::deserialize(deserializer)?;
        let current_theme = CurrentThemeConfig::new(&temp.dir, &temp.current_page_theme);
        let custom_theme = CustomThemeConfig::new(&temp.custom_dir);

        Ok(ThemeConfig {
            dir: temp.dir,
            custom_dir: temp.custom_dir,
            code_syntax_source: temp.code_syntax_source,
            code_themes_source: temp.code_themes_source,
            current_page_theme: temp.current_page_theme,
            current_code_theme: temp.current_code_theme,
            render_version: temp.render_version,
            extensions: temp.extensions,
            current: current_theme,
            custom: custom_theme,
        })
    }
}

/// 当前使用的主题配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CurrentThemeConfig {
    /// 当前主题的静态资源目录
    pub assets_dir: String,
    /// 当前主题的模板文件目录
    pub templates_dir: String,
    /// 当前主题的代码主题文件目录
    pub code_themes_dir: String,
    /// 当前主题的代码语法文件目录
    pub code_syntax_dir: String,
}

impl CurrentThemeConfig {
    pub fn new(dir: &str, name: &str) -> Self {
        let base = PathJoin::root(dir).join(name);
        Self {
            assets_dir: base.clone().join("assets").into_string(),
            templates_dir: base.clone().join("templates").into_string(),
            code_themes_dir: base.clone().join("code/themes").into_string(),
            code_syntax_dir: base.clone().join("code/syntax").into_string(),
        }
    }
}

/// 自定义的主题配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomThemeConfig {
    /// 自定义的模板文件目录
    pub templates_dir: String,
    /// 自定义的代码主题文件目录
    pub code_themes_dir: String,
    /// 自定义的代码语法文件目录
    pub code_syntax_dir: String,
}

impl CustomThemeConfig {
    pub fn new(dir: &str) -> Self {
        let base = PathJoin::root(dir);
        Self {
            templates_dir: base.clone().join("templates").into_string(),
            code_themes_dir: base.clone().join("code/themes").into_string(),
            code_syntax_dir: base.clone().join("code/syntax").into_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeCodeSource {
    // 使用默认内置
    Default,
    // 使用主题内置
    Theme,
    // 使用自定义
    Custom,
}

/// 定时任务配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CronConfig {
    /// 定时任务项配置
    pub tasks: HashMap<String, CronTaskConfig>,
}

/// 定时任务项配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CronTaskConfig {
    /// 启用定时任务
    pub enabled: bool,
    /// 日程表达式
    pub schedule: String,
}

/// 文章配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticleConfig {
    /// 文章访问许可有效期
    #[serde(with = "humantime_serde")]
    pub access_ttl: Duration,
    /// 全文搜索匹配结果最大输出条目
    pub full_text_search_limit: u64,
    /// 文章标题最大长度限制
    pub title_max_length: usize,
    /// 文章摘要最大长度限制
    pub excerpt_max_length: usize,
    /// 文章正文最大字节限制
    #[serde(default, with = "crate::util::serde::human_size")]
    pub content_max_size: usize,
    /// 文章解锁尝试次数统计的时间窗口
    #[serde(with = "humantime_serde")]
    pub unlock_try_window: Duration,
    /// 文章解锁时间窗口内允许的最大尝试次数
    pub unlock_try_max_times: u32,
    /// 文章解锁尝试次数超限后的封禁时长
    #[serde(with = "humantime_serde")]
    pub unlock_ban_ttl: Duration,
    /// 作为 About 页面的文章
    #[serde(default)]
    pub about_article_id: Option<String>,
    /// 文章分页配置
    pub pagination: ArticlePaginationConfig,
}

/// 游客配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VisitorConfig {
    /// 游客会话的有效期
    #[serde(with = "humantime_serde")]
    pub session_ttl: Duration,
    /// 游客会话自动续期的触发阈值
    #[serde(with = "humantime_serde")]
    pub session_keep_threshold: Duration,
}

/// 文章分页配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticlePaginationConfig {
    /// 最大页码值
    pub max_page_number: u64,
    /// 默认分页大小
    pub default_page_size: u64,
    /// 分页大小允许的值列表
    pub allowed_page_sizes: Vec<u64>,
    /// 分页导航栏显示的页码数量
    pub page_nav_max_visible: u64,
}
