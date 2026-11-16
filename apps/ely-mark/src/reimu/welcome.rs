use ely_gpui_component::{
    buttons::{Button, ButtonVariant},
    primitives::{FocusRing, IconName, Tooltip},
    theme::{ActiveTheme as _, ControlSize, Radius},
};
use ely_ui::{commands, i18n::text};
use gpui::{
    App, Div, FontWeight, InteractiveElement as _, MouseButton, ObjectFit, ParentElement as _,
    Role, StatefulInteractiveElement as _, Styled as _, StyledImage as _, div, img, px, relative,
};

use super::{NAME, art::Art};

pub(crate) fn welcome(art: Art, pick: impl Fn(Art, &mut App) + Clone + 'static, cx: &App) -> Div {
    let theme = cx.theme();
    let new = Button::new("reimu-home-new", text(cx, "command.file.new", &[]))
        .icon(IconName::Plus)
        .size(ControlSize::Lg)
        .variant(ButtonVariant::Primary)
        .on_click(|_, window, cx| window.dispatch_action(Box::new(commands::New), cx));
    let open = Button::new("reimu-home-open", text(cx, "command.file.open", &[]))
        .icon(IconName::FolderOpen)
        .size(ControlSize::Lg)
        .variant(ButtonVariant::Outline)
        .on_click(|_, window, cx| window.dispatch_action(Box::new(commands::Open), cx));
    let title = div()
        .id("reimu-title")
        .role(Role::Heading)
        .aria_level(1)
        .text_size(px(44.0))
        .line_height(px(54.0))
        .font_weight(FontWeight::BOLD)
        .text_color(theme.colors.fg)
        .child(NAME);
    let actions = div()
        .flex()
        .flex_wrap()
        .gap_2()
        .child(div().debug_selector(|| "reimu-home-new".into()).child(new))
        .child(open);
    let intro = div()
        .flex()
        .flex_col()
        .flex_basis(relative(0.40))
        .min_w(px(184.0))
        .justify_center()
        .items_center()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(div().w_7().h(px(3.0)).bg(theme.colors.accent))
                .child(title)
                .child(div().pt_2().child(actions)),
        );
    let cover = div()
        .debug_selector(|| "reimu-cover".into())
        .relative()
        .flex_1()
        .w_full()
        .min_h_0()
        .child(
            img(art.image())
                .debug_selector(move || format!("reimu-cover-{}", art.key()))
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .rounded(theme.radius(Radius::Xl))
                .object_fit(ObjectFit::Contain),
        );
    let slides = div()
        .id("reimu-cover-slides")
        .role(Role::RadioGroup)
        .flex()
        .justify_center()
        .flex_none()
        .children(Art::ALL.map(|choice| {
            let pick = pick.clone();
            let key = choice.key();
            let selected = choice == art;
            let label = text(cx, &format!("mark.art.{key}"), &[]);
            let ink = if selected {
                theme.colors.accent
            } else {
                theme.colors.fg_muted
            };
            div()
                .id(key)
                .debug_selector(move || format!("reimu-art-{key}"))
                .role(Role::RadioButton)
                .aria_label(label.clone())
                .aria_toggled(selected.into())
                .flex()
                .items_center()
                .justify_center()
                .size(theme.control_height(ControlSize::Sm))
                .rounded(theme.radius(Radius::Sm))
                .border_1()
                .border_color(gpui::transparent_black())
                .focus_ring(cx)
                .tab_index(0)
                .cursor_pointer()
                .hover(|style| style.bg(theme.colors.hover))
                .child(
                    div()
                        .rounded_full()
                        .h(px(7.0))
                        .w(px(if selected { 13.0 } else { 7.0 }))
                        .bg(ink),
                )
                .tooltip(Tooltip::text(label))
                .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                .on_click(move |_, _, cx| pick(choice, cx))
        }));
    div()
        .size_full()
        .flex()
        .justify_center()
        .p_4()
        .bg(theme.colors.bg)
        .child(
            div()
                .size_full()
                .max_w(px(1200.0))
                .flex()
                .gap_4()
                .child(intro)
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(cover)
                        .child(slides),
                ),
        )
}
