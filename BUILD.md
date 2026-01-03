# Keep Accounts 构建说明

## GitHub Actions 自动构建

项目配置了 GitHub Actions workflow，可以自动构建 Windows 和 Android 版本。

### 触发构建

构建可以通过以下两种方式触发：

1. **推送 tag（推荐）**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **手动触发**
   - 访问 GitHub 仓库的 Actions 页面
   - 选择 "release" workflow
   - 点击 "Run workflow" 按钮

### 构建平台

workflow 会并行构建以下平台：

- **Windows**: 生成 `.exe` 安装包
- **Android**: 生成 `.apk` 安装包

### 构建产物

构建完成后，产物会自动上传到当前仓库的 GitHub Release：

- Windows 安装包：`keep-accounts_<version>_x64-setup.exe`
- Android 安装包：`keep-accounts_<version>.apk`

## 本地构建

### Windows 构建

```bash
cd keep-accounts
# 安装依赖
cargo install --locked trunk
# 构建前端
trunk build
# 构建 Tauri 应用
cargo tauri build
```

构建产物位于：`src-tauri/target/release/bundle/nsis/`

### Android 构建

Android 构建需要额外配置：

#### 1. 安装依赖

```bash
# 安装 Rust Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android

# 安装 JDK 17
# Ubuntu/Debian:
sudo apt-get install openjdk-17-jdk

# macOS:
brew install openjdk@17

# Windows:
# 下载并安装 Oracle JDK 17 或 Temurin JDK 17
```

#### 2. 安装 Android SDK

```bash
# Ubuntu/Debian:
sudo apt-get install android-sdk

# macOS:
brew install --cask android-sdk

# Windows:
# 使用 Android Studio 或手动安装 Android SDK
```

#### 3. 配置环境变量

设置以下环境变量：

```bash
# Android SDK 路径
export ANDROID_SDK_ROOT=/path/to/android/sdk
export ANDROID_HOME=/path/to/android/sdk

# Java 路径
export JAVA_HOME=/path/to/java17
```

#### 4. 初始化 Android 项目

```bash
cd keep-accounts
cargo tauri android init
```

#### 5. 构建 APK

```bash
# 构建前端
trunk build

# 构建 Android APK
cargo tauri android build
```

构建产物位于：`src-tauri/gen/android/app/build/outputs/apk/`

## Tauri 2.0 Android 配置

### 配置文件位置

- Android 项目配置：`src-tauri/gen/android/`
- Tauri 配置：`src-tauri/tauri.conf.json`
- Cargo 配置：`src-tauri/Cargo.toml`

### 常见问题

1. **构建失败：找不到 NDK**
   ```bash
   # 通过 Android SDK Manager 安装 NDK
   # 或在 tauri.conf.json 中指定 NDK 版本
   ```

2. **签名问题**
   - Debug 构建使用自动生成的 debug 密钥
   - Release 构建需要配置签名密钥

3. **权限配置**
   - 在 `src-tauri/gen/android/app/src/main/AndroidManifest.xml` 中配置所需的 Android 权限

## 发布流程

1. 更新 `src-tauri/tauri.conf.json` 中的版本号
2. 提交所有更改
3. 创建并推送 tag
4. 等待 GitHub Actions 完成构建
5. 检查 Release 页面，确认构建产物
6. 测试安装包
7. 将 Release 从草稿状态发布

## 相关链接

- [Tauri 2.0 文档](https://v2.tauri.app/)
- [Tauri Android 指南](https://v2.tauri.app/start/migrate/from-tauri-1)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
