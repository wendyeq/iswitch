# Spec: Mini HUD 设置

## ADDED Requirements

### Requirement: Mini HUD 设置持久化
系统 SHALL 持久化 Mini HUD 的用户偏好设置，包括启用状态和窗口位置。

#### Scenario: HUD 启用状态持久化
- **GIVEN** 用户通过托盘菜单打开 Mini HUD
- **WHEN** 应用下次启动
- **THEN** HUD 窗口自动显示（如果之前是开启状态）
- **AND** 设置保存在本地配置文件中

#### Scenario: HUD 位置持久化
- **GIVEN** 用户拖拽调整了 HUD 窗口位置
- **WHEN** 应用重启
- **THEN** HUD 窗口在之前保存的位置显示
- **AND** 位置信息 (x, y) 被保存到配置文件

#### Scenario: HUD 首次启动默认位置
- **GIVEN** 用户首次打开 Mini HUD
- **WHEN** 没有保存的位置信息
- **THEN** HUD 窗口显示在屏幕右上角
- **AND** 距离屏幕边缘保持适当间距（如 20px）
