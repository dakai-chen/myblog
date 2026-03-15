#![allow(dead_code)]

mod app;
mod config;
mod context;
mod cron;
mod error;
mod jwt;
mod logger;
mod markdown;
mod middleware;
mod model;
mod response;
mod service;
mod shutdown;
mod state;
mod storage;
mod template;
mod util;
mod validator;

use std::env::VarError;
use std::net::SocketAddr;
use std::sync::Arc;

use boluo::server::{RunError, Server};
use tokio::net::TcpListener;

use crate::config::HttpConfig;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::from_path("./.env").or_else(|e| {
        // 忽略文件未找到错误
        e.not_found().then_some(()).ok_or(e)
    })?;

    let mode: Option<String> = match std::env::var("app.mode") {
        Ok(val) => Some(val),
        Err(VarError::NotPresent) => None,
        Err(e) => return Err(e.into()),
    };
    config::init(mode.as_deref())?;

    shutdown::set_timeout(config::get().http.shutdown_timeout);

    let _guard = logger::init(&config::get().logger)?;

    tracing::debug!("{}", serde_json::to_string_pretty(config::get())?);

    let state = AppState::from_config(config::get()).await?;

    if config::get().database.migrations.auto_migrate {
        let mut db = state.db.acquire().await?;
        crate::storage::db::init(&mut db).await?;
    }

    crate::storage::cache::storage::init(state.db.clone())?;

    cron::init(state.clone()).await?;
    cron::start().await?;

    start_http_server(state).await?;

    cron::shutdown().await?;

    Ok(())
}

async fn start_http_server(state: Arc<AppState>) -> anyhow::Result<()> {
    let app = app::build(state).await?;
    let tcp = listen().await?;

    tracing::info!("HTTP 服务启动，监听地址：{}", tcp.local_addr()?);
    if let Err(e) = Server::new(tcp)
        .run_with_graceful_shutdown(app, shutdown::graceful())
        .await
    {
        handle_run_error(e).await;
    }
    tracing::info!("HTTP 服务已关闭");

    Ok(())
}

async fn listen() -> anyhow::Result<TcpListener> {
    let HttpConfig {
        bind_ip, bind_port, ..
    } = config::get().http;
    Ok(TcpListener::bind(SocketAddr::from((bind_ip, bind_port))).await?)
}

async fn handle_run_error<E>(error: RunError<E>)
where
    E: std::fmt::Display,
{
    match error {
        RunError::GracefulShutdownTimeout => {
            tracing::warn!("HTTP 服务优雅关闭超时");
        }
        RunError::Listener(e, graceful_shutdown) => {
            tracing::error!("HTTP 服务监听失败: {e}");
            if let Some(timeout) = shutdown::timeout() {
                tracing::info!(
                    "HTTP 服务开始优雅关闭，等待活跃请求处理完成（超时时间：{timeout:?}）"
                );
            } else {
                tracing::info!("HTTP 服务开始优雅关闭，等待活跃请求处理完成");
            }
            if !graceful_shutdown.shutdown(shutdown::timeout()).await {
                tracing::warn!("HTTP 服务优雅关闭超时");
            }
        }
    }
}
