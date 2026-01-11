//! [INPUT]:
//!   source: ../../../../code-switch/services/skillservice.go ([POS]: 原 Go Skill 数据模型)
//!   source: ../../../openspec/changes/migrate-to-tauri-stack/design.md ([POS]: Rust 数据结构规范)
//!
//! [OUTPUT]:
//!   - Skill 结构体
//!   - SkillRepo 结构体
//!   - SkillStore 存储结构
//!
//! [POS]: Skill 管理数据模型定义，用于从 GitHub 仓库安装和管理 Claude Skills
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill 配置
///
/// 对应 Go 版本的 Skill 结构体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Skill {
    /// 唯一标识符（格式：owner/repo:directory 或 local:directory）
    pub key: String,

    /// Skill 名称
    pub name: String,

    /// Skill 描述
    #[serde(default)]
    pub description: String,

    /// 目录名称
    pub directory: String,

    /// README 链接
    #[serde(default)]
    pub readme_url: String,

    /// 是否已安装
    #[serde(default)]
    pub installed: bool,

    /// 来源仓库 Owner
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub repo_owner: String,

    /// 来源仓库名称
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub repo_name: String,

    /// 来源仓库分支
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub repo_branch: String,
}

impl Skill {
    /// 构建 Skill key
    pub fn build_key(owner: &str, name: &str, directory: &str) -> String {
        let owner = owner.to_lowercase().trim().to_string();
        let name = name.to_lowercase().trim().to_string();
        let directory = directory.to_lowercase();

        if owner.is_empty() && name.is_empty() {
            format!("local:{}", directory)
        } else {
            format!("{}/{}:{}", owner, name, directory)
        }
    }
}

/// Skill 元数据（从 SKILL.md 解析）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillMetadata {
    /// Skill 名称
    pub name: String,

    /// Skill 描述
    pub description: String,
}

/// Skill 状态信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillState {
    /// 是否已安装
    pub installed: bool,

    /// 安装时间
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installed_at: Option<DateTime<Utc>>,
}

/// Skill 仓库配置
///
/// 对应 Go 版本的 skillRepoConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRepoConfig {
    /// GitHub 用户/组织名
    pub owner: String,

    /// 仓库名称
    pub name: String,

    /// 分支名称
    #[serde(default = "default_branch")]
    pub branch: String,

    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for SkillRepoConfig {
    fn default() -> Self {
        Self {
            owner: String::new(),
            name: String::new(),
            branch: "main".to_string(),
            enabled: true,
        }
    }
}

impl SkillRepoConfig {
    /// 规范化仓库配置
    pub fn normalize(&mut self) {
        self.owner = self.owner.trim().to_string();
        self.name = self.name.trim().to_string();
        self.branch = self.branch.trim().to_string();

        if self.branch.is_empty() {
            self.branch = "main".to_string();
        }
    }

    /// 检查两个仓库配置是否相等（忽略大小写）
    pub fn equals(&self, other: &Self) -> bool {
        self.owner.eq_ignore_ascii_case(&other.owner) && self.name.eq_ignore_ascii_case(&other.name)
    }

    /// 构建仓库 ZIP 下载 URL
    pub fn zip_url(&self) -> String {
        format!(
            "https://github.com/{}/{}/archive/refs/heads/{}.zip",
            self.owner, self.name, self.branch
        )
    }

    /// 构建仓库 URL
    pub fn repo_url(&self, directory: Option<&str>) -> String {
        match directory {
            Some(dir) if !dir.is_empty() => {
                format!(
                    "https://github.com/{}/{}/tree/{}/{}",
                    self.owner,
                    self.name,
                    self.branch,
                    dir.trim_matches('/')
                )
            }
            _ => format!("https://github.com/{}/{}", self.owner, self.name),
        }
    }
}

/// Skill 存储文件结构
///
/// 对应 Go 版本的 skillStore
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillStore {
    /// 已安装的 Skills 状态
    #[serde(default)]
    pub skills: HashMap<String, SkillState>,

    /// 配置的仓库列表
    #[serde(default)]
    pub repos: Vec<SkillRepoConfig>,
}

impl SkillStore {
    /// 确保仓库列表已初始化
    pub fn ensure_repos(&mut self) {
        if self.repos.is_empty() {
            self.repos = default_skill_repos();
        }

        for repo in &mut self.repos {
            repo.normalize();
        }
    }
}

/// 安装请求
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillInstallRequest {
    /// 要安装的目录名
    pub directory: String,

    /// 指定仓库 Owner（可选）
    #[serde(default)]
    pub repo_owner: String,

    /// 指定仓库名称（可选）
    #[serde(default)]
    pub repo_name: String,

    /// 指定分支（可选）
    #[serde(default)]
    pub repo_branch: String,
}

/// 默认 Skill 仓库列表
pub fn default_skill_repos() -> Vec<SkillRepoConfig> {
    vec![
        SkillRepoConfig {
            owner: "ComposioHQ".to_string(),
            name: "awesome-claude-skills".to_string(),
            branch: "main".to_string(),
            enabled: true,
        },
        SkillRepoConfig {
            owner: "anthropics".to_string(),
            name: "skills".to_string(),
            branch: "main".to_string(),
            enabled: true,
        },
    ]
}

/// 默认尝试的分支列表
pub fn default_repo_branches() -> Vec<&'static str> {
    vec!["main", "master"]
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_build_key() {
        assert_eq!(
            Skill::build_key("anthropics", "skills", "my-skill"),
            "anthropics/skills:my-skill"
        );
        assert_eq!(Skill::build_key("", "", "local-skill"), "local:local-skill");
    }

    #[test]
    fn test_skill_repo_config() {
        let mut repo = SkillRepoConfig {
            owner: " ComposioHQ ".to_string(),
            name: "awesome-claude-skills".to_string(),
            branch: "".to_string(),
            enabled: true,
        };

        repo.normalize();

        assert_eq!(repo.owner, "ComposioHQ");
        assert_eq!(repo.branch, "main");

        let url = repo.zip_url();
        assert!(url.contains("github.com"));
        assert!(url.contains("ComposioHQ"));
    }

    #[test]
    fn test_skill_repo_equals() {
        let repo1 = SkillRepoConfig {
            owner: "anthropics".to_string(),
            name: "skills".to_string(),
            ..Default::default()
        };

        let repo2 = SkillRepoConfig {
            owner: "ANTHROPICS".to_string(),
            name: "SKILLS".to_string(),
            branch: "dev".to_string(),
            enabled: false,
        };

        assert!(repo1.equals(&repo2));
    }

    #[test]
    fn test_default_skill_repos() {
        let repos = default_skill_repos();
        assert_eq!(repos.len(), 2);
        assert!(repos.iter().any(|r| r.owner == "anthropics"));
    }

    #[test]
    fn test_skill_json() {
        let json = r#"{
            "key": "anthropics/skills:code-review",
            "name": "Code Review",
            "description": "Review code changes",
            "directory": "code-review",
            "installed": true
        }"#;

        let skill: Skill = serde_json::from_str(json).unwrap();
        assert_eq!(skill.name, "Code Review");
        assert!(skill.installed);
    }
}
