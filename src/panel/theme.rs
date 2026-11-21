#[derive(Clone, Copy)]
pub(super) struct Palette {
    pub canvas: u32,
    pub text: u32,
    pub sidebar: u32,
    pub control: u32,
    pub selected: u32,
    pub selection: u32,
    pub border: u32,
    pub grid: u32,
    pub accent: u32,
}
impl Palette {
    pub fn resolve(style: i64, system_light: bool) -> Self {
        if style == 2 || (style == 3 && system_light) {
            Self {
                canvas: 0xfcfcfc,
                text: 0x24272b,
                sidebar: 0xf0f1f3,
                control: 0xe4e6e9,
                selected: 0xf0cdd0,
                selection: 0xe6afb5,
                border: 0x969ba1,
                grid: 0xd5d8dc,
                accent: 0xac3947,
            }
        } else {
            Self {
                canvas: if style == 1 { 0 } else { 0x171a20 },
                text: 0xe4e6eb,
                sidebar: 0x202328,
                control: 0x292d34,
                selected: 0x713e47,
                selection: 0x80525b,
                border: 0x555b63,
                grid: 0x3c424a,
                accent: 0xc85c63,
            }
        }
    }
}
