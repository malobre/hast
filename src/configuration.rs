#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Configuration {
    pub line_width: u32,
    pub indent_width: u8,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            line_width: 80,
            indent_width: 2,
        }
    }
}
