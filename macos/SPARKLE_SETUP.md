# Sparkle 自动更新接入报告

**日期：** 2026-04-15  
**状态：** 代码已完成，待配置 Secrets 并测试

---

## 一、做了什么

### 1. Xcode 项目（`macos/SEE.xcodeproj/project.pbxproj`）

- 新增 Sparkle 2.6+ SPM 依赖（`https://github.com/sparkle-project/Sparkle`）
- 将 `Sparkle` framework 链接到 SEE target

### 2. App 代码（`macos/SEE/SEEApp.swift`）

- 导入 Sparkle（仅 macOS）
- 在 app 启动时初始化 `SPUStandardUpdaterController`
- 在 macOS App 菜单中添加 **"Check for Updates…"** 菜单项（`CommandGroup(after: .appInfo)`）

### 3. Info.plist（`macos/SEE/Info.plist`，新建）

新建了一个补充 Info.plist，Xcode 会将它与自动生成的 Info.plist 合并：

```xml
<key>SUFeedURL</key>
<string>https://s.ee/sparkle/appcast.xml</string>   <!-- 检查更新的 appcast 地址 -->
<key>SUPublicEDKey</key>
<string>SPARKLE_ED25519_PUBLIC_KEY</string>          <!-- 占位符，需替换为真实公钥 -->
<key>SUEnableAutomaticChecks</key>
<true/>
```

> ⚠️ `SUPublicEDKey` 目前是占位符。本地开发前需替换为真实公钥（见下方步骤）。CI 中由 GitHub Secret 自动注入。

### 4. GitHub Actions 工作流（`.github/workflows/release-macos.yml`，新建）

完整的自动化流程，推送 `v*` tag 或手动触发后执行：

1. 从 Secret 导入 Developer ID 证书到临时 Keychain
2. `xcodebuild archive`（Developer ID 签名，Hardened Runtime）
3. `xcodebuild -exportArchive`（developer-id 导出方式）
4. `hdiutil create` 打包 DMG（含拖拽到 Applications 的快捷方式）
5. `xcrun notarytool submit --wait` 公证
6. `xcrun stapler staple` 钉入公证票据
7. 用 Sparkle 工具或 fallback 生成 `appcast.xml`
8. 将 DMG + SHA256 校验文件 + appcast.xml 发布到 GitHub Release

---

## 二、测试前需要准备的东西

### 第一步：生成 Sparkle EdDSA 密钥对

在你的 Mac 上，用 Xcode 打开项目解析依赖后运行：

```bash
# 找到 Sparkle 的 generate_keys 工具
find ~/Library/Developer/Xcode/DerivedData -name "generate_keys" -type f 2>/dev/null | head -1
```

如果找不到，直接下载 Sparkle release 包：

```bash
curl -L https://github.com/sparkle-project/Sparkle/releases/latest/download/Sparkle-2.6.4.tar.xz | tar xJ
./Sparkle.framework/Resources/bin/generate_keys
```

输出示例：
```
Private signing key (base64-encoded):
<YOUR_PRIVATE_KEY>

Public signing key (base64-encoded):
<YOUR_PUBLIC_KEY>
```

- **私钥**：妥善保存，绝对不要提交到 Git
- **公钥**：填入 Info.plist 和 GitHub Secret

### 第二步：更新本地 Info.plist 中的公钥

编辑 `macos/SEE/Info.plist`，将：
```xml
<string>SPARKLE_ED25519_PUBLIC_KEY</string>
```
替换为真实公钥（仅本地开发需要，CI 会自动注入）。

### 第三步：导出 Developer ID Application 证书（.p12）

1. 打开 **Keychain Access**
2. 找到 `Developer ID Application: <Your Name> (MJRBT9FJS7)`
3. 右键 → Export → 保存为 `.p12`，设置一个密码
4. Base64 编码：
   ```bash
   base64 -i certificate.p12 | pbcopy
   ```

### 第四步：在 GitHub Repo 配置 Secrets

进入 **Settings → Secrets and variables → Actions → New repository secret**，依次添加：

| Secret 名称 | 值 |
|---|---|
| `MACOS_CERTIFICATE_P12` | 上一步 base64 编码的 .p12 内容 |
| `MACOS_CERTIFICATE_PASSWORD` | 导出 .p12 时设置的密码 |
| `KEYCHAIN_PASSWORD` | 任意随机字符串，例如 `openssl rand -hex 16` 的输出 |
| `APPLE_TEAM_ID` | `MJRBT9FJS7` |
| `APPLE_ID` | 你的 Apple ID 邮箱 |
| `APPLE_APP_PASSWORD` | 在 [appleid.apple.com](https://appleid.apple.com) 生成的 App 专用密码 |
| `SPARKLE_ED25519_PUBLIC_KEY` | 第一步生成的公钥 |
| `SPARKLE_ED25519_PRIVATE_KEY` | 第一步生成的私钥 |
| `SPARKLE_DOWNLOAD_URL_PREFIX` | DMG 托管的 URL 前缀，例如 `https://s.ee/sparkle/` |

### 第五步：配置服务端托管

Sparkle 检查更新需要两个 URL 可访问：

1. **Appcast XML**：`https://s.ee/sparkle/appcast.xml`  
   每次发版后，从 GitHub Release 下载新生成的 `appcast.xml` 上传到此处

2. **DMG 下载**：`https://s.ee/sparkle/SEE-<version>.dmg`  
   即 `SPARKLE_DOWNLOAD_URL_PREFIX` + DMG 文件名

> 如果不想手动同步，可以考虑直接将 `SUFeedURL` 指向 GitHub Release 的 raw URL，或写一个 webhook 自动同步。

---

## 三、触发第一次发版（测试）

准备好上述所有配置后：

```bash
git add .
git commit -m "chore: integrate Sparkle auto-update"
git tag v1.0.2
git push origin main --tags
```

然后在 GitHub → Actions → Release macOS 查看构建进度。

---

## 四、验证清单

| 项目 | 方法 |
|---|---|
| DMG 可正常安装 | 双击 DMG，拖入 Applications，打开 App |
| App 已签名 | `codesign --verify --deep --strict SEE.app` |
| App 已公证 | `spctl --assess --verbose SEE.app` |
| 签名者为 Developer ID | `codesign -dv SEE.app 2>&1 \| grep Authority` |
| Sparkle 能检查更新 | App 菜单 → "Check for Updates…" |
| 自动更新提示 | 修改 appcast.xml 指向更高版本，重新打开 App |

---

## 五、常见问题

**Q：notarytool 超时？**  
A：工作流设置了 `--timeout 30m`，Apple 公证通常 2-5 分钟完成。如果超时，检查 App 是否启用了 Hardened Runtime 且证书为 Developer ID（不是 Apple Development）。

**Q：本地 Xcode 打开提示找不到 Sparkle？**  
A：打开 `macos/SEE.xcodeproj`，Xcode 会自动解析 SPM 依赖并下载 Sparkle，等待完成即可。

**Q：iOS 构建是否受影响？**  
A：不受影响。Sparkle 相关代码全部用 `#if os(macOS)` 包裹，iOS target 不会引用 Sparkle。
