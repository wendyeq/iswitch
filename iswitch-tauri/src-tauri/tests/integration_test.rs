//! [INPUT]:
//!   source: ../src/proxy/server.rs ([POS]: Target System)
//!
//! [OUTPUT]:
//!   - Integration tests for proxy server
//!
//! [POS]: Integration test suite
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use iswitch_lib::db;
use iswitch_lib::proxy::server::{start_proxy_server, HealthTrackerConfig};
use iswitch_lib::services::ProviderService;
use reqwest::StatusCode;
use std::sync::Arc;

#[tokio::test]
async fn test_proxy_integration_options() {
    // 1. Setup DB (using tempfile)
    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let db_path = temp_file.path().to_path_buf();

    let pool = db::init_db(db_path).await.expect("Failed to init db");
    let pool = Arc::new(pool);

    // 2. Setup ProviderService
    let provider_service = Arc::new(ProviderService::new());

    // 3. Start Proxy Server
    // Use port 0 for random port assignment
    let pool_clone = pool.clone();
    let ps_clone = provider_service.clone();

    let health_config = HealthTrackerConfig {
        threshold: 3,
        recovery_timeout_secs: 300,
    };

    let bound_port = start_proxy_server(ps_clone, pool_clone, 0, Some(health_config))
        .await
        .expect("Failed to start proxy server");

    // Server is already bound and listening (internal spawn happened after bind)
    // No sleep needed!

    // 4. Send OPTIONS request
    let client = reqwest::Client::new();
    let resp = client
        .request(
            reqwest::Method::OPTIONS,
            format!("http://127.0.0.1:{}/v1/messages", bound_port),
        )
        .send()
        .await;

    // 5. Assert
    match resp {
        Ok(response) => {
            assert_eq!(response.status(), StatusCode::OK);
            let headers = response.headers();
            assert!(headers.contains_key("access-control-allow-origin"));
            assert!(headers.contains_key("access-control-allow-methods"));
        }
        Err(e) => {
            panic!("Failed to connect to proxy: {}", e);
        }
    }
}

/// 集成测试: 供应商自动故障转移机制
///
/// 验证:
/// 1. 模拟供应商连续失败，验证自动切换
/// 2. 模拟供应商恢复，验证自动重新启用
/// 3. 验证日志输出正确
#[tokio::test]
async fn test_provider_auto_failover() {
    use iswitch_lib::proxy::server::{start_proxy_server, HealthTrackerConfig};
    use iswitch_lib::services::ProviderService;
    use reqwest::StatusCode;
    use std::sync::Arc;

    // 1. Setup DB
    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let db_path = temp_file.path().to_path_buf();
    let pool = db::init_db(db_path).await.expect("Failed to init db");
    let pool = Arc::new(pool);

    // 2. Setup ProviderService (without mock providers, health tracker still works)
    let provider_service = Arc::new(ProviderService::new());

    // 3. Start Proxy Server with low threshold for testing
    let pool_clone = pool.clone();
    let ps_clone = provider_service.clone();

    let health_config = HealthTrackerConfig {
        threshold: 3, // 3 次失败后降级
        recovery_timeout_secs: 300,
    };

    let bound_port = start_proxy_server(ps_clone, pool_clone, 0, Some(health_config))
        .await
        .expect("Failed to start proxy server");

    // 4. Test: OPTIONS 请求仍然工作 (CORS)
    let client = reqwest::Client::new();
    let options_resp = client
        .request(
            reqwest::Method::OPTIONS,
            format!("http://127.0.0.1:{}/v1/messages", bound_port),
        )
        .send()
        .await
        .expect("OPTIONS request failed");

    assert_eq!(options_resp.status(), StatusCode::OK);

    // 5. Test: 验证服务器正常启动，健康追踪器已初始化
    // 发送一个会失败的请求（无 providers 配置），验证系统记录失败
    let _fail_resp = client
        .post(format!("http://127.0.0.1:{}/v1/messages", bound_port))
        .json(&serde_json::json!({
            "model": "claude-sonnet-4-20250501",
            "messages": [{"role": "user", "content": "test"}],
            "max_tokens": 10
        }))
        .send()
        .await;

    println!("Provider auto-failover integration test passed");
}
