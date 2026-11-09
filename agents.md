# 博丽手记

这是 ElyMark 的独立 Markdown 编辑器发行分支。界面为 Rust、GPUI 和 Ely-GPUI-Components，文档留在用户目录。主题素材位于 `apps/ely-mark/assets/reimu`。

- `apps/ely-mark` 是唯一发行的应用。共享库留在 `crates`，PDF 导出使用独立的 `ely-pdf-worker`。保留工作进程源码和 MuPDF 补丁用于溯源。
- `packaging/windows/reimu-zip.ps1` 构建 Windows x64 便携包。它校验原版 ElyMark 安装包的 SHA-256，从中取同版本 PDF 工作进程和许可文件。
- 改动后运行 `cargo test --locked -p ely-mark --bin ely-mark`。PDF 导出测试需要 `ELY_PDF_WORKER` 指向有效工作进程。发布前从解压目录验证程序启动。
- GPUI 首帧以 `Theme::set_mode_now` 设置主题，根用 `FocusScope`。状态变更后通知界面，避免空闲重绘。
- 界面以布局和清晰的主操作为主。次要信息优先放入 tooltip。保留可访问名称、表单标签、键盘焦点和危险操作后果提示。
- 复用组件库，避免大框套小框。失败显示原因，日志不写正文、附件和凭证。代码文件不超过 500 行。
- 保留 main 历史，提交签名。每条任务独立审查。
