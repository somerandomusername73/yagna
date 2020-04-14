use anyhow::anyhow;
use std::rc::Rc;
use std::time::Duration;
use tokio::time::delay_for;
use url::Url;

use ya_client::{
    market::MarketProviderApi,
    web::{WebClient, WebInterface},
};
use ya_core_model::{appkey, market};
use ya_identity::cli::AppKeyCommand;
use ya_service_api::CliCtx;
use ya_service_api_interfaces::Service;
use ya_service_bus::{typed as bus, RpcEndpoint, RpcMessage};

use crate::Error;

const CREATE_TEST_APPKEY_ENV_VAR: &str = "YAGNA_CREATE_TEST_APPKEY";
const KEY_EXPORT_RETRY_COUNT_ENV_VAR: &str = "YAGNA_KEY_EXPORT_RETRY_COUNT";
const KEY_EXPORT_RETRY_DELAY_ENV_VAR: &str = "YAGNA_KEY_EXPORT_RETRY_DELAY";
const TEST_APPKEY_ENV_VAR: &str = "YAGNA_TEST_APPKEY_NAME";

const DEFAULT_KEY_EXPORT_RETRY_COUNT: u8 = 0;
const DEFAULT_KEY_EXPORT_RETRY_DELAY: Duration = Duration::from_secs(5);
const DEFAULT_TEST_APPKEY_NAME: &str = "test_key";

fn get_key_export_retry_count() -> u8 {
    return std::env::var(KEY_EXPORT_RETRY_COUNT_ENV_VAR)
        .and_then(|var| var.parse().map_err(|_| std::env::VarError::NotPresent))
        .unwrap_or(DEFAULT_KEY_EXPORT_RETRY_COUNT);
}

fn get_key_export_retry_delay() -> Duration {
    return std::env::var(KEY_EXPORT_RETRY_DELAY_ENV_VAR)
        .and_then(|var| {
            var.parse()
                .map(|parsed: u8| Duration::from_secs(parsed.into()))
                .map_err(|_| std::env::VarError::NotPresent)
        })
        .unwrap_or(DEFAULT_KEY_EXPORT_RETRY_DELAY);
}

fn get_test_appkey_name() -> String {
    std::env::var(TEST_APPKEY_ENV_VAR).unwrap_or(DEFAULT_TEST_APPKEY_NAME.to_string())
}

pub type RpcMessageResult<T> = Result<<T as RpcMessage>::Item, <T as RpcMessage>::Error>;

pub struct MarketService;

impl Service for MarketService {
    type Cli = ();
}

impl MarketService {
    pub async fn gsb<Context>(_: &Context) -> anyhow::Result<()> {
        let _ = bus::bind(market::BUS_ID, |get: market::GetAgreement| async move {
            Ok(get_agreement(&get).await?)
        });

        tmp_send_keys()
            .await
            .unwrap_or_else(|e| log::error!("app-key export error: {}", e));

        Ok(())
    }
}

async fn get_agreement(get: &market::GetAgreement) -> Result<market::Agreement, Error> {
    let market_api: MarketProviderApi = WebClient::builder().build()?.interface()?;

    Ok(market_api.get_agreement(&get.agreement_id).await?)
}

async fn get_app_keys() -> anyhow::Result<Vec<serde_json::Value>> {
    let (ids, _n) = bus::service(appkey::BUS_ID)
        .send(appkey::List {
            identity: None,
            page: 1,
            per_page: 10,
        })
        .await??;

    let ids: Vec<serde_json::Value> = ids
        .into_iter()
        .map(|k: appkey::AppKey| serde_json::json! {{"key": k.key, "nodeId": k.identity}})
        .collect();

    Ok(ids)
}

async fn create_test_key() -> anyhow::Result<()> {
    let ids = get_app_keys().await?;

    if ids.len() == 0 {
        log::info!("Creating test app-key");

        AppKeyCommand::Create {
            name: get_test_appkey_name(),
            role: appkey::DEFAULT_ROLE.to_string(),
            id: None,
        }
        .run_command(&CliCtx::default())
        .await?;

        log::info!("Test app-key created");
    }

    Ok(())
}

async fn tmp_send_keys() -> anyhow::Result<()> {
    if std::env::var(CREATE_TEST_APPKEY_ENV_VAR).is_ok() {
        create_test_key().await?;
    }

    let mut url =
        MarketProviderApi::rebase_service_url(Rc::new(Url::parse("http://127.0.0.1:5001")?))?
            .as_ref()
            .clone();
    url.set_path("admin/import-key");
    log::debug!("posting to: {:?}", url);

    let request: awc::FrozenClientRequest = awc::Client::new()
        .post(url.to_string())
        .freeze()
        .map_err(|e| anyhow!("Failed to build frozen request. Error: {}", e.to_string()))?;

    let request_count = get_key_export_retry_count() + 1;
    let retry_delay = get_key_export_retry_delay();

    for count in 0..request_count {
        let ids = get_app_keys().await?;

        match request.send_json(&ids).await {
            Ok(mut response) => {
                let parsed_response: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| anyhow!("Failed to parse key export response. Error: {}", e))?;

                log::info!(
                    "Key export successful, exported keys count: {}",
                    parsed_response
                );
                break;
            }
            Err(e) => {
                if count == request_count - 1 {
                    log::error!("Key export failed, no retries left. Error: {}", e);
                } else {
                    log::debug!(
                        "Key export failed, retrying in: {} seconds. Retries left: {}",
                        retry_delay.as_secs(),
                        request_count - (count + 1)
                    );
                    delay_for(retry_delay).await;
                }
            }
        }
    }

    Ok(())
}
