use strum_macros::EnumString;

#[derive(Debug, EnumString)]
pub enum AfterSearchOption {
    #[strum(serialize = "Show All")]
    ShowAll,
    Filter,
}

impl AfterSearchOption {
    pub const VARIANTS: [&'static str; 2] = ["Show All", "Filter"];
}
