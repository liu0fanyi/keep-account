# Keep Accounts 部署指南

本文档详细说明如何配置和使用 GitHub Actions 自动构建和发布 Keep Accounts 应用。

## 目录

- [前置准备](#前置准备)
- [仓库配置](#仓库配置)
- [GitHub 配置](#github-配置)
- [本地测试](#本地测试)
- [发布流程](#发布流程)
- [故障排查](#故障排查)

---

## 前置准备

### 1. 确认项目状态

确保 keep-accounts 项目已完成以下工作：

- ✅ 所有代码已提交到 Git
- ✅ `src-tauri/tauri.conf.json` 配置正确
- ✅ `.github/workflows/release.yml` 文件存在
- ✅ 项目可以本地构建成功

```bash
cd keep-accounts

# 检查 Git 状态
git status

# 确认 workflow 文件存在
ls -la .github/workflows/release.yml

# 测试本地构建
trunk build
cargo tauri build
```

### 2. 准备 GitHub 仓库

#### 选项 A: 使用现有仓库

如果 keep-accounts 已经是独立的 Git 仓库：

```bash
# 查看当前 remote
git remote -v

# 如果没有 remote，添加一个
git remote add origin https://github.com/yourusername/keep-accounts.git

# 推送代码
git push -u origin master
```

#### 选项 B: 从 monorepo 分离

如果 keep-accounts 当前在 tauri-apps 目录下，需要分离成独立仓库：

```bash
# 在 tauri-apps 目录下
cd tauri-apps

# 导出 keep-accounts 为独立仓库
git subtree push --prefix=keep-accounts origin keep-accounts

# 或者创建全新的仓库
cd keep-accounts
git remote add origin https://github.com/yourusername/keep-accounts.git
git push -u origin master
```

---

## GitHub 配置

### 1. 启用 GitHub Actions

1. 访问你的 GitHub 仓库页面
2. 点击 **Settings** 标签
3. 在左侧菜单中选择 **Actions** → **General**
4. 在 **Actions permissions** 下选择：
   - ✅ **Allow all actions and reusable workflows**
5. 点击 **Save** 保存

### 2. 配置仓库变量 (Variables)

1. 进入 **Settings** → **Secrets and variables** → **Actions**
2. 点击 **Variables** 标签
3. 点击 **New repository variable** 添加以下变量：

| 变量名 | 值 | 说明 |
|--------|-----|------|
| `TAURI_ANDROID_PACKAGE` | `com.keep_accounts.app` | Android 包名 |

**步骤**:
```
Name: TAURI_ANDROID_PACKAGE
Value: com.keep_accounts.app
→ Click "Add variable"
```

### 3. 配置仓库密钥 (Secrets) - 可选

如果需要签名应用（推荐用于生产环境）：

1. 在 **Secrets and variables** → **Actions** 中
2. 点击 **New repository secret**
3. 添加以下密钥：

| 密钥名 | 说明 | 获取方式 |
|--------|------|----------|
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri 私钥 | `cargo tauri signer generate` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 私钥密码 | 生成时设置的密码 |

**生成签名密钥** (可选):
```bash
# 安装 signer 工具
cargo install tauri-cli

# 生成密钥对
cargo tauri signer generate

# 会输出：
# - Private key (保存到 TAURI_SIGNING_PRIVATE_KEY)
# - Public key (保存到项目配置中)
# - Password (保存到 TAURI_SIGNING_PRIVATE_KEY_PASSWORD)
```

---

## 本地测试

在触发 CI/CD 前，建议先在本地测试构建：

### Windows 构建

```bash
cd keep-accounts

# 安装依赖
cargo install --locked trunk

# 构建前端
trunk build

# 构建 Windows 应用
cargo tauri build

# 检查输出
ls -la src-tauri/target/release/bundle/nsis/
# 应该看到: keep-accounts_<version>_x64-setup.exe
```

### Android 构建

```bash
cd keep-accounts

# 安装 Rust Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi

# 确保已安装 JDK 17
java -version  # 应该显示 17.x.x

# 初始化 Android 项目（首次）
cargo tauri android init

# 构建 Android APK
trunk build
cargo tauri android build

# 查找生成的 APK
find src-tauri/gen/android -name "*.apk"
```

---

## 发布流程

### 方式 1: 自动发布 (推荐)

使用 Git tag 触发自动构建和发布：

#### 步骤 1: 更新版本号

编辑 `src-tauri/tauri.conf.json`:

```json
{
  "version": "1.0.0"  // 修改为新版本号
}
```

#### 步骤 2: 提交版本更改

```bash
git add src-tauri/tauri.conf.json
git commit -m "Bump version to 1.0.0"
git push origin master
```

#### 步骤 3: 创建并推送 tag

```bash
# 创建 tag
git tag v1.0.0

# 推送 tag 到 GitHub
git push origin v1.0.0
```

#### 步骤 4: 监控构建进度

1. 访问 GitHub 仓库的 **Actions** 页面
2. 查看 "release" workflow 的运行状态
3. 构建时间通常：
   - Windows: 5-10 分钟
   - Android: 10-15 分钟

#### 步骤 5: 下载和测试

1. 构建完成后，访问 **Releases** 页面
2. 下载对应平台的安装包：
   - Windows: `keep-accounts_1.0.0_x64-setup.exe`
   - Android: `keep-accounts_1.0.0.apk`
3. 测试安装和基本功能

#### 步骤 6: 发布 Release

检查构建产物无误后：
- Release 默认会自动发布（移除 draft 状态）
- 如需修改 release 描述，可以手动编辑

### 方式 2: 手动触发

用于测试或不方便创建 tag 的情况：

1. 访问 GitHub 仓库
2. 点击 **Actions** 标签
3. 选择 **release** workflow
4. 点击 **Run workflow** 按钮
5. 选择分支（通常是 `master`）
6. 点击 **Run workflow** 确认

---

## 发布检查清单

在发布新版本前，请确认：

- [ ] 版本号已更新（`tauri.conf.json`）
- [ ] 所有更改已提交并推送
- [ ] 本地构建成功（Windows + Android）
- [ ] CHANGELOG.md 已更新（如果有）
- [ ] README.md 中的功能列表准确
- [ ] 测试了主要功能：
  - [ ] 添加分类
  - [ ] 记录交易
  - [ ] 查看汇总
  - [ ] 分期管理
- [ ] 构建产物已下载并测试
- [ ] Release 描述完整

---

## 故障排查

### 问题 1: Workflow 失败 - Windows 构建

**症状**: Windows 构建步骤失败

**可能原因和解决方案**:

1. **Trunk 未安装**
   ```yaml
   # 在 workflow 中已包含: cargo install --locked trunk
   # 如仍有问题，检查 trunk 版本
   ```

2. **构建产物未找到**
   - 检查路径: `src-tauri/target/release/bundle/nsis/`
   - 查看构建日志确认输出路径

3. **上传失败**
   - 检查 `GITHUB_TOKEN` 权限
   - 确认 Actions 权限已启用

### 问题 2: Workflow 失败 - Android 构建

**症状**: Android 构建步骤失败

**可能原因和解决方案**:

1. **APK 未找到**
   ```yaml
   # Workflow 中包含详细的错误处理和查找逻辑
   # 查看日志找到实际的 APK 输出路径
   ```

2. **Android SDK 配置问题**
   - 确认 `android-actions/setup-android@v3` 正常运行
   - 检查 NDK 是否正确安装

3. **Package 名称错误**
   - 确认 `TAURI_ANDROID_PACKAGE` 变量已设置
   - 格式: `com.keep_accounts.app`

### 问题 3: 签名错误

**症状**: 更新检查失败或签名验证失败

**解决方案**:

1. 检查是否正确配置了签名密钥
2. 确认公钥已添加到 `src-tauri/tauri.conf.json`
3. 验证密钥密码正确

### 问题 4: Release 创建失败

**症状**: create-release 步骤失败

**解决方案**:

1. 检查 tag 格式: 必须是 `v*` (如 v1.0.0)
2. 确认有写入权限
3. 查看详细错误日志

### 查看日志

所有构建步骤的日志都可以在 GitHub Actions 页面查看：

1. 访问 Actions 页面
2. 点击失败的 workflow run
3. 展开失败的步骤
4. 查看详细日志

---

## 常用命令

### Git 操作

```bash
# 查看当前版本
grep '"version"' src-tauri/tauri.conf.json

# 创建 tag
git tag v1.0.0

# 查看所有 tags
git tag -l

# 删除本地 tag
git tag -d v1.0.0

# 删除远程 tag
git push origin :refs/tags/v1.0.0

# 推送所有 tags
git push origin --tags
```

### 构建操作

```bash
# 清理构建缓存
cargo clean

# 重新构建
trunk build
cargo tauri build --verbose

# Android 构建
cargo tauri android build --verbose

# 初始化 Android 项目
cargo tauri android init
```

---

## 最佳实践

1. **版本管理**
   - 使用语义化版本 (Semantic Versioning): `MAJOR.MINOR.PATCH`
   - 例: `1.0.0` → `1.0.1` (bug 修复) → `1.1.0` (新功能) → `2.0.0` (重大更改)

2. **发布前测试**
   - 始终在本地先测试构建
   - 使用 draft release 先测试，确认无误后再发布

3. **Change Log**
   - 维护 CHANGELOG.md 记录版本变更
   - 在 release 描述中引用相关 commit

4. **回滚计划**
   - 保留旧版本的 release
   - 如果新版本有严重问题，可以快速指引用户回退

---

## 相关资源

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Tauri 2.0 构建文档](https://v2.tauri.app/distribute/)
- [Tauri Android 指南](https://v2.tauri.app/distribute/android/)
- [语义化版本](https://semver.org/lang/zh-CN/)
- [BUILD.md](./BUILD.md) - 详细的构建说明
- [README.md](./README.md) - 项目介绍

---

## 支持

如果遇到问题：

1. 查看本文档的 [故障排查](#故障排查) 部分
2. 检查 GitHub Actions 日志
3. 搜索 [Tauri GitHub Issues](https://github.com/tauri-apps/tauri/issues)
4. 提交新的 Issue 并附上详细的错误日志

---

**文档版本**: 1.0.0
**最后更新**: 2026-01-02
