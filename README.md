# Hugo Blog Manager

一个使用 Rust 和 egui 构建的 Hugo 博客管理桌面应用。

## 功能特性

### Dashboard
- 项目概览（站点标题、主题、Base URL）
- 文章数量和主题数量统计
- 快速操作：浏览器打开站点、启动 Hugo Server、构建站点
- 一键切换项目目录
- **自定义 Hugo 路径**：可手动指定 hugo.exe 位置（无需系统 PATH）

### 文章管理
- 文章列表，支持分页浏览和关键词搜索（标题/分类/标签）
- 创建、编辑、删除文章（删除移至回收站）
- 自动生成 TOML frontmatter（`+++` 格式）
- 支持设置标题、标签、分类、草稿状态
- 支持 CJK（中日韩）字符作为文章 slug

### 主题管理
- 主题画廊：查看已安装主题
- 一键激活/停用主题
- 配置颜色方案（Auto / Light / Dark）
- 删除不需要的主题（回收站）
- 打开主题文件夹添加新主题

### 站点设置
- 基本设置：站点标题、Base URL、语言代码
- 侧边栏：副标题、Emoji
- 文章设置：数学公式、目录、阅读时间开关
- 保存时保留 `hugo.yaml` 中未管理的字段（不丢失手动配置）

## 安全特性

- **命令执行防护**：Hugo 可执行文件仅从系统 PATH 查找或用户指定路径，且必须为绝对路径
- **路径遍历防护**：所有文件操作通过 `canonicalize` + `starts_with` 校验路径归属
- **Frontmatter 注入防护**：用户输入经 TOML 转义处理，过滤引号、换行和控制字符
- **删除操作防护**：删除前校验路径在 `content/` 或 `themes/` 目录内
- **配置文件保护**：原子写入（先写临时文件再重命名），防止写入中断导致损坏
- **目录扫描保护**：递归深度限制（10 层）+ 跳过符号链接，防止无限循环
- **文件大小限制**：读取文件上限 10MB，防止 OOM
- **外部程序调用保护**：`open::that` 严格校验 URL 协议和文件类型

## 系统要求

- Rust 1.70+
- Hugo（可通过 Dashboard 手动指定路径，或添加到系统 PATH）
- Windows 10+ （当前仅支持 Windows，中文字体自动加载）

## 安装

```bash
# 克隆项目
git clone <repo-url>
cd hugo-manager

# 编译
cargo build --release

# 运行
cargo run --release
```

## 使用说明

### 启动

1. 在 Hugo 博客项目目录下运行程序，自动检测 `hugo.yaml` / `hugo.toml` / `config.yaml` / `config.toml`
2. 或通过 Dashboard 的 "Change..." 按钮手动选择项目目录

### 配置 Hugo 路径

1. 在 Dashboard 的 **Project Overview** 中找到 **Hugo Path** 配置项
2. 点击 **Browse...** 选择 hugo.exe 文件
3. 路径会自动保存到 `config/settings.json`
4. 点击 **Clear** 可清除自定义路径，恢复使用系统 PATH

### 文章管理

1. 点击 **Articles** 标签
2. 使用搜索框过滤文章（支持标题、分类、标签）
3. 点击 **+ New Article** 创建新文章
4. 填写标题、标签（逗号分隔）、分类、内容
5. 点击 **Save** 保存，文章自动创建在 `content/post/<slug>/index.md`

### 主题管理

1. 点击 **Themes** 标签
2. 当前激活主题显示在顶部（蓝色边框）
3. 点击 **Activate** 切换主题，点击 **Deactivate** 停用
4. 通过 Color 下拉框切换颜色方案

### 站点设置

1. 点击 **Settings** 标签
2. 修改所需配置项
3. 点击右上角 **Save Settings** 保存

## 项目结构

```
hugo-manager/
├── Cargo.toml
├── README.md
├── config/
│   └── settings.json      # 应用配置（Hugo 路径、项目路径）
├── src/
│   ├── main.rs          # 程序入口、字体配置
│   ├── app.rs           # 主应用逻辑和全部 UI
│   ├── models/
│   │   ├── mod.rs       # 模块导出
│   │   ├── article.rs   # 文章 CRUD、frontmatter 解析
│   │   ├── config.rs    # Hugo 配置读写（YAML）
│   │   └── theme.rs     # 主题扫描与管理
│   └── utils/
│       └── mod.rs       # 文件读写、编码检测、工具函数
```

## 技术栈

| 组件 | 技术 |
|------|------|
| GUI 框架 | [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) 0.28 |
| 序列化 | serde + serde_yaml + toml |
| 文件对话框 | [rfd](https://github.com/PolyMeilex/rfd) |
| 回收站 | [trash](https://github.com/Byron/trash-rs) |
| 日期处理 | [chrono](https://github.com/chronotope/chrono) |
| 错误处理 | [anyhow](https://github.com/dtolnay/anyhow) |

## 注意事项

- 文章使用 TOML frontmatter 格式（`+++` 分隔符）
- 配置文件仅支持 YAML 格式的读写，TOML 配置会使用默认值
- 删除操作移至系统回收站，不会永久删除
- `config/settings.json` 保存应用配置（Hugo 路径、项目路径）

## 许可证

MIT License
