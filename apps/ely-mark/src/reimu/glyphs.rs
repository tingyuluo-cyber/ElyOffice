use gpui::{Hsla, Pixels, Styled as _, Svg, svg};

pub(crate) struct Glyph {
    light: &'static [u8],
    dark: &'static [u8],
}

impl Glyph {
    pub(crate) fn asset(&self, dark: bool) -> &'static [u8] {
        if dark { self.dark } else { self.light }
    }

    pub(crate) fn draw(&self, dark: bool, size: Pixels, color: Hsla) -> Svg {
        svg()
            .data(self.asset(dark))
            .size(size)
            .flex_none()
            .text_color(color)
    }
}

pub(crate) const APPEARANCE: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/moon.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/sun.svg"),
};
pub(crate) const MENU: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/menu.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/menu.svg"),
};
pub(crate) const PANEL: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/panel.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/panel.svg"),
};
pub(crate) const NEW: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/new.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/new.svg"),
};
pub(crate) const OPEN: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/open.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/open.svg"),
};
pub(crate) const SAVE: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/save.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/save.svg"),
};
pub(crate) const VISUAL: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/visual.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/visual.svg"),
};
pub(crate) const SOURCE: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/source.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/source.svg"),
};
pub(crate) const PREVIEW: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/preview.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/preview.svg"),
};
pub(crate) const WIDE: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/wide.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/wide.svg"),
};
pub(crate) const MINIMAP: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/minimap.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/minimap.svg"),
};
pub(crate) const FOLDER: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/folder.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/folder.svg"),
};
pub(crate) const OUTLINE: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/outline.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/outline.svg"),
};
pub(crate) const SEARCH: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/search.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/search.svg"),
};
pub(crate) const NOTE: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/note.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/note.svg"),
};
pub(crate) const DIRTY: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/dirty.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/dirty.svg"),
};
pub(crate) const CLOSE: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/close.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/close.svg"),
};
pub(crate) const BOW: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/bow.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/bow.svg"),
};
pub(crate) const SEAL: Glyph = Glyph {
    light: include_bytes!("../../assets/reimu/glyphs/seal.svg"),
    dark: include_bytes!("../../assets/reimu/glyphs/night/seal.svg"),
};
