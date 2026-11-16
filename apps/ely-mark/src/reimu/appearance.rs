use ely_gpui_component::theme::ActiveTheme as _;
use ely_platform::settings::{APPEARANCE, Appearance, word};
use gpui::{App, Hsla, rgb};

pub(crate) fn install(cx: &mut App) -> anyhow::Result<()> {
    let palette =
        |source| ely_ui::parse_omarchy_theme(source, None, false).map(|theme| theme.palette);
    let light = palette(include_str!("../../assets/reimu/theme/colors.toml"))?;
    let mut dark = palette(include_str!("../../assets/reimu/theme/nord.toml"))?;
    let nord = |value| -> Hsla { rgb(value).into() };
    dark.surface = nord(0x3b4252);
    dark.overlay = dark.surface;
    dark.sunken = dark.bg;
    dark.hover = nord(0x434c5e);
    dark.active = dark.hover;
    dark.border = dark.hover;
    dark.border_strong = nord(0x4c566a);
    ely_ui::set_paired_palettes(light, dark, cx);
    Ok(())
}

pub(super) fn toggle(cx: &mut App) {
    let next = if cx.theme().is_dark() {
        Appearance::Light
    } else {
        Appearance::Dark
    };
    ely_ui::settings::set(APPEARANCE, word(&Appearance::ALL, &next).into(), cx);
}
