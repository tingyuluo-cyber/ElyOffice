use std::sync::{Arc, OnceLock};

use ely_gpui_component::theme::ActiveTheme as _;
use gpui::{App, Bounds, Image, ImageFormat, Pixels, Size, point, px, size};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Art {
    Reimu,
    Yukari,
}

impl Art {
    pub(super) const ALL: [Self; 2] = [Self::Reimu, Self::Yukari];

    pub(crate) fn for_mode(cx: &App) -> Self {
        if cx.theme().is_dark() {
            Self::Yukari
        } else {
            Self::Reimu
        }
    }

    pub(super) fn key(self) -> &'static str {
        match self {
            Self::Reimu => "reimu",
            Self::Yukari => "yukari",
        }
    }

    pub(super) fn image(self) -> Arc<Image> {
        static REIMU: OnceLock<Arc<Image>> = OnceLock::new();
        static YUKARI: OnceLock<Arc<Image>> = OnceLock::new();
        let (slot, bytes): (_, &[u8]) = match self {
            Self::Reimu => (
                &REIMU,
                include_bytes!("../../assets/reimu/reimu-daylight.png"),
            ),
            Self::Yukari => (
                &YUKARI,
                include_bytes!("../../assets/reimu/yukari-nord.png"),
            ),
        };
        slot.get_or_init(|| Arc::new(Image::from_bytes(ImageFormat::Png, bytes.to_vec())))
            .clone()
    }

    pub(super) fn wallpaper_bounds(
        self,
        viewport: Bounds<Pixels>,
        source_size: Size<Pixels>,
    ) -> Bounds<Pixels> {
        // 共用眼线基准。
        let frame = size(px(1448.0), px(960.0));
        let scale = (viewport.size.width / frame.width).max(viewport.size.height / frame.height);
        let min_left = (viewport.size.width - frame.width * scale).min(px(0.0));
        let min_top = (viewport.size.height - frame.height * scale).min(px(0.0));
        let left = (viewport.size.width * 0.64 - px(924.0) * scale).clamp(min_left, px(0.0));
        let top = (viewport.size.height * 0.4 - px(313.0) * scale).clamp(min_top, px(0.0));
        let origin = viewport.origin + point(left, top);
        let (crop_origin, crop_scale) = match self {
            Self::Reimu => (point(px(0.0), px(105.0)), 1.0),
            Self::Yukari => (point(px(113.0), px(0.0)), 0.82),
        };
        let image_scale = scale / crop_scale;
        Bounds {
            origin: origin - crop_origin * image_scale,
            size: source_size.map(|dimension| dimension * image_scale),
        }
    }
}
