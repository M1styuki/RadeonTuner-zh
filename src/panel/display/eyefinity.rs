use super::Panel;
use crate::adl::display::Display;
use crate::panel::controls::card_rows;
use anyhow::{Context as _, Result};
use gpui_kit::component::{ActiveTheme as _, Sizable as _, alert::Alert, h_flex, v_flex};
use gpui_kit::{Context, Div, FontWeight, ParentElement as _, Styled as _, div};
pub(super) struct Editor {
    adapter: i32,
    displays: Vec<Display>,
    columns: i32,
    rows: i32,
    orientation: i32,
    state: Option<bool>,
    automatic: bool,
    status: String,
}
impl Panel {
    pub(super) fn open_eyefinity(&mut self) -> Result<()> {
        self.dropdown = None;
        let driver = self.driver.as_ref().context("AMD 驱动不可用")?;
        let adapter = self
            .display
            .displays
            .get(self.display.display_index)
            .context("未选择显示器")?
            .adapter;
        let displays = driver.display_connections(adapter)?;
        let columns = (displays.len() as i32).clamp(1, 6);
        let state = driver.eyefinity_state(adapter).ok().map(|(_, s)| s);
        let rows =
            crate::profile_store::load(&self.settings.profiles_path("DisplayProfiles.json")?)?;
        let automatic = displays
            .iter()
            .filter_map(|d| driver.display_profile_key(d))
            .any(|key| {
                crate::profiles::display_profiles(&rows, &key)
                    .iter()
                    .any(|row| {
                        row["Application"]
                            .as_str()
                            .is_some_and(|s| !s.eq_ignore_ascii_case("Global"))
                            && crate::profiles::automatic_eye(row)
                    })
            });
        self.display.eyefinity = Some(Editor {
            adapter,
            displays,
            columns,
            rows: 1,
            orientation: 0,
            state,
            automatic,
            status: String::new(),
        });
        Ok(())
    }
    fn eyefinity_action(&mut self, action: usize) -> Result<()> {
        let _write = crate::driver_write::lock()?;
        let editor = self.display.eyefinity.as_ref().context("未选择显示器")?;
        let driver = self.driver.as_ref().context("驱动不可用")?;
        match action {
            0 => {
                let ids = editor
                    .displays
                    .iter()
                    .map(|d| d.logical)
                    .collect::<Vec<_>>();
                driver.create_eyefinity(
                    editor.adapter,
                    editor.columns,
                    editor.rows,
                    editor.orientation,
                    &ids,
                )?;
            }
            1 | 2 => driver.set_eyefinity(editor.adapter, action == 1)?,
            _ => driver.remove_eyefinity(editor.adapter)?,
        }
        self.read_displays();
        self.open_eyefinity()
    }
    pub(in crate::panel) fn render_eyefinity_actions(&self, cx: &mut Context<Self>) -> Div {
        let Some(editor) = &self.display.eyefinity else {
            return h_flex();
        };
        let mut row = h_flex().flex_wrap().gap_2();
        for (i, label) in ["创建拼接配置", "启用", "禁用", "移除拼接配置"]
            .into_iter()
            .enumerate()
        {
            let available = match i {
                0 => {
                    editor.columns * editor.rows >= 2
                        && editor.columns + editor.rows <= 8
                        && (editor.columns * editor.rows) as usize == editor.displays.len()
                }
                1 => !editor.automatic && editor.state == Some(false),
                2 => !editor.automatic && editor.state == Some(true),
                _ => true,
            };
            let on_click =
                move |p: &mut Panel, _: &mut gpui_kit::Window, _: &mut Context<Panel>| {
                    let result = p.eyefinity_action(i);
                    let message = result
                        .map(|_| {
                            [
                                "已创建 Eyefinity",
                                "已启用 Eyefinity",
                                "已禁用 Eyefinity",
                                "已移除 Eyefinity",
                            ][i]
                                .to_owned()
                        })
                        .unwrap_or_else(|e| e.to_string());
                    if let Some(e) = p.display.eyefinity.as_mut() {
                        e.status = message;
                    } else {
                        p.display.status = message;
                    }
                };
            row = row.child(if i == 0 {
                self.primary_button(("eye-action", i), label, available, None, on_click, cx)
            } else {
                self.contour_button(("eye-action", i), label, available, None, on_click, cx)
            });
        }
        row
    }

    pub(in crate::panel) fn eyefinity_status(&self) -> &str {
        self.display
            .eyefinity
            .as_ref()
            .map_or("", |editor| editor.status.as_str())
    }

    pub(super) fn render_eyefinity(&self, editor: &Editor, cx: &mut Context<Self>) -> Div {
        let state_text = match editor.state {
            Some(true) => "已启用",
            Some(false) => "已禁用",
            None => "未检测到可用的拼接配置",
        };
        let mut config_rows = vec![
            self.setting_row(
                "当前状态",
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(cx.theme().foreground)
                    .child(state_text),
            ),
        ];
        for (axis, label, current) in [
            (0, "列数", editor.columns),
            (1, "行数", editor.rows),
            (2, "显示方向", editor.orientation),
        ] {
            let values: &[i32] = if axis == 2 {
                &[0, 90, 180, 270]
            } else {
                &[1, 2, 3, 4, 5, 6]
            };
            config_rows.push(
                self.setting_row(
                    label,
                    self.select(
                        ("eye-axis", axis as usize),
                        values
                            .iter()
                            .map(|v| {
                                if axis == 2 {
                                    format!("{v}°")
                                } else {
                                    v.to_string()
                                }
                            })
                            .collect(),
                        values.iter().position(|&v| v == current),
                        label,
                        crate::panel::controls::SELECT_MAX_WIDTH,
                        move |p, i, _| {
                            if let Some(e) = &mut p.display.eyefinity {
                                match axis {
                                    0 => e.columns = values[i],
                                    1 => e.rows = values[i],
                                    _ => e.orientation = values[i],
                                }
                            }
                        },
                        cx,
                    ),
                ),
            );
        }

        let mut sections: Vec<(&'static str, Div)> = vec![("拼接配置", card_rows(config_rows, cx))];
        if !editor.displays.is_empty() {
            let order_rows: Vec<_> = editor
                .displays
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    let mut row = h_flex().gap_1();
                    for (delta, id, label) in [(-1, "eye-up", "上移"), (1, "eye-down", "下移")]
                    {
                        if let Some(j) = i
                            .checked_add_signed(delta)
                            .filter(|n| *n < editor.displays.len())
                        {
                            row = row.child(self.inline_button(
                                (id, i),
                                label,
                                move |p, _, _| {
                                    if let Some(e) = p.display.eyefinity.as_mut() {
                                        e.displays.swap(i, j);
                                    }
                                },
                                cx,
                            ));
                        }
                    }
                    self.setting_row(format!("{}　{}（{}）", i + 1, d.name, d.logical), row)
                })
                .collect();
            sections.push(("显示器顺序（每列从上到下）", card_rows(order_rows, cx)));
        }

        let mut view = v_flex().gap_3();
        if editor.automatic {
            view = view.child(
                Alert::info(
                    "eyefinity-auto-notice",
                    "应用配置正在自动控制 Eyefinity。关闭配置中的自动 Eyefinity 后可手动切换。",
                )
                .small(),
            );
        }
        view.child(self.setting_sections_titled(sections, cx))
    }
}
