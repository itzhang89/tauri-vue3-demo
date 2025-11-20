# GitHub Actions 构建说明

## 概述

这个 workflow 会自动为以下平台构建 Tauri 应用：

- **桌面平台**：
  - macOS (Apple Silicon - aarch64)
  - macOS (Intel - x86_64)
  - Windows (x86_64)
  - Linux (x86_64)

- **移动平台**：
  - Android (APK)
  - iOS (IPA) - 需要配置证书

## 触发条件

Workflow 会在以下情况触发：
1. 推送标签（格式：`v*`，如 `v1.0.0`）
2. 创建 Pull Request
3. 手动触发（在 GitHub Actions 页面）

## 配置说明

### 桌面平台构建

桌面平台构建会自动运行，无需额外配置。构建产物会自动上传到 GitHub Releases（仅当推送标签时）。

### Android 构建

Android 构建会自动运行，无需额外配置。构建产物会作为 Artifact 上传。

### iOS 构建（可选）

iOS 构建需要配置以下 GitHub Secrets（在仓库 Settings > Secrets and variables > Actions 中配置）：

- `APPLE_CERTIFICATE`: Apple 开发者证书（.p12 文件内容，base64 编码）
- `APPLE_CERTIFICATE_PASSWORD`: 证书密码
- `APPLE_TEAM_ID`: Apple 开发者团队 ID
- `APPLE_ID`: Apple ID 邮箱
- `APPLE_APP_SPECIFIC_PASSWORD`: App 专用密码（在 appleid.apple.com 生成）

**注意**：如果没有配置这些 secrets，iOS 构建会失败但不会影响其他平台的构建。

## 使用方式

### 自动构建（推荐）

1. 创建并推送一个标签：
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. GitHub Actions 会自动触发构建

3. 构建完成后，在 GitHub Releases 页面可以下载桌面平台的安装包

### 手动构建

1. 在 GitHub 仓库页面，点击 "Actions" 标签
2. 选择 "Build Tauri App" workflow
3. 点击 "Run workflow" 按钮
4. 选择分支并运行

## 构建产物

### 桌面平台
- **macOS**: `.dmg` 和 `.app` 文件
- **Windows**: `.msi` 和 `.exe` 文件
- **Linux**: `.deb` 和 `.AppImage` 文件

### 移动平台
- **Android**: `.apk` 文件（在 Artifacts 中）
- **iOS**: `.ipa` 文件（在 Artifacts 中，需要配置证书）

## 故障排除

### Android 构建失败
- 确保 Android SDK 已正确安装
- 检查 Java 版本（需要 JDK 17）

### iOS 构建失败
- 检查是否配置了所有必需的 secrets
- 确保证书未过期
- 检查 Team ID 是否正确

### 桌面平台构建失败
- 检查 Rust 工具链是否正确安装
- 确保所有依赖都已安装
- 查看构建日志获取详细错误信息

