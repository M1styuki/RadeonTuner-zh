#[derive(Clone, Copy)]
pub(super) struct Palette {
    pub light: bool,
    pub hover: u32,
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
                light: true,
                hover: 0xe9edf3,
                canvas: 0xffffff,
                text: 0x1b2028,
                sidebar: 0xeef1f5,
                control: 0xffffff,
                selected: 0xf0cdd0,
                selection: 0xe6afb5,
                border: 0xd3d9e2,
                grid: 0xd5d8dc,
                accent: 0xb84b52,
            }
        } else {
            Self {
                light: false,
                hover: 0x2b323d,
                canvas: if style == 1 { 0 } else { 0x15181d },
                text: 0xe6ebf2,
                sidebar: 0x121418,
                control: 0x20252d,
                selected: 0x713e47,
                selection: 0x80525b,
                border: 0x3a424e,
                grid: 0x3c424a,
                accent: 0xc85c63,
            }
        }
    }
}
