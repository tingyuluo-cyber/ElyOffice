/// The eight apps, in PRD order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppId {
    Doc,
    Table,
    Show,
    Pdf,
    Mail,
    Tree,
    Mark,
    Agent,
}

impl AppId {
    pub const ALL: [AppId; 8] = [
        AppId::Doc,
        AppId::Table,
        AppId::Show,
        AppId::Pdf,
        AppId::Mail,
        AppId::Tree,
        AppId::Mark,
        AppId::Agent,
    ];

    /// The macOS bundle identifier used by upstream.
    pub fn bundle_id(self) -> String {
        format!("ely.zacharyzhang.office.{}", &self.slug()["ely-".len()..])
    }

    pub fn name(self) -> &'static str {
        match self {
            AppId::Doc => "ElyDoc",
            AppId::Table => "ElyTable",
            AppId::Show => "ElyShow",
            AppId::Pdf => "ElyPDF",
            AppId::Mail => "ElyMail",
            AppId::Tree => "ElyTree",
            AppId::Mark => "ElyMark",
            AppId::Agent => "ElyAgent",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            AppId::Doc => "ely-doc",
            AppId::Table => "ely-table",
            AppId::Show => "ely-show",
            AppId::Pdf => "ely-pdf",
            AppId::Mail => "ely-mail",
            AppId::Tree => "ely-tree",
            AppId::Mark => "ely-mark",
            AppId::Agent => "ely-agent",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppId::{self, *};

    const IN_PRD_ORDER: [AppId; 8] = AppId::ALL;

    #[test]
    fn names_follow_the_prd() {
        let names: Vec<_> = IN_PRD_ORDER.iter().map(|app| app.name()).collect();
        assert_eq!(
            names,
            [
                "ElyDoc", "ElyTable", "ElyShow", "ElyPDF", "ElyMail", "ElyTree", "ElyMark",
                "ElyAgent"
            ]
        );
    }

    #[test]
    fn slugs_are_the_binary_names() {
        let slugs: Vec<_> = IN_PRD_ORDER.iter().map(|app| app.slug()).collect();
        assert_eq!(
            slugs,
            [
                "ely-doc",
                "ely-table",
                "ely-show",
                "ely-pdf",
                "ely-mail",
                "ely-tree",
                "ely-mark",
                "ely-agent"
            ]
        );
    }
}
