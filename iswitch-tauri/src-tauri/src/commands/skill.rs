//! [INPUT]:
//!   source: ../../../openspec/changes/migrate-to-tauri-stack/design.md ([POS]: Command 定义)
//!   source: ../services/skill_service.rs ([POS]: 调用 Service)
//!
//! [OUTPUT]:
//!   - list_skills
//!   - install_skill
//!   - uninstall_skill
//!   - list_skill_repos
//!   - add_skill_repo
//!   - remove_skill_repo
//!
//! [POS]: Skill 模块的前端调用接口
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::models::{Skill, SkillInstallRequest, SkillRepoConfig};
use crate::services::SkillService;
use tauri::State;
use tracing::info;

/// 列出所有 Skills
#[tauri::command]
pub async fn list_skills(service: State<'_, SkillService>) -> AppResult<Vec<Skill>> {
    service.list_skills().await
}

/// 安装 Skill
#[tauri::command]
pub async fn install_skill(
    req: SkillInstallRequest,
    service: State<'_, SkillService>,
) -> AppResult<()> {
    info!(repo = %req.repo_name, dir = %req.directory, "正在安装 Skill");
    service.install_skill(req).await
}

/// 卸载 Skill
#[tauri::command]
pub async fn uninstall_skill(key: String, service: State<'_, SkillService>) -> AppResult<()> {
    info!(key = %key, "正在卸载 Skill");
    service.uninstall_skill(key).await
}

/// 列出 Skill 仓库列表
#[tauri::command]
pub async fn list_skill_repos(service: State<'_, SkillService>) -> AppResult<Vec<SkillRepoConfig>> {
    service.list_repos().await
}

/// 添加 Skill 仓库
#[tauri::command]
pub async fn add_skill_repo(
    repo: SkillRepoConfig,
    service: State<'_, SkillService>,
) -> AppResult<()> {
    info!(owner = %repo.owner, name = %repo.name, "添加 Skill 仓库");
    service.add_repo(repo).await
}

/// 移除 Skill 仓库
#[tauri::command]
pub async fn remove_skill_repo(
    repo: SkillRepoConfig,
    service: State<'_, SkillService>,
) -> AppResult<()> {
    info!(owner = %repo.owner, name = %repo.name, "移除 Skill 仓库");
    service.remove_repo(repo).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::Manager;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_list_skills_command() {
        let dir = tempdir().unwrap();

        // 创建空的 skill_repos.json 以避免使用默认仓库（需要网络请求）
        let repos_path = dir.path().join("skill_repos.json");
        std::fs::write(&repos_path, "[]").unwrap();

        let service = SkillService::with_root(dir.path().to_path_buf());

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(service);
        let state = app.state::<SkillService>();

        // Initially no skills (empty repos list means no remote skills to fetch)
        let result = list_skills(state).await.unwrap();
        assert!(
            result.is_empty(),
            "Expected empty skills list, got {} skills",
            result.len()
        );
    }

    #[tokio::test]
    async fn test_repo_management_commands() {
        let dir = tempdir().unwrap();
        let service = SkillService::with_root(dir.path().to_path_buf());

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(service);
        let state = app.state::<SkillService>();

        let repo = SkillRepoConfig {
            owner: "test".to_string(),
            name: "repo".to_string(),
            branch: "main".to_string(),
            enabled: true,
        };

        // Add
        add_skill_repo(repo.clone(), state.clone()).await.unwrap();

        // List
        let repos = list_skill_repos(state.clone()).await.unwrap();
        assert!(repos.iter().any(|r| r.name == "repo"));

        // Remove
        remove_skill_repo(repo, state).await.unwrap();
        let repos = list_skill_repos(app.state::<SkillService>()).await.unwrap();
        assert!(!repos.iter().any(|r| r.name == "repo"));
    }
}
