use super::DetectedGameLanguage;
use std::io::Write;
use std::path::Path;

pub(super) fn show_setup_message_to(writer: &mut impl Write, config_path: &str, exe_path: &str) {
    let message = format!(
        "\n\
         ╔═══════════════════════════════════════════════╗\n\
         ║   Slay the Spire AI Copilot — 尚未配置       ║\n\
         ╚═══════════════════════════════════════════════╝\n\n\
         尚未找到指向当前二进制文件的 Communication Mod CJK 配置。\n\n\
         配置步骤：\n\
         1. 安装 ModTheSpire 和 Communication Mod CJK\n\
            https://github.com/kiooeht/ModTheSpire\n\
            Communication Mod CJK:\n\
            https://steamcommunity.com/sharedfiles/filedetails/?id=3748153752\n\
            https://github.com/ArtemisSaber/CommunicationMod/releases\n\n\
         2. 编辑配置文件：\n\
            {config_path}\n\n\
         3. 添加以下内容（确保 command 指向当前二进制文件）：\n\
            command={exe_path}\n\
            runAtGameStart=true\n\n\
         4. 通过 ModTheSpire 启动游戏并启用 Communication Mod CJK\n\n\
         也可以手动测试（mock provider）：\n\
            echo '{{\"in_game\":true,...}}' | {exe_path}\n"
    );

    let _ = writeln!(writer, "{message}");
    let _ = writer.flush();
}

pub(super) fn show_cjk_mod_language_message_to(
    writer: &mut impl Write,
    language: &DetectedGameLanguage,
    config_path: &Path,
    cjk_config_path: &Path,
) {
    let message = format!(
        "\n\
         检测到 Slay the Spire 当前语言为 `{}`（来源：{}）。\n\
         这个语言需要非 ASCII 文本支持，请改用 Communication Mod CJK。\n\n\
         当前配置路径：{}\n\
         推荐配置路径：{}\n\n\
         请在 ModTheSpire 中启用 Communication Mod CJK，然后重新启动游戏。\n",
        language.value,
        language.source,
        config_path.display(),
        cjk_config_path.display()
    );

    let _ = writeln!(writer, "{message}");
    let _ = writer.flush();
}

pub(super) fn show_setup_message(config_path: &str, exe_path: &str) {
    show_setup_message_to(&mut std::io::stdout().lock(), config_path, exe_path);
}

pub(super) fn restart_hint() {
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "请通过 ModTheSpire 重新启动游戏以加载新配置。");
    let _ = stdout.flush();
}
