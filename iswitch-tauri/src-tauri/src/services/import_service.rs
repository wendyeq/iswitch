//! [INPUT]:
//!   source: ../../../../code-switch/services/importservice.go ([POS]: 原 Go 实现参考)
//!   source: ../models/provider.rs ([POS]: Provider 数据模型)
//!   source: ../models/mcp.rs ([POS]: MCP 数据模型)
//!
//! [OUTPUT]:
//!   - ImportService 结构体
//!   - ConfigImportStatus, ConfigImportResult 数据结构
//!   - get_status(), import_all(), import_from_file() API
//!
//! [POS]: 配置导入服务，支持从 cc-switch 和 code-switch 导入供应商和 MCP 配置
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::{AppError, AppResult};
use crate::models::{MCPServer, MCPServerType, Provider};
use crate::services::{MCPService, ProviderService};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

// === 导出状态结构 ===

/// 导入来源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportSource {
    /// 旧版 cc-switch (~/.cc-switch/)
    CcSwitch,
    /// Go 版本 code-switch (~/.code-switch/)
    CodeSwitch,
    /// 用户指定的文件路径
    CustomFile,
}

impl ImportSource {
    pub fn display_name(&self) -> &'static str {
        match self {
            ImportSource::CcSwitch => "cc-switch (旧版)",
            ImportSource::CodeSwitch => "code-switch (Go 版本)",
            ImportSource::CustomFile => "自定义文件",
        }
    }
}

/// 导入来源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSourceInfo {
    /// 来源类型
    pub source: ImportSource,
    /// 配置文件路径
    pub config_path: String,
    /// 是否存在配置
    pub config_exists: bool,
    /// 待导入的 Provider 数量
    pub pending_provider_count: i32,
    /// 待导入的 MCP 数量
    pub pending_mcp_count: i32,
}

/// 导入状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportStatus {
    /// 是否存在旧版配置
    pub config_exists: bool,
    /// 配置文件路径
    pub config_path: String,
    /// 是否有待导入的 Provider
    pub pending_providers: bool,
    /// 是否有待导入的 MCP
    pub pending_mcp: bool,
    /// 待导入的 Provider 数量
    pub pending_provider_count: i32,
    /// 待导入的 MCP 数量
    pub pending_mcp_count: i32,
    /// 导入来源
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ImportSource>,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigImportResult {
    /// 导入后的状态
    pub status: ImportStatus,
    /// 导入的 Provider 数量
    pub imported_providers: i32,
    /// 导入的 MCP 数量
    pub imported_mcp: i32,
}

// === 内部数据结构 (对应旧版 cc-switch 配置格式) ===

/// 旧版 cc-switch 配置文件结构
#[derive(Debug, Clone, Deserialize, Default)]
struct CCSwitchConfig {
    #[serde(default)]
    claude: CCProviderSection,
    #[serde(default)]
    codex: CCProviderSection,
    #[serde(default)]
    mcp: CCMCPSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CCProviderSection {
    #[serde(default)]
    providers: HashMap<String, CCProviderEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CCProviderEntry {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(rename = "websiteUrl", default)]
    website_url: String,
    #[serde(rename = "settingsConfig", default)]
    settings: CCProviderSettings,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CCProviderSettings {
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    auth: HashMap<String, String>,
    #[serde(default)]
    config: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CCMCPSection {
    #[serde(default)]
    claude: CCMCPPlatform,
    #[serde(default)]
    codex: CCMCPPlatform,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CCMCPPlatform {
    #[serde(default)]
    servers: HashMap<String, CCMCPServerEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CCMCPServerEntry {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    homepage: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    server: CCMCPServerConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CCMCPServerConfig {
    #[serde(rename = "type", default)]
    server_type: String,
    #[serde(default)]
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    url: String,
}

/// Provider 候选项 (从旧配置解析)
#[derive(Debug, Clone)]
struct ProviderCandidate {
    name: String,
    api_url: String,
    api_key: String,
    site: String,
    /// 原始 ID (用于 code-switch 去重)
    id: Option<i64>,
    icon: Option<String>,
    tint: Option<String>,
    accent: Option<String>,
    supported_models: Option<HashMap<String, bool>>,
    model_mapping: Option<HashMap<String, String>>,
}

// === Go 版本 code-switch 配置格式 ===

/// Go 版本 code-switch Provider 格式
/// 对应 ~/.code-switch/claude-code.json 和 ~/.code-switch/codex.json 中的 provider 结构
#[derive(Debug, Clone, Deserialize)]
pub struct CodeSwitchProvider {
    pub id: i64,
    pub name: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "officialSite", default)]
    pub official_site: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub tint: String,
    #[serde(default)]
    pub accent: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(rename = "supportedModels", default)]
    pub supported_models: HashMap<String, bool>,
    #[serde(rename = "modelMapping", default)]
    pub model_mapping: HashMap<String, String>,
}

/// claude-code.json / codex.json 文件结构
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CodeSwitchProviderFile {
    #[serde(default)]
    pub providers: Vec<CodeSwitchProvider>,
}

/// Go 版本 code-switch MCP 服务器格式
/// 对应 ~/.code-switch/mcp.json 中的服务器结构
#[derive(Debug, Clone, Deserialize)]
pub struct CodeSwitchMCPServer {
    #[serde(rename = "type", default)]
    pub server_type: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub tips: String,
    #[serde(rename = "enable_platform", default)]
    pub enable_platform: Vec<String>,
}

/// Go 版本 code-switch 聚合配置
/// 从 ~/.code-switch/ 目录下的多个文件聚合而来
#[derive(Debug, Clone, Default)]
pub struct CodeSwitchConfig {
    /// 来自 claude-code.json
    pub claude_providers: Vec<CodeSwitchProvider>,
    /// 来自 codex.json
    pub codex_providers: Vec<CodeSwitchProvider>,
    /// 来自 mcp.json (key 为服务器名称)
    pub mcp_servers: HashMap<String, CodeSwitchMCPServer>,
}

// === 导入服务 ===

/// 导入服务
///
/// 负责从旧版 cc-switch 和 code-switch 配置导入供应商和 MCP 配置
pub struct ImportService {
    /// Provider 服务引用
    provider_service: Arc<ProviderService>,
    /// MCP 服务引用
    mcp_service: Arc<MCPService>,
    /// cc-switch 配置路径
    cc_switch_path: PathBuf,
    /// code-switch 配置路径
    code_switch_path: PathBuf,
}

impl ImportService {
    /// 创建新的导入服务实例
    pub fn new(provider_service: Arc<ProviderService>, mcp_service: Arc<MCPService>) -> Self {
        use crate::utils::paths::{old_cc_switch_dir, old_code_switch_dir};
        Self {
            provider_service,
            mcp_service,
            cc_switch_path: old_cc_switch_dir().join("config.json"),
            // code-switch 使用目录而非单个文件（多文件聚合格式）
            code_switch_path: old_code_switch_dir(),
        }
    }

    /// 创建带自定义路径的导入服务 (用于测试)
    #[cfg(test)]
    pub fn with_paths(
        provider_service: Arc<ProviderService>,
        mcp_service: Arc<MCPService>,
        cc_switch_path: PathBuf,
        code_switch_path: PathBuf,
    ) -> Self {
        Self {
            provider_service,
            mcp_service,
            cc_switch_path,
            code_switch_path,
        }
    }

    /// 列出所有可用的导入来源
    ///
    /// 返回所有检测到的配置来源及其状态
    pub async fn list_import_sources(&self) -> AppResult<Vec<ImportSourceInfo>> {
        let mut sources = Vec::new();

        // 检查 cc-switch
        if let Some(info) = self
            .check_source(ImportSource::CcSwitch, &self.cc_switch_path)
            .await?
        {
            sources.push(info);
        }

        // 检查 code-switch
        if let Some(info) = self
            .check_source(ImportSource::CodeSwitch, &self.code_switch_path)
            .await?
        {
            sources.push(info);
        }

        debug!(count = sources.len(), "检测到可用的导入来源");
        Ok(sources)
    }

    /// 检查单个导入来源
    async fn check_source(
        &self,
        source: ImportSource,
        path: &Path,
    ) -> AppResult<Option<ImportSourceInfo>> {
        let config_exists = path.exists();

        if !config_exists {
            return Ok(Some(ImportSourceInfo {
                source,
                config_path: path.display().to_string(),
                config_exists: false,
                pending_provider_count: 0,
                pending_mcp_count: 0,
            }));
        }

        let config = match self.load_config(path).await? {
            Some(cfg) => cfg,
            None => return Ok(None),
        };

        let pending_claude = self
            .get_pending_providers("claude", &config.claude.providers)
            .await?;
        let pending_codex = self
            .get_pending_providers("codex", &config.codex.providers)
            .await?;
        let provider_count = (pending_claude.len() + pending_codex.len()) as i32;

        let pending_mcp = self.get_pending_mcp_servers(&config).await?;
        let mcp_count = pending_mcp.len() as i32;

        Ok(Some(ImportSourceInfo {
            source,
            config_path: path.display().to_string(),
            config_exists: true,
            pending_provider_count: provider_count,
            pending_mcp_count: mcp_count,
        }))
    }

    /// 从指定来源导入配置
    pub async fn import_from_source(&self, source: ImportSource) -> AppResult<ConfigImportResult> {
        let path = match source {
            ImportSource::CcSwitch => &self.cc_switch_path,
            ImportSource::CodeSwitch => &self.code_switch_path,
            ImportSource::CustomFile => {
                return Err(AppError::InvalidInput(
                    "请使用 import_from_file 指定自定义文件路径".into(),
                ));
            }
        };

        info!(source = ?source, path = %path.display(), "从指定来源导入配置");
        self.import_from_path(path, Some(source)).await
    }

    /// 获取导入状态（优先检查 code-switch，其次 cc-switch）
    ///
    /// 检测旧版配置是否存在，以及有多少待导入的内容
    pub async fn get_status(&self) -> AppResult<ImportStatus> {
        // 优先检查 code-switch（最新的 Go 版本）
        if self.code_switch_path.exists() {
            debug!(path = %self.code_switch_path.display(), "检查 code-switch 导入状态");
            return self
                .get_status_for_path(&self.code_switch_path, Some(ImportSource::CodeSwitch))
                .await;
        }

        // 其次检查 cc-switch
        if self.cc_switch_path.exists() {
            debug!(path = %self.cc_switch_path.display(), "检查 cc-switch 导入状态");
            return self
                .get_status_for_path(&self.cc_switch_path, Some(ImportSource::CcSwitch))
                .await;
        }

        // 都不存在
        Ok(ImportStatus::default())
    }

    /// 获取指定路径的导入状态
    async fn get_status_for_path(
        &self,
        path: &Path,
        source: Option<ImportSource>,
    ) -> AppResult<ImportStatus> {
        let mut status = ImportStatus {
            config_path: path.display().to_string(),
            source,
            ..Default::default()
        };

        let config = match self.load_config(path).await? {
            Some(cfg) => {
                status.config_exists = true;
                cfg
            }
            None => {
                debug!("配置不存在");
                return Ok(status);
            }
        };

        self.evaluate_status(&config, &mut status).await?;
        Ok(status)
    }

    /// 导入所有配置（从第一个可用来源）
    ///
    /// 优先从 code-switch 导入，如果不存在则从 cc-switch 导入
    pub async fn import_all(&self) -> AppResult<ConfigImportResult> {
        // 优先从 code-switch 导入
        if self.code_switch_path.exists() {
            info!(path = %self.code_switch_path.display(), "从 code-switch 导入配置");
            return self
                .import_from_path(&self.code_switch_path, Some(ImportSource::CodeSwitch))
                .await;
        }

        // 其次从 cc-switch 导入
        if self.cc_switch_path.exists() {
            info!(path = %self.cc_switch_path.display(), "从 cc-switch 导入配置");
            return self
                .import_from_path(&self.cc_switch_path, Some(ImportSource::CcSwitch))
                .await;
        }

        // 都不存在
        Ok(ConfigImportResult::default())
    }

    /// 从指定文件导入配置
    pub async fn import_from_file(&self, path: &str) -> AppResult<ConfigImportResult> {
        let path = Path::new(path.trim());
        if path.as_os_str().is_empty() {
            return Err(AppError::InvalidInput("配置路径不能为空".into()));
        }
        info!(path = %path.display(), "从指定文件导入配置");
        self.import_from_path(path, Some(ImportSource::CustomFile))
            .await
    }

    /// 从路径导入配置
    async fn import_from_path(
        &self,
        path: &Path,
        source: Option<ImportSource>,
    ) -> AppResult<ConfigImportResult> {
        let mut result = ConfigImportResult {
            status: ImportStatus {
                config_path: path.display().to_string(),
                source,
                ..Default::default()
            },
            ..Default::default()
        };

        // 加载配置
        let config = match self.load_config(path).await? {
            Some(cfg) => {
                result.status.config_exists = true;
                cfg
            }
            None => {
                warn!(path = %path.display(), "配置文件不存在");
                return Ok(result);
            }
        };

        // 导入 Provider
        let imported_providers = self.import_providers(&config).await?;
        result.imported_providers = imported_providers;

        // 导入 MCP
        let imported_mcp = self.import_mcp_servers(&config).await?;
        result.imported_mcp = imported_mcp;

        // 更新状态
        self.evaluate_status(&config, &mut result.status).await?;

        info!(
            providers = imported_providers,
            mcp = imported_mcp,
            "配置导入完成"
        );
        Ok(result)
    }

    /// 加载旧版配置文件
    async fn load_config(&self, path: &Path) -> AppResult<Option<CCSwitchConfig>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            error!(error = %e, path = %path.display(), "读取配置文件失败");
            AppError::ConfigRead {
                path: path.display().to_string(),
                source: e,
            }
        })?;

        if content.trim().is_empty() {
            return Ok(Some(CCSwitchConfig::default()));
        }

        let config: CCSwitchConfig = serde_json::from_str(&content).map_err(|e| {
            error!(error = %e, "解析配置文件失败");
            AppError::ConfigParse(e)
        })?;

        Ok(Some(config))
    }

    /// 评估导入状态
    async fn evaluate_status(
        &self,
        config: &CCSwitchConfig,
        status: &mut ImportStatus,
    ) -> AppResult<()> {
        // 计算待导入的 Provider
        let pending_claude = self
            .get_pending_providers("claude", &config.claude.providers)
            .await?;
        let pending_codex = self
            .get_pending_providers("codex", &config.codex.providers)
            .await?;
        let provider_count = (pending_claude.len() + pending_codex.len()) as i32;

        status.pending_providers = provider_count > 0;
        status.pending_provider_count = provider_count;

        // 计算待导入的 MCP
        let pending_mcp = self.get_pending_mcp_servers(config).await?;
        status.pending_mcp_count = pending_mcp.len() as i32;
        status.pending_mcp = status.pending_mcp_count > 0;

        Ok(())
    }

    /// 获取待导入的 Provider 列表
    async fn get_pending_providers(
        &self,
        kind: &str,
        entries: &HashMap<String, CCProviderEntry>,
    ) -> AppResult<Vec<ProviderCandidate>> {
        if entries.is_empty() {
            return Ok(Vec::new());
        }

        // 加载现有 Provider
        let existing = self.provider_service.load_providers(kind).await?;
        let existing_urls: std::collections::HashSet<_> =
            existing.iter().map(|p| normalize_url(&p.api_url)).collect();
        let existing_names: std::collections::HashSet<_> =
            existing.iter().map(|p| normalize_name(&p.name)).collect();

        let mut candidates = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for (key, entry) in entries {
            let candidate = match self.parse_provider_entry(kind, key, entry) {
                Some(c) => c,
                None => continue,
            };

            let url = normalize_url(&candidate.api_url);
            let name = normalize_name(&candidate.name);

            // 跳过已存在的
            if !url.is_empty() && existing_urls.contains(&url) {
                continue;
            }
            if !name.is_empty() && existing_names.contains(&name) {
                continue;
            }

            // 跳过重复
            let dedup_key = if url.is_empty() {
                name.clone()
            } else {
                url.clone()
            };
            if !dedup_key.is_empty() && seen.contains(&dedup_key) {
                continue;
            }

            if !dedup_key.is_empty() {
                seen.insert(dedup_key);
            }
            candidates.push(candidate);
        }

        // 按名称排序
        candidates.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(candidates)
    }

    /// 解析 Provider 条目
    fn parse_provider_entry(
        &self,
        kind: &str,
        key: &str,
        entry: &CCProviderEntry,
    ) -> Option<ProviderCandidate> {
        let name = if !entry.name.trim().is_empty() {
            entry.name.trim().to_string()
        } else if !entry.id.trim().is_empty() {
            entry.id.trim().to_string()
        } else {
            key.trim().to_string()
        };

        let site = entry.website_url.trim().to_string();

        match kind.to_lowercase().as_str() {
            "claude" => {
                let api_url = entry
                    .settings
                    .env
                    .get("ANTHROPIC_BASE_URL")
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let api_key = entry
                    .settings
                    .env
                    .get("ANTHROPIC_AUTH_TOKEN")
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                if api_url.is_empty() || api_key.is_empty() {
                    return None;
                }

                Some(ProviderCandidate {
                    name,
                    api_url,
                    api_key,
                    site,
                    id: None,
                    icon: None,
                    tint: None,
                    accent: None,
                    supported_models: None,
                    model_mapping: None,
                })
            }
            "codex" => {
                let api_key = pick_first_non_empty(&[
                    entry.settings.auth.get("OPENAI_API_KEY"),
                    entry.settings.auth.get("OPENAI_API_KEY_1"),
                    entry.settings.env.get("OPENAI_API_KEY"),
                ]);

                if api_key.is_empty() {
                    return None;
                }

                let api_url = self.resolve_codex_api_url(&entry.settings.config);
                if api_url.is_empty() {
                    return None;
                }

                Some(ProviderCandidate {
                    name,
                    api_url,
                    api_key,
                    site,
                    id: None,
                    icon: None,
                    tint: None,
                    accent: None,
                    supported_models: None,
                    model_mapping: None,
                })
            }
            _ => None,
        }
    }

    /// 解析 Codex 配置中的 API URL
    fn resolve_codex_api_url(&self, raw_config: &str) -> String {
        if raw_config.trim().is_empty() {
            return String::new();
        }

        // 解析 TOML 配置
        #[derive(Deserialize)]
        struct CodexConfig {
            model_provider: Option<String>,
            #[serde(default)]
            model_providers: HashMap<String, CodexProviderConfig>,
        }

        #[derive(Deserialize)]
        struct CodexProviderConfig {
            base_url: Option<String>,
        }

        let config: CodexConfig = match toml::from_str(raw_config) {
            Ok(c) => c,
            Err(_) => return String::new(),
        };

        // 尝试根据 model_provider 查找
        if let Some(ref provider_key) = config.model_provider {
            if let Some(provider) = config.model_providers.get(provider_key) {
                if let Some(ref url) = provider.base_url {
                    return url.trim().to_string();
                }
            }
        }

        // 返回第一个非空的 base_url
        for provider in config.model_providers.values() {
            if let Some(ref url) = provider.base_url {
                if !url.trim().is_empty() {
                    return url.trim().to_string();
                }
            }
        }

        String::new()
    }

    /// 导入 Provider
    async fn import_providers(&self, config: &CCSwitchConfig) -> AppResult<i32> {
        let mut total = 0;

        // 导入 Claude Provider
        let claude_candidates = self
            .get_pending_providers("claude", &config.claude.providers)
            .await?;
        if !claude_candidates.is_empty() {
            let added = self.save_providers("claude", claude_candidates).await?;
            total += added;
        }

        // 导入 Codex Provider
        let codex_candidates = self
            .get_pending_providers("codex", &config.codex.providers)
            .await?;
        if !codex_candidates.is_empty() {
            let added = self.save_providers("codex", codex_candidates).await?;
            total += added;
        }

        Ok(total)
    }

    /// 保存 Provider 到服务
    async fn save_providers(
        &self,
        kind: &str,
        candidates: Vec<ProviderCandidate>,
    ) -> AppResult<i32> {
        let mut existing = self.provider_service.load_providers(kind).await?;
        let next_id = existing.iter().map(|p| p.id).max().unwrap_or(0) + 1;

        let (accent, tint) = default_visual(kind);

        for (i, candidate) in candidates.iter().enumerate() {
            let provider = Provider {
                id: next_id + i as i64,
                name: candidate.name.clone(),
                api_url: candidate.api_url.clone(),
                api_key: candidate.api_key.clone(),
                site: candidate.site.clone(),
                tint: tint.clone(),
                accent: accent.clone(),
                enabled: true,
                ..Default::default()
            };
            existing.push(provider);
        }

        self.provider_service.save_providers(kind, existing).await?;
        Ok(candidates.len() as i32)
    }

    /// 获取待导入的 MCP 服务器
    async fn get_pending_mcp_servers(&self, config: &CCSwitchConfig) -> AppResult<Vec<MCPServer>> {
        let existing = self.mcp_service.list_servers().await?;
        let existing_names: std::collections::HashSet<_> =
            existing.iter().map(|s| normalize_name(&s.name)).collect();

        let mut servers = self.collect_mcp_servers(config);
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        for server in servers.drain(..) {
            let name = normalize_name(&server.name);
            if name.is_empty() {
                continue;
            }
            if existing_names.contains(&name) {
                continue;
            }
            if seen.contains(&name) {
                continue;
            }
            seen.insert(name);
            result.push(server);
        }

        result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(result)
    }

    /// 收集 MCP 服务器配置
    fn collect_mcp_servers(&self, config: &CCSwitchConfig) -> Vec<MCPServer> {
        let mut stores: HashMap<String, MCPServer> = HashMap::new();

        self.append_mcp_entries(&mut stores, &config.mcp.claude.servers, "claude-code");
        self.append_mcp_entries(&mut stores, &config.mcp.codex.servers, "codex");

        stores.into_values().collect()
    }

    /// 添加 MCP 条目
    fn append_mcp_entries(
        &self,
        target: &mut HashMap<String, MCPServer>,
        entries: &HashMap<String, CCMCPServerEntry>,
        platform: &str,
    ) {
        for (key, entry) in entries {
            let name = if !entry.name.trim().is_empty() {
                entry.name.trim().to_string()
            } else if !entry.id.trim().is_empty() {
                entry.id.trim().to_string()
            } else {
                key.trim().to_string()
            };

            if name.is_empty() {
                continue;
            }

            let server_type = if !entry.server.server_type.is_empty() {
                MCPServerType::from_str(&entry.server.server_type)
            } else if !entry.server.url.is_empty() {
                MCPServerType::Http
            } else if !entry.server.command.is_empty() {
                MCPServerType::Stdio
            } else {
                continue;
            };

            // 验证必要字段
            if server_type == MCPServerType::Http && entry.server.url.is_empty() {
                continue;
            }
            if server_type == MCPServerType::Stdio && entry.server.command.is_empty() {
                continue;
            }

            let normalized_name = name.to_lowercase();

            if let Some(existing) = target.get_mut(&normalized_name) {
                // 更新现有条目
                if entry.enabled && !existing.enable_platform.contains(&platform.to_string()) {
                    existing.enable_platform.push(platform.to_string());
                }
            } else {
                // 创建新条目
                let mut enable_platform = Vec::new();
                if entry.enabled {
                    enable_platform.push(platform.to_string());
                }

                let server = MCPServer {
                    name,
                    server_type,
                    command: entry.server.command.trim().to_string(),
                    args: entry.server.args.clone(),
                    env: entry.server.env.clone(),
                    url: entry.server.url.trim().to_string(),
                    website: entry.homepage.trim().to_string(),
                    tips: entry.description.trim().to_string(),
                    enable_platform,
                    enabled_in_claude: platform == "claude-code" && entry.enabled,
                    enabled_in_codex: platform == "codex" && entry.enabled,
                    ..Default::default()
                };

                target.insert(normalized_name, server);
            }
        }
    }

    /// 导入 MCP 服务器
    async fn import_mcp_servers(&self, config: &CCSwitchConfig) -> AppResult<i32> {
        let candidates = self.get_pending_mcp_servers(config).await?;
        if candidates.is_empty() {
            return Ok(0);
        }

        let mut existing = self.mcp_service.list_servers().await?;
        let count = candidates.len() as i32;
        existing.extend(candidates);

        self.mcp_service.save_servers(existing).await?;
        Ok(count)
    }

    // ============================================================
    // Code-Switch 特定方法（Go 版本分散配置格式）
    // ============================================================

    /// 加载 Code-Switch 配置（从 ~/.code-switch/ 目录聚合）
    ///
    /// 从以下文件聚合配置：
    /// - claude-code.json: Claude providers
    /// - codex.json: Codex providers
    /// - mcp.json: MCP servers
    pub async fn load_code_switch_config(&self) -> AppResult<Option<CodeSwitchConfig>> {
        let dir = &self.code_switch_path;

        if !dir.exists() || !dir.is_dir() {
            debug!(path = %dir.display(), "Code-Switch 配置目录不存在");
            return Ok(None);
        }

        let mut config = CodeSwitchConfig::default();
        let mut has_any_file = false;

        // 加载 claude-code.json
        let claude_path = dir.join("claude-code.json");
        if claude_path.exists() {
            match self.load_code_switch_providers(&claude_path).await {
                Ok(providers) => {
                    debug!(count = providers.len(), "加载 Claude providers 成功");
                    config.claude_providers = providers;
                    has_any_file = true;
                }
                Err(e) => {
                    warn!(error = %e, path = %claude_path.display(), "加载 claude-code.json 失败，跳过");
                }
            }
        }

        // 加载 codex.json
        let codex_path = dir.join("codex.json");
        if codex_path.exists() {
            match self.load_code_switch_providers(&codex_path).await {
                Ok(providers) => {
                    debug!(count = providers.len(), "加载 Codex providers 成功");
                    config.codex_providers = providers;
                    has_any_file = true;
                }
                Err(e) => {
                    warn!(error = %e, path = %codex_path.display(), "加载 codex.json 失败，跳过");
                }
            }
        }

        // 加载 mcp.json
        let mcp_path = dir.join("mcp.json");
        if mcp_path.exists() {
            match self.load_code_switch_mcp_servers(&mcp_path).await {
                Ok(servers) => {
                    debug!(count = servers.len(), "加载 MCP servers 成功");
                    config.mcp_servers = servers;
                    has_any_file = true;
                }
                Err(e) => {
                    warn!(error = %e, path = %mcp_path.display(), "加载 mcp.json 失败，跳过");
                }
            }
        }

        if has_any_file {
            Ok(Some(config))
        } else {
            debug!("Code-Switch 目录中没有可用的配置文件");
            Ok(None)
        }
    }

    /// 加载 Code-Switch Provider 文件（claude-code.json 或 codex.json）
    async fn load_code_switch_providers(&self, path: &Path) -> AppResult<Vec<CodeSwitchProvider>> {
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            error!(error = %e, path = %path.display(), "读取 Code-Switch Provider 文件失败");
            AppError::ConfigRead {
                path: path.display().to_string(),
                source: e,
            }
        })?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let file: CodeSwitchProviderFile = serde_json::from_str(&content).map_err(|e| {
            error!(error = %e, path = %path.display(), "解析 Code-Switch Provider 文件失败");
            AppError::ConfigParse(e)
        })?;

        Ok(file.providers)
    }

    /// 加载 Code-Switch MCP 服务器文件（mcp.json）
    async fn load_code_switch_mcp_servers(
        &self,
        path: &Path,
    ) -> AppResult<HashMap<String, CodeSwitchMCPServer>> {
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            error!(error = %e, path = %path.display(), "读取 Code-Switch MCP 文件失败");
            AppError::ConfigRead {
                path: path.display().to_string(),
                source: e,
            }
        })?;

        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }

        let servers: HashMap<String, CodeSwitchMCPServer> = serde_json::from_str(&content)
            .map_err(|e| {
                error!(error = %e, path = %path.display(), "解析 Code-Switch MCP 文件失败");
                AppError::ConfigParse(e)
            })?;

        Ok(servers)
    }

    /// 获取 Code-Switch 导入状态
    pub async fn get_code_switch_status(&self) -> AppResult<ImportStatus> {
        let mut status = ImportStatus {
            config_path: self.code_switch_path.display().to_string(),
            source: Some(ImportSource::CodeSwitch),
            ..Default::default()
        };

        let config = match self.load_code_switch_config().await? {
            Some(cfg) => {
                status.config_exists = true;
                cfg
            }
            None => {
                debug!("Code-Switch 配置不存在");
                return Ok(status);
            }
        };

        // 计算待导入的 Provider
        let pending_claude = self
            .get_pending_code_switch_providers("claude", &config.claude_providers)
            .await?;
        let pending_codex = self
            .get_pending_code_switch_providers("codex", &config.codex_providers)
            .await?;
        let provider_count = (pending_claude.len() + pending_codex.len()) as i32;

        status.pending_providers = provider_count > 0;
        status.pending_provider_count = provider_count;

        // 计算待导入的 MCP
        let pending_mcp = self
            .get_pending_code_switch_mcp_servers(&config.mcp_servers)
            .await?;
        status.pending_mcp_count = pending_mcp.len() as i32;
        status.pending_mcp = status.pending_mcp_count > 0;

        Ok(status)
    }

    /// 获取待导入的 Code-Switch Provider 列表
    ///
    /// 基于 id 去重：如果已存在相同 id 的 provider，则跳过
    async fn get_pending_code_switch_providers(
        &self,
        kind: &str,
        providers: &[CodeSwitchProvider],
    ) -> AppResult<Vec<ProviderCandidate>> {
        if providers.is_empty() {
            return Ok(Vec::new());
        }

        // 加载现有 Provider
        let existing = self.provider_service.load_providers(kind).await?;
        let existing_ids: std::collections::HashSet<i64> = existing.iter().map(|p| p.id).collect();
        let existing_urls: std::collections::HashSet<_> =
            existing.iter().map(|p| normalize_url(&p.api_url)).collect();

        let mut candidates = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for provider in providers {
            // 基于 id 去重
            if existing_ids.contains(&provider.id) {
                debug!(id = provider.id, name = %provider.name, "Provider 已存在（id 匹配），跳过");
                continue;
            }

            // 避免重复处理
            if seen_ids.contains(&provider.id) {
                continue;
            }

            // 检查 URL 是否已存在
            let url = normalize_url(&provider.api_url);
            if !url.is_empty() && existing_urls.contains(&url) {
                debug!(url = %url, name = %provider.name, "Provider 已存在（URL 匹配），跳过");
                continue;
            }

            seen_ids.insert(provider.id);

            candidates.push(ProviderCandidate {
                name: provider.name.clone(),
                api_url: provider.api_url.clone(),
                api_key: provider.api_key.clone(),
                site: provider.official_site.clone(),
                id: Some(provider.id),
                icon: if provider.icon.is_empty() {
                    None
                } else {
                    Some(provider.icon.clone())
                },
                tint: if provider.tint.is_empty() {
                    None
                } else {
                    Some(provider.tint.clone())
                },
                accent: if provider.accent.is_empty() {
                    None
                } else {
                    Some(provider.accent.clone())
                },
                supported_models: if provider.supported_models.is_empty() {
                    None
                } else {
                    Some(provider.supported_models.clone())
                },
                model_mapping: if provider.model_mapping.is_empty() {
                    None
                } else {
                    Some(provider.model_mapping.clone())
                },
            });
        }

        // 按名称排序
        candidates.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(candidates)
    }

    /// 获取待导入的 Code-Switch MCP 服务器列表
    ///
    /// 基于名称去重：如果已存在同名服务器，则跳过
    async fn get_pending_code_switch_mcp_servers(
        &self,
        servers: &HashMap<String, CodeSwitchMCPServer>,
    ) -> AppResult<Vec<MCPServer>> {
        if servers.is_empty() {
            return Ok(Vec::new());
        }

        let existing = self.mcp_service.list_servers().await?;
        let existing_names: std::collections::HashSet<_> =
            existing.iter().map(|s| normalize_name(&s.name)).collect();

        let mut result = Vec::new();

        for (name, server) in servers {
            let normalized_name = normalize_name(name);
            if normalized_name.is_empty() {
                continue;
            }

            if existing_names.contains(&normalized_name) {
                debug!(name = %name, "MCP 服务器已存在，跳过");
                continue;
            }

            // 确定服务器类型
            let server_type = if !server.server_type.is_empty() {
                MCPServerType::from_str(&server.server_type)
            } else if !server.url.is_empty() {
                MCPServerType::Http
            } else if !server.command.is_empty() {
                MCPServerType::Stdio
            } else {
                warn!(name = %name, "MCP 服务器缺少类型信息，跳过");
                continue;
            };

            // 验证必要字段
            if server_type == MCPServerType::Http && server.url.is_empty() {
                warn!(name = %name, "HTTP 类型 MCP 服务器缺少 URL，跳过");
                continue;
            }
            if server_type == MCPServerType::Stdio && server.command.is_empty() {
                warn!(name = %name, "Stdio 类型 MCP 服务器缺少 command，跳过");
                continue;
            }

            // 转换 enable_platform -> platforms 映射
            let enable_platform = server.enable_platform.clone();
            let enabled_in_claude = enable_platform.iter().any(|p| p == "claude-code");
            let enabled_in_codex = enable_platform.iter().any(|p| p == "codex");

            result.push(MCPServer {
                name: name.clone(),
                server_type,
                command: server.command.trim().to_string(),
                args: server.args.clone(),
                env: server.env.clone(),
                url: server.url.trim().to_string(),
                website: server.website.trim().to_string(),
                tips: server.tips.trim().to_string(),
                enable_platform,
                enabled_in_claude,
                enabled_in_codex,
                ..Default::default()
            });
        }

        result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(result)
    }

    /// 从 Code-Switch 导入配置
    pub async fn import_from_code_switch(&self) -> AppResult<ConfigImportResult> {
        let mut result = ConfigImportResult {
            status: ImportStatus {
                config_path: self.code_switch_path.display().to_string(),
                source: Some(ImportSource::CodeSwitch),
                ..Default::default()
            },
            ..Default::default()
        };

        // 加载配置
        let config = match self.load_code_switch_config().await? {
            Some(cfg) => {
                result.status.config_exists = true;
                cfg
            }
            None => {
                warn!(path = %self.code_switch_path.display(), "Code-Switch 配置不存在");
                return Ok(result);
            }
        };

        // 导入 Provider
        let imported_providers = self.import_code_switch_providers(&config).await?;
        result.imported_providers = imported_providers;

        // 导入 MCP
        let imported_mcp = self.import_code_switch_mcp_servers(&config).await?;
        result.imported_mcp = imported_mcp;

        // 更新状态
        let status = self.get_code_switch_status().await?;
        result.status = status;

        info!(
            providers = imported_providers,
            mcp = imported_mcp,
            "Code-Switch 配置导入完成"
        );
        Ok(result)
    }

    /// 导入 Code-Switch Provider
    async fn import_code_switch_providers(&self, config: &CodeSwitchConfig) -> AppResult<i32> {
        let mut total = 0;

        // 导入 Claude Provider
        let claude_candidates = self
            .get_pending_code_switch_providers("claude", &config.claude_providers)
            .await?;
        if !claude_candidates.is_empty() {
            let added = self
                .save_code_switch_providers("claude", claude_candidates)
                .await?;
            total += added;
        }

        // 导入 Codex Provider
        let codex_candidates = self
            .get_pending_code_switch_providers("codex", &config.codex_providers)
            .await?;
        if !codex_candidates.is_empty() {
            let added = self
                .save_code_switch_providers("codex", codex_candidates)
                .await?;
            total += added;
        }

        Ok(total)
    }

    /// 保存 Code-Switch Provider（保留原始 id、icon、tint、accent 等字段）
    async fn save_code_switch_providers(
        &self,
        kind: &str,
        candidates: Vec<ProviderCandidate>,
    ) -> AppResult<i32> {
        let mut existing = self.provider_service.load_providers(kind).await?;
        let (default_accent, default_tint) = default_visual(kind);

        for candidate in &candidates {
            let provider = Provider {
                // 使用原始 id，或生成新的
                id: candidate
                    .id
                    .unwrap_or_else(|| existing.iter().map(|p| p.id).max().unwrap_or(0) + 1),
                name: candidate.name.clone(),
                api_url: candidate.api_url.clone(),
                api_key: candidate.api_key.clone(),
                site: candidate.site.clone(),
                icon: candidate.icon.clone().unwrap_or_default(),
                tint: candidate
                    .tint
                    .clone()
                    .unwrap_or_else(|| default_tint.clone()),
                accent: candidate
                    .accent
                    .clone()
                    .unwrap_or_else(|| default_accent.clone()),
                enabled: true,
                // Provider 的 supported_models 和 model_mapping 是 Option<HashMap<...>>
                supported_models: candidate.supported_models.clone(),
                model_mapping: candidate.model_mapping.clone(),
                ..Default::default()
            };
            existing.push(provider);
        }

        self.provider_service.save_providers(kind, existing).await?;
        Ok(candidates.len() as i32)
    }

    /// 导入 Code-Switch MCP 服务器
    async fn import_code_switch_mcp_servers(&self, config: &CodeSwitchConfig) -> AppResult<i32> {
        let candidates = self
            .get_pending_code_switch_mcp_servers(&config.mcp_servers)
            .await?;
        if candidates.is_empty() {
            return Ok(0);
        }

        let mut existing = self.mcp_service.list_servers().await?;
        let count = candidates.len() as i32;
        existing.extend(candidates);

        self.mcp_service.save_servers(existing).await?;
        Ok(count)
    }
}

// === 辅助函数 ===

fn normalize_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_lowercase()
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

fn pick_first_non_empty(values: &[Option<&String>]) -> String {
    for value in values {
        if let Some(v) = value {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    String::new()
}

fn default_visual(kind: &str) -> (String, String) {
    match kind.to_lowercase().as_str() {
        "codex" => (
            "#ec4899".to_string(),
            "rgba(236, 72, 153, 0.16)".to_string(),
        ),
        _ => ("#0a84ff".to_string(), "rgba(15, 23, 42, 0.12)".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            normalize_url("https://api.example.com/"),
            "https://api.example.com"
        );
        assert_eq!(normalize_url("  HTTPS://API.COM  "), "https://api.com");
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("  Test Provider  "), "test provider");
    }

    #[test]
    fn test_pick_first_non_empty() {
        let a = "first".to_string();
        let b = "".to_string();
        assert_eq!(pick_first_non_empty(&[None, Some(&b), Some(&a)]), "first");
        assert_eq!(pick_first_non_empty(&[None, None]), "");
    }

    #[test]
    fn test_default_visual() {
        let (accent, _tint) = default_visual("codex");
        assert_eq!(accent, "#ec4899");

        let (accent, _tint) = default_visual("claude");
        assert_eq!(accent, "#0a84ff");
    }

    #[test]
    fn test_parse_cc_switch_config() {
        let json = r#"{
            "claude": {
                "providers": {
                    "test": {
                        "name": "Test Provider",
                        "settingsConfig": {
                            "env": {
                                "ANTHROPIC_BASE_URL": "https://api.test.com",
                                "ANTHROPIC_AUTH_TOKEN": "sk-test"
                            }
                        }
                    }
                }
            },
            "codex": {},
            "mcp": {
                "claude": {
                    "servers": {
                        "test-mcp": {
                            "name": "Test MCP",
                            "enabled": true,
                            "server": {
                                "type": "stdio",
                                "command": "npx",
                                "args": ["-y", "test"]
                            }
                        }
                    }
                },
                "codex": {}
            }
        }"#;

        let config: CCSwitchConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.claude.providers.len(), 1);
        assert!(config.claude.providers.contains_key("test"));

        let mcp_servers = &config.mcp.claude.servers;
        assert_eq!(mcp_servers.len(), 1);
        assert!(mcp_servers.contains_key("test-mcp"));
    }

    // ============================================================
    // Code-Switch 相关测试
    // ============================================================

    #[test]
    fn test_parse_code_switch_provider_file() {
        // 测试 claude-code.json / codex.json 格式解析
        let json = r##"{
            "providers": [
                {
                    "id": 1763435071814,
                    "name": "Test Provider",
                    "apiUrl": "https://api.example.com",
                    "apiKey": "sk-test-key",
                    "officialSite": "https://example.com",
                    "icon": "test-icon",
                    "tint": "rgba(0, 0, 0, 0.1)",
                    "accent": "#0a84ff",
                    "enabled": true,
                    "supportedModels": {
                        "claude-sonnet-4-20250514": true,
                        "claude-*": true
                    },
                    "modelMapping": {
                        "claude-sonnet-4-20250514": "claude-sonnet-latest"
                    }
                }
            ]
        }"##;

        let file: CodeSwitchProviderFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.providers.len(), 1);

        let provider = &file.providers[0];
        assert_eq!(provider.id, 1763435071814);
        assert_eq!(provider.name, "Test Provider");
        assert_eq!(provider.api_url, "https://api.example.com");
        assert_eq!(provider.api_key, "sk-test-key");
        assert_eq!(provider.official_site, "https://example.com");
        assert_eq!(provider.icon, "test-icon");
        assert!(provider.enabled);
        assert_eq!(provider.supported_models.len(), 2);
        assert_eq!(provider.model_mapping.len(), 1);
    }

    #[test]
    fn test_parse_code_switch_provider_with_defaults() {
        // 测试可选字段的默认值
        let json = r#"{
            "providers": [
                {
                    "id": 123,
                    "name": "Minimal Provider",
                    "apiUrl": "https://api.minimal.com",
                    "apiKey": "sk-minimal"
                }
            ]
        }"#;

        let file: CodeSwitchProviderFile = serde_json::from_str(json).unwrap();
        let provider = &file.providers[0];

        assert_eq!(provider.official_site, "");
        assert_eq!(provider.icon, "");
        assert_eq!(provider.tint, "");
        assert_eq!(provider.accent, "");
        assert!(!provider.enabled); // 默认 false
        assert!(provider.supported_models.is_empty());
        assert!(provider.model_mapping.is_empty());
    }

    #[test]
    fn test_parse_code_switch_mcp_server() {
        // 测试 mcp.json 格式解析
        let json = r#"{
            "filesystem": {
                "type": "stdio",
                "command": "npx",
                "args": ["-y", "@anthropic/mcp-server-filesystem"],
                "env": {"HOME": "/Users/test"},
                "enable_platform": ["claude-code", "codex"]
            },
            "remote-api": {
                "type": "http",
                "url": "https://mcp.example.com",
                "website": "https://example.com",
                "tips": "Get API key from settings"
            }
        }"#;

        let servers: HashMap<String, CodeSwitchMCPServer> = serde_json::from_str(json).unwrap();
        assert_eq!(servers.len(), 2);

        // 验证 stdio 类型服务器
        let fs_server = servers.get("filesystem").unwrap();
        assert_eq!(fs_server.server_type, "stdio");
        assert_eq!(fs_server.command, "npx");
        assert_eq!(fs_server.args.len(), 2);
        assert_eq!(fs_server.env.get("HOME"), Some(&"/Users/test".to_string()));
        assert!(fs_server
            .enable_platform
            .contains(&"claude-code".to_string()));
        assert!(fs_server.enable_platform.contains(&"codex".to_string()));

        // 验证 http 类型服务器
        let http_server = servers.get("remote-api").unwrap();
        assert_eq!(http_server.server_type, "http");
        assert_eq!(http_server.url, "https://mcp.example.com");
        assert_eq!(http_server.website, "https://example.com");
        assert_eq!(http_server.tips, "Get API key from settings");
    }

    #[test]
    fn test_parse_code_switch_mcp_server_with_defaults() {
        // 测试 MCP 服务器的可选字段
        let json = r#"{
            "minimal": {
                "command": "python"
            }
        }"#;

        let servers: HashMap<String, CodeSwitchMCPServer> = serde_json::from_str(json).unwrap();
        let server = servers.get("minimal").unwrap();

        assert_eq!(server.server_type, ""); // 默认空
        assert_eq!(server.command, "python");
        assert!(server.args.is_empty());
        assert!(server.env.is_empty());
        assert!(server.enable_platform.is_empty());
    }

    #[test]
    fn test_empty_code_switch_provider_file() {
        // 测试空文件
        let json = r#"{"providers": []}"#;
        let file: CodeSwitchProviderFile = serde_json::from_str(json).unwrap();
        assert!(file.providers.is_empty());

        // 测试完全空的 JSON
        let json = r#"{}"#;
        let file: CodeSwitchProviderFile = serde_json::from_str(json).unwrap();
        assert!(file.providers.is_empty());
    }

    #[test]
    fn test_empty_code_switch_mcp_file() {
        // 测试空 MCP 文件
        let json = r#"{}"#;
        let servers: HashMap<String, CodeSwitchMCPServer> = serde_json::from_str(json).unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn test_code_switch_provider_field_mapping() {
        // 验证 camelCase → snake_case 字段映射
        let json = r#"{
            "providers": [{
                "id": 1,
                "name": "Test",
                "apiUrl": "https://api.test.com",
                "apiKey": "key",
                "officialSite": "https://test.com",
                "supportedModels": {"model1": true},
                "modelMapping": {"external": "internal"}
            }]
        }"#;

        let file: CodeSwitchProviderFile = serde_json::from_str(json).unwrap();
        let p = &file.providers[0];

        // 验证 serde rename 正确工作
        assert_eq!(p.api_url, "https://api.test.com"); // apiUrl -> api_url
        assert_eq!(p.api_key, "key"); // apiKey -> api_key
        assert_eq!(p.official_site, "https://test.com"); // officialSite -> official_site
        assert!(p.supported_models.contains_key("model1")); // supportedModels -> supported_models
        assert!(p.model_mapping.contains_key("external")); // modelMapping -> model_mapping
    }

    #[test]
    fn test_code_switch_mcp_platform_mapping() {
        // 验证 enable_platform 字段正确解析
        let json = r#"{
            "test-server": {
                "type": "stdio",
                "command": "test",
                "enable_platform": ["claude-code"]
            }
        }"#;

        let servers: HashMap<String, CodeSwitchMCPServer> = serde_json::from_str(json).unwrap();
        let server = servers.get("test-server").unwrap();

        assert!(server.enable_platform.contains(&"claude-code".to_string()));
        assert!(!server.enable_platform.contains(&"codex".to_string()));
    }
}
