# 博丽手记 · ReimuMark

博丽手记是基于 [ElyOffice 的 ElyMark](https://github.com/ELY-Open-Source/ElyOffice) 制作的独立 Markdown 编辑器。这个 fork 只维护 Markdown 编辑器，发布包只包含编辑器及 PDF 导出所需的工作进程。ElyOffice 的其他应用由上游维护。

## 下载

从 [GitHub Releases](https://github.com/tingyuluo-cyber/ElyOffice/releases/latest) 下载 `ReimuMark-*-windows-x64.zip`，解压整个文件夹，双击 `ReimuMark.exe`。适用于 Windows x64，无需安装 Rust。

- 支持可视编辑、源码、阅读预览和宽幅显示。
- 可以打开文件或目录，也可以拖入 Markdown 文件。左侧大纲默认展开，右侧缩略图可切换。
- 提供灵梦红白与八云紫 Nord 两套主题，以及 HTML、纯文本和 PDF 导出。
- 文档保存在自己选择的位置。默认设置目录为 `%LOCALAPPDATA%\ElyMarkReimu`。

## 0.1.26

- 可视模式和源码模式下, 框选文本后连按三次反引号可转为 Markdown 代码块, 保留原文和换行.

## 0.1.25

- Markdown 正文支持省略分号的 HTML 命名实体, 例如 `&rarr` 显示为 `→`.
- 可视编辑, 预览和导出使用相同的解析规则.

## 0.1.24

- 源码和预览切换时保留选区, 并定位到对应文字.
- 长段落自动换行后仍能定位选中文字.
- 修复行边界, 段落间隙和表格中的点击命中错位.

## 0.1.23

- 预览模式中已勾选的任务文本显示删除线, 取消勾选后恢复.
- 删除线与 Markdown `~~文本~~` 一致, 子任务按各自勾选状态显示.

## 0.1.22

- 预览模式支持拖选文本, Shift 扩展选择, Ctrl+C 和右键复制.
- Ctrl+A 可选中完整预览文本, 滚动后保留选区.

## 0.1.21

- 预览模式可直接勾选和取消 Markdown 任务项, 状态同步保存到文档.
- 支持嵌套, 引用和编号任务列表, 保留原始格式及撤销记录.

## 0.1.20

- 列表前插入条目后自动重排编号.
- 选中文本后可连续输入星号添加强调和粗体.
- 源码随窗口宽度和字号自动换行.
- 中文斜体正常显示. 粗体与斜体使用不同主题色, 编辑和预览一致.
- 默认取消 Ctrl+Shift+E, Ctrl+/ 切换源码.

## 构建

准备 Windows x64、Visual Studio C++ Build Tools（含 Windows SDK）和 Rustup。Rust 版本由 `rust-toolchain.toml` 指定。

```powershell
cargo run --locked -p ely-mark --features standalone
```

打包需要 PowerShell 7.4 和 7-Zip。在仓库根目录运行以下命令，生成的 ZIP 位于 `dist`。

```powershell
pwsh -File packaging/windows/reimu-zip.ps1
```

从源码运行时，如需 PDF 导出，请将 `ELY_PDF_WORKER` 设为从 ZIP 解压出的 `ely-pdf-worker.exe` 的绝对路径。运行 `cargo test --locked -p ely-mark --bin ely-mark` 前也要设置此变量。

## 许可与来源

自有代码使用 [MIT](LICENSE)。随包的第三方许可位于 `licenses`。PDF 导出随附 ElyMark 0.1.15 的原版工作进程，其中 MuPDF 使用 AGPL。对应的 [工作进程及构建脚本源码](https://github.com/ELY-Open-Source/ElyOffice/releases/download/v0.1.15/ElyOffice-0.1.15-dc32744d5daa-source.tar.gz) 和 [已应用补丁的 MuPDF 源码](https://github.com/ELY-Open-Source/ElyOffice/releases/download/v0.1.15/ElyOffice-0.1.15-dc32744d5daa-upstream-source.tar.gz) 均可从原版发行下载。

主题为《东方 Project》非官方二次创作。博丽灵梦与八云紫来自上海爱丽丝幻乐团的《东方 Project》，两张主题插画由 AI 生成。角色相关权利归原权利人所有。 [东方 Project 二次创作指引](https://touhou-project.news/guidelines_en/)
