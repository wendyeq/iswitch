//! [INPUT]:
//!   source: ../../../../code-switch/services/skillservice.go ([POS]: 原 Go 实现参考)
//!   source: ../models/skill.rs ([POS]: Skill 数据模型)
//!
//! [OUTPUT]:
//!   - SkillService 结构体
//!   - list_skills(), install_skill(), uninstall_skill() API
//!   - 仓库管理 API
//!
//! [POS]: Skill 管理服务，负责 Skill 的发现、安装和管理
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::{AppError, AppResult};
use crate::models::{
    default_skill_repos, Skill, SkillInstallRequest, SkillMetadata, SkillRepoConfig,
};
use crate::utils::paths::{ensure_dir, iswitch_dir};
use crate::utils::security::secure_write;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tracing::{info, warn};
use zip::ZipArchive;

/// Skill 服务
pub struct SkillService {
    http_client: Client,
    root_dir: PathBuf,
}

impl SkillService {
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .user_agent("iswitch/1.0")
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            root_dir: iswitch_dir(),
        }
    }

    #[cfg(test)]
    pub fn with_root(root_dir: PathBuf) -> Self {
        Self {
            http_client: Client::builder().build().unwrap_or_default(),
            root_dir,
        }
    }

    /// 列出所有 Skills (已安装 + 仓库中可用)
    pub async fn list_skills(&self) -> AppResult<Vec<Skill>> {
        let mut all_skills = HashMap::new();

        // 1. 获取配置的仓库列表
        let repos = self.list_repos().await?;

        // 2. 并行获取仓库中的 Skills (available)
        // 为简化实现，暂时串行获取 (TODO: 优化为并行)
        for repo in repos {
            if !repo.enabled {
                continue;
            }

            match self.fetch_repo_skills(&repo).await {
                Ok(skills) => {
                    for skill in skills {
                        all_skills.insert(skill.key.clone(), skill);
                    }
                }
                Err(e) => {
                    warn!(repo = %repo.name, error = %e, "获取仓库 Skills 失败");
                }
            }
        }

        // 3. 扫描本地已安装的 Skills (覆盖状态)
        let installed_skills = self.scan_installed_skills().await?;
        for skill in installed_skills {
            all_skills.insert(skill.key.clone(), skill);
        }

        let mut result: Vec<Skill> = all_skills.into_values().collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(result)
    }

    /// 安装 Skill
    pub async fn install_skill(&self, req: SkillInstallRequest) -> AppResult<()> {
        let repo = SkillRepoConfig {
            owner: req.repo_owner.clone(),
            name: req.repo_name.clone(),
            branch: req.repo_branch.clone(), // Should be set if known, else default
            enabled: true,
        };

        // 1. 确保缓存中有仓库 ZIP
        let zip_path = self.ensure_repo_cache(&repo).await?;

        // 2. 解压指定目录
        let target_dir =
            self.get_skill_install_path(&req.repo_owner, &req.repo_name, &req.directory);
        ensure_dir(&target_dir).map_err(|e| AppError::DirCreate {
            path: target_dir.display().to_string(),
            source: e,
        })?;

        let zip_content = fs::read(&zip_path)
            .await
            .map_err(|e| AppError::ConfigRead {
                path: zip_path.display().to_string(),
                source: e,
            })?;

        // Blocking ZIP operation
        let target_dir_clone = target_dir.clone();
        let directory_name = req.directory.clone();

        tokio::task::spawn_blocking(move || -> AppResult<()> {
            let reader = Cursor::new(zip_content);
            let mut archive = ZipArchive::new(reader)
                .map_err(|e| AppError::Internal(format!("ZIP error: {}", e)))?;

            // ZIP structure is usually: `repo-branch/directory/file`
            // We need to find `directory/` and extract it to `target_dir`

            let mut found = false;

            for i in 0..archive.len() {
                let mut file = archive
                    .by_index(i)
                    .map_err(|e| AppError::Internal(format!("ZIP read error: {}", e)))?;
                let path = file.mangled_name();

                // Check if file is inside the requested directory
                // Path format in ZIP: "repo-branch/requested_dir/file"
                let components: Vec<_> = path
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect();

                if components.len() > 1 {
                    // components[0] is root (repo-branch)
                    // components[1] check if it matches requested directory
                    if components[1] == directory_name {
                        found = true;
                        // Construct dest path: target_dir / (relative path from directory)
                        let mut dest_path = target_dir_clone.clone();
                        for component in components.iter().skip(2) {
                            dest_path.push(component.as_ref());
                        }

                        if file.is_dir() {
                            std::fs::create_dir_all(&dest_path).map_err(|e| {
                                AppError::DirCreate {
                                    path: dest_path.display().to_string(),
                                    source: e,
                                }
                            })?;
                        } else {
                            if let Some(parent) = dest_path.parent() {
                                std::fs::create_dir_all(parent).map_err(|e| {
                                    AppError::DirCreate {
                                        path: parent.display().to_string(),
                                        source: e,
                                    }
                                })?;
                            }
                            let mut outfile = std::fs::File::create(&dest_path).map_err(|e| {
                                AppError::ConfigWrite {
                                    path: dest_path.display().to_string(),
                                    source: e,
                                }
                            })?;
                            std::io::copy(&mut file, &mut outfile).map_err(|e| {
                                AppError::ConfigWrite {
                                    path: dest_path.display().to_string(),
                                    source: e,
                                }
                            })?;
                        }
                    }
                }
            }

            if !found {
                return Err(AppError::Internal(format!(
                    "Directory '{}' not found in repo",
                    directory_name
                )));
            }

            Ok(())
        })
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

        info!(key = %format!("{}/{}", req.repo_name, req.directory), "Skill 已安装");
        Ok(())
    }

    /// 卸载 Skill
    pub async fn uninstall_skill(&self, key: String) -> AppResult<()> {
        // key format: owner/repo:directory
        // parse key to path
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() != 2 {
            return Err(AppError::InvalidArgument(
                "Invalid skill key format".to_string(),
            ));
        }

        let path_parts: Vec<&str> = parts[0].split('/').collect();
        if path_parts.len() != 2 {
            return Err(AppError::InvalidArgument(
                "Invalid repo format in key".to_string(),
            ));
        }

        let owner = path_parts[0];
        let repo = path_parts[1];
        let directory = parts[1];

        let install_path = self.get_skill_install_path(owner, repo, directory);

        if install_path.exists() {
            fs::remove_dir_all(&install_path)
                .await
                .map_err(|e| AppError::ConfigWrite {
                    path: install_path.display().to_string(),
                    source: e,
                })?;
            info!(key = %key, "Skill 已卸载");
        }

        Ok(())
    }

    /// 列出配置的仓库
    pub async fn list_repos(&self) -> AppResult<Vec<SkillRepoConfig>> {
        let path = self.root_dir.join("skill_repos.json");
        if !path.exists() {
            return Ok(default_skill_repos());
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::ConfigRead {
                path: path.display().to_string(),
                source: e,
            })?;

        if content.trim().is_empty() {
            return Ok(default_skill_repos());
        }

        Ok(serde_json::from_str(&content)?)
    }

    /// 添加仓库
    pub async fn add_repo(&self, mut repo: SkillRepoConfig) -> AppResult<()> {
        repo.normalize();
        let mut repos = self.list_repos().await?;

        // Check duplicate
        if repos.iter().any(|r| r.equals(&repo)) {
            return Ok(());
        }

        repos.push(repo);
        self.save_repos_file(repos).await
    }

    /// 移除仓库
    pub async fn remove_repo(&self, current_repo: SkillRepoConfig) -> AppResult<()> {
        let repos = self.list_repos().await?;
        let new_repos: Vec<SkillRepoConfig> = repos
            .into_iter()
            .filter(|r| !r.equals(&current_repo))
            .collect();

        self.save_repos_file(new_repos).await
    }

    // --- Private Helpers ---

    async fn save_repos_file(&self, repos: Vec<SkillRepoConfig>) -> AppResult<()> {
        let path = self.root_dir.join("skill_repos.json");

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AppError::DirCreate {
                        path: parent.display().to_string(),
                        source: e,
                    })?;
            }
        }
        let content = serde_json::to_string_pretty(&repos)?;
        secure_write(&path, content.as_bytes())?;
        Ok(())
    }

    async fn fetch_repo_skills(&self, repo: &SkillRepoConfig) -> AppResult<Vec<Skill>> {
        let zip_path = self.ensure_repo_cache(repo).await?;
        let zip_content = fs::read(&zip_path)
            .await
            .map_err(|e| AppError::ConfigRead {
                path: zip_path.display().to_string(),
                source: e,
            })?;

        let repo_clone = repo.clone();

        // Parse ZIP in blocking thread
        let skills = tokio::task::spawn_blocking(move || -> AppResult<Vec<Skill>> {
            let reader = Cursor::new(zip_content);
            let mut archive = ZipArchive::new(reader)
                .map_err(|e| AppError::Internal(format!("ZIP error: {}", e)))?;

            let mut skills = Vec::new();
            let mut directories = HashSet::new();

            // 1. Identify top-level directories under root
            for i in 0..archive.len() {
                let file = archive.by_index(i).ok();
                if let Some(f) = file {
                    let path = f.mangled_name();
                    let components: Vec<_> = path
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy())
                        .collect();
                    // Structure: root/directory/...
                    if components.len() >= 2 {
                        let dir_name = components[1].to_string();
                        // Ignore hidden files or common non-skill dirs
                        if !dir_name.starts_with('.') && dir_name != "src" && dir_name != "test" {
                            directories.insert(dir_name);
                        }
                    }
                }
            }

            // 2. Check for skill.yaml in each directory
            for dir in directories {
                let key = Skill::build_key(&repo_clone.owner, &repo_clone.name, &dir);

                let skill = Skill {
                    key: key.clone(),
                    name: dir.clone(), // Default to dir name
                    description: "".to_string(),
                    directory: dir.clone(),
                    readme_url: repo_clone.repo_url(Some(&dir)),
                    installed: false,
                    repo_owner: repo_clone.owner.clone(),
                    repo_name: repo_clone.name.clone(),
                    repo_branch: repo_clone.branch.clone(),
                };

                // Try to find skill.yaml in ZIP?
                // Too complex for now without full extraction or multi-pass.
                // However, we can try to guess or just leave it empty.
                // Users usually see directory name.

                skills.push(skill);
            }
            Ok(skills)
        })
        .await
        .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

        Ok(skills)
    }

    async fn ensure_repo_cache(&self, repo: &SkillRepoConfig) -> AppResult<PathBuf> {
        let cache_dir = self.root_dir.join("cache");
        ensure_dir(&cache_dir).map_err(|e| AppError::DirCreate {
            path: cache_dir.display().to_string(),
            source: e,
        })?;

        let filename = format!("{}-{}-{}.zip", repo.owner, repo.name, repo.branch);
        let path = cache_dir.join(filename);

        if path.exists() {
            return Ok(path);
        }

        // Download
        let url = repo.zip_url();
        info!(url = %url, "下载仓库 ZIP");

        let resp = self.http_client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(AppError::Internal(format!(
                "Download failed: {}",
                resp.status()
            )));
        }

        let bytes = resp.bytes().await?;
        secure_write(&path, &bytes)?;

        Ok(path)
    }

    async fn scan_installed_skills(&self) -> AppResult<Vec<Skill>> {
        let base_dir = self.root_dir.join("skills");
        if !base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();

        // Structure: base_dir/owner/repo/directory
        let mut owners = fs::read_dir(&base_dir)
            .await
            .map_err(|_| AppError::Internal("Read dir failed".into()))?;
        while let Ok(Some(owner_entry)) = owners.next_entry().await {
            if !owner_entry.file_type().await?.is_dir() {
                continue;
            }
            let owner = owner_entry.file_name().to_string_lossy().to_string();

            let repo_path = owner_entry.path();
            let mut repos = fs::read_dir(&repo_path)
                .await
                .map_err(|_| AppError::Internal("Read dir failed".into()))?;

            while let Ok(Some(repo_entry)) = repos.next_entry().await {
                if !repo_entry.file_type().await?.is_dir() {
                    continue;
                }
                let repo = repo_entry.file_name().to_string_lossy().to_string();

                let dir_path = repo_entry.path();
                let mut dirs = fs::read_dir(&dir_path)
                    .await
                    .map_err(|_| AppError::Internal("Read dir failed".into()))?;

                while let Ok(Some(dir_entry)) = dirs.next_entry().await {
                    if !dir_entry.file_type().await?.is_dir() {
                        continue;
                    }
                    let directory = dir_entry.file_name().to_string_lossy().to_string();

                    let key = Skill::build_key(&owner, &repo, &directory);

                    // Check for skill.yaml locally?
                    let mut skill = Skill {
                        key,
                        name: directory.clone(),
                        description: "(Installed)".to_string(),
                        directory: directory.clone(),
                        readme_url: "".to_string(),
                        installed: true,
                        repo_owner: owner.clone(),
                        repo_name: repo.clone(),
                        repo_branch: "unknown".to_string(),
                    };

                    // Try reading skill.yaml
                    let yaml_path = dir_entry.path().join("skill.yaml");
                    if yaml_path.exists() {
                        if let Ok(content) = fs::read_to_string(&yaml_path).await {
                            // Use serde_yaml
                            if let Ok(meta) = serde_yaml::from_str::<SkillMetadata>(&content) {
                                skill.name = meta.name;
                                skill.description = meta.description;
                            }
                        }
                    }

                    skills.push(skill);
                }
            }
        }

        Ok(skills)
    }

    fn get_skill_install_path(&self, owner: &str, repo: &str, directory: &str) -> PathBuf {
        self.root_dir
            .join("skills")
            .join(owner)
            .join(repo)
            .join(directory)
    }
}

impl Default for SkillService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::FileOptions;

    fn create_dummy_zip(path: &PathBuf, structure: Vec<(&str, &str)>) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        let options =
            FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

        for (name, content) in structure {
            if name.ends_with('/') {
                zip.add_directory(name, options).unwrap();
            } else {
                zip.start_file(name, options).unwrap();
                zip.write_all(content.as_bytes()).unwrap();
            }
        }
        zip.finish().unwrap();
    }

    #[test]
    fn test_skill_metadata_parsing() {
        let yaml = r#"
name: Code Review
description: Analyze code changes
"#;
        let meta: SkillMetadata = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(meta.name, "Code Review");
        assert_eq!(meta.description, "Analyze code changes");
    }

    #[tokio::test]
    async fn test_install_skill_success() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root.clone()); // Fixed name

        // 1. Prepare Request
        let req = SkillInstallRequest {
            repo_owner: "test-owner".to_string(),
            repo_name: "test-repo".to_string(),
            repo_branch: "main".to_string(),
            directory: "my-skill".to_string(),
        };

        // 2. Pre-seed Cache with ZIP
        // Cache path: root/cache/test-owner-test-repo-main.zip
        let cache_dir = root.join("cache");
        std::fs::create_dir_all(&cache_dir).unwrap();
        let zip_path = cache_dir.join("test-owner-test-repo-main.zip");

        // ZIP Structure: test-repo-main/my-skill/skill.yaml
        create_dummy_zip(
            &zip_path,
            vec![
                ("test-repo-main/", ""),
                ("test-repo-main/my-skill/", ""),
                ("test-repo-main/my-skill/skill.yaml", "name: My Skill\n"),
                ("test-repo-main/my-skill/src/index.js", "console.log('hi')"),
                ("test-repo-main/other-skill/skill.yaml", "name: Other\n"),
            ],
        );

        // 3. Install
        service.install_skill(req).await.unwrap();

        // 4. Verify Installation
        // Path: root/skills/test-owner/test-repo/my-skill/skill.yaml
        let installed_file = root
            .join("skills")
            .join("test-owner")
            .join("test-repo")
            .join("my-skill")
            .join("skill.yaml");

        assert!(installed_file.exists());

        let content = std::fs::read_to_string(installed_file).unwrap();
        assert_eq!(content, "name: My Skill\n");

        // Verify other files
        let src_file = root
            .join("skills")
            .join("test-owner")
            .join("test-repo")
            .join("my-skill")
            .join("src")
            .join("index.js");
        assert!(src_file.exists());

        // Verify isolation (other-skill should NOT be installed)
        let other_file = root
            .join("skills")
            .join("test-owner")
            .join("test-repo")
            .join("other-skill");
        assert!(!other_file.exists());
    }

    #[tokio::test]
    async fn test_install_skill_not_found() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root.clone());

        let req = SkillInstallRequest {
            repo_owner: "test-owner".to_string(),
            repo_name: "test-repo".to_string(),
            repo_branch: "main".to_string(),
            directory: "missing-skill".to_string(),
        };

        let cache_dir = root.join("cache");
        std::fs::create_dir_all(&cache_dir).unwrap();
        let zip_path = cache_dir.join("test-owner-test-repo-main.zip");

        create_dummy_zip(
            &zip_path,
            vec![
                ("test-repo-main/", ""),
                ("test-repo-main/existing-skill/", ""),
            ],
        );

        let result = service.install_skill(req).await;
        assert!(result.is_err());
        match result {
            Err(AppError::Internal(msg)) => assert!(msg.contains("not found in repo")),
            _ => panic!("Expected AppError::Internal"),
        }
    }

    // ===== 卸载测试 =====

    /// 测试卸载已安装的 Skill
    #[tokio::test]
    async fn test_uninstall_skill_success() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root.clone());

        // 创建模拟已安装的 Skill
        let skill_path = root
            .join("skills")
            .join("owner")
            .join("repo")
            .join("skill-name");
        std::fs::create_dir_all(&skill_path).unwrap();
        let skill_yaml = skill_path.join("skill.yaml");
        std::fs::write(&skill_yaml, "name: Test\n").unwrap();

        // 验证存在
        assert!(skill_path.exists());

        // 卸载
        let key = "owner/repo:skill-name".to_string();
        service.uninstall_skill(key).await.unwrap();

        // 验证已删除
        assert!(!skill_path.exists());
    }

    /// 测试卸载不存在的 Skill（应该成功）
    #[tokio::test]
    async fn test_uninstall_nonexistent_skill() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root.clone());

        let key = "owner/repo:nonexistent".to_string();
        let result = service.uninstall_skill(key).await;
        // 不存在的 Skill 卸载应该成功（幂等）
        assert!(result.is_ok());
    }

    /// 测试无效的 key 格式
    #[tokio::test]
    async fn test_uninstall_invalid_key_format() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root);

        // 缺少冒号
        let result = service.uninstall_skill("invalid-key".to_string()).await;
        assert!(result.is_err());

        // 缺少斜杠
        let result = service.uninstall_skill("invalid:skill".to_string()).await;
        assert!(result.is_err());
    }

    // ===== 仓库管理测试 =====

    /// 测试列出默认仓库
    #[tokio::test]
    async fn test_list_repos_default() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root);

        let repos = service.list_repos().await.unwrap();
        // 应该返回默认仓库
        assert!(!repos.is_empty());
    }

    /// 测试添加和列出仓库
    #[tokio::test]
    async fn test_add_and_list_repo() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root);

        let new_repo = SkillRepoConfig {
            owner: "test-owner".to_string(),
            name: "test-repo".to_string(),
            branch: "dev".to_string(),
            enabled: true,
        };

        service.add_repo(new_repo.clone()).await.unwrap();

        let repos = service.list_repos().await.unwrap();
        assert!(repos
            .iter()
            .any(|r| r.owner == "test-owner" && r.name == "test-repo"));
    }

    /// 测试添加重复仓库（幂等）
    #[tokio::test]
    async fn test_add_duplicate_repo() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root);

        let repo = SkillRepoConfig {
            owner: "dup-owner".to_string(),
            name: "dup-repo".to_string(),
            branch: "main".to_string(),
            enabled: true,
        };

        // 添加两次
        service.add_repo(repo.clone()).await.unwrap();
        service.add_repo(repo.clone()).await.unwrap();

        let repos = service.list_repos().await.unwrap();
        let count = repos.iter().filter(|r| r.owner == "dup-owner").count();
        // 应该只有一个
        assert_eq!(count, 1);
    }

    /// 测试移除仓库
    #[tokio::test]
    async fn test_remove_repo() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root);

        let repo = SkillRepoConfig {
            owner: "remove-owner".to_string(),
            name: "remove-repo".to_string(),
            branch: "main".to_string(),
            enabled: true,
        };

        // 添加
        service.add_repo(repo.clone()).await.unwrap();
        let repos = service.list_repos().await.unwrap();
        assert!(repos.iter().any(|r| r.owner == "remove-owner"));

        // 移除
        service.remove_repo(repo).await.unwrap();
        let repos = service.list_repos().await.unwrap();
        assert!(!repos.iter().any(|r| r.owner == "remove-owner"));
    }

    // ===== 扫描已安装 Skill 测试 =====

    /// 测试扫描空目录
    #[tokio::test]
    async fn test_scan_installed_skills_empty() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root);

        let skills = service.scan_installed_skills().await.unwrap();
        assert!(skills.is_empty());
    }

    /// 测试扫描已安装的 Skill
    #[tokio::test]
    async fn test_scan_installed_skills() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root.clone());

        // 创建模拟已安装的 Skill
        let skill_path = root
            .join("skills")
            .join("my-owner")
            .join("my-repo")
            .join("cool-skill");
        std::fs::create_dir_all(&skill_path).unwrap();
        let skill_yaml = skill_path.join("skill.yaml");
        std::fs::write(&skill_yaml, "name: Cool Skill\ndescription: A cool skill\n").unwrap();

        let skills = service.scan_installed_skills().await.unwrap();

        assert_eq!(skills.len(), 1);
        let skill = &skills[0];
        assert_eq!(skill.name, "Cool Skill");
        assert_eq!(skill.description, "A cool skill");
        assert!(skill.installed);
        assert_eq!(skill.repo_owner, "my-owner");
        assert_eq!(skill.repo_name, "my-repo");
        assert_eq!(skill.directory, "cool-skill");
    }

    /// 测试扫描无 skill.yaml 的目录
    #[tokio::test]
    async fn test_scan_installed_skills_no_yaml() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let service = SkillService::with_root(root.clone());

        // 创建目录但不创建 skill.yaml
        let skill_path = root
            .join("skills")
            .join("owner")
            .join("repo")
            .join("skill-no-yaml");
        std::fs::create_dir_all(&skill_path).unwrap();

        let skills = service.scan_installed_skills().await.unwrap();

        assert_eq!(skills.len(), 1);
        let skill = &skills[0];
        // 没有 skill.yaml，使用目录名作为名称
        assert_eq!(skill.name, "skill-no-yaml");
        assert_eq!(skill.description, "(Installed)");
    }

    // ===== 路径生成测试 =====

    /// 测试 Skill 安装路径生成
    #[test]
    fn test_get_skill_install_path() {
        let root = PathBuf::from("/tmp/test");
        let service = SkillService {
            http_client: Client::new(),
            root_dir: root.clone(),
        };

        let path = service.get_skill_install_path("owner", "repo", "skill");
        assert_eq!(
            path,
            root.join("skills").join("owner").join("repo").join("skill")
        );
    }
}
