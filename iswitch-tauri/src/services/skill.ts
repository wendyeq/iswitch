/**
 * ---
 * [INPUT]: {Tauri Commands}
 *     - Commands: source: ../../src-tauri/src/commands/skill.rs ([POS]: Skill 命令)
 * [OUTPUT]: {Skill API} - Skill 管理相关的前端 API
 * [POS]: Skill 服务层，封装 Skill 仓库管理和安装的 Tauri 调用
 * [PROTOCOL]:
 * 1. 重新导出 tauri.ts 中的 Skill 相关函数
 * 2. 提供类型安全的 Skill 接口
 * ---
 */

/**
 * Skill 服务
 * Skill 仓库管理与下载安装
 */
import {
  fetchSkills as tauriFetchSkills,
  installSkill as tauriInstallSkill,
  uninstallSkill as tauriUninstallSkill,
  fetchSkillRepos as tauriFetchSkillRepos,
  addSkillRepo as tauriAddSkillRepo,
  removeSkillRepo as tauriRemoveSkillRepo,
  type SkillSummary,
  type SkillRepoConfig,
  type InstallSkillPayload,
} from './tauri';

export const fetchSkills = tauriFetchSkills;
export const installSkill = tauriInstallSkill;
export const uninstallSkill = tauriUninstallSkill;
export const fetchSkillRepos = tauriFetchSkillRepos;
export const addSkillRepo = tauriAddSkillRepo;
export const removeSkillRepo = tauriRemoveSkillRepo;
export type { SkillSummary, SkillRepoConfig, InstallSkillPayload };
