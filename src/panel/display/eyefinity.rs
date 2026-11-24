use super::Panel;
use crate::adl::display::Display;
use anyhow::{Context as _, Result};
use gpui::{Context, Div, div, prelude::*};
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
    pub(super) fn render_eyefinity(&self, editor: &Editor, cx: &mut Context<Self>) -> Div {
        let mut view = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(div().text_xl().child("管理 Eyefinity"))
            .child(match editor.state {
                Some(true) => "当前已启用",
                Some(false) => "当前已禁用",
                None => "未检测到可用的拼接配置",
            });
        if editor.automatic {
            view = view
                .child("应用配置正在自动控制 Eyefinity。关闭配置中的自动 Eyefinity 后可手动切换。");
        }
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
            view = view.child(
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
        view = view.child("显示器顺序（每列从上到下）");
        for (i, d) in editor.displays.iter().enumerate() {
            let mut row =
                div()
                    .flex()
                    .gap_2()
                    .child(format!("{}　{}（{}）", i + 1, d.name, d.logical));
            for (delta, id, label) in [(-1, "eye-up", "上移"), (1, "eye-down", "下移")] {
                if let Some(j) = i
                    .checked_add_signed(delta)
                    .filter(|n| *n < editor.displays.len())
                {
                    row = row.child(
                        self.inline_action()
                            .id((id, i))
                            .px_2()
                            .child(label)
                            .on_click(cx.listener(move |p, _, _, cx| {
                                if let Some(e) = p.display.eyefinity.as_mut() {
                                    e.displays.swap(i, j);
                                }
                                cx.notify();
                            })),
                    );
                }
            }
            view = view.child(row);
        }
        let mut row = div().flex().flex_wrap().gap_2();
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
            let button = self
                .button_style(available)
                .id(("eye-action", i))
                .child(label);
            row = row.child(if available {
                button
                    .cursor_pointer()
                    .on_click(cx.listener(move |p, _, _, cx| {
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
                        cx.notify();
                    }))
            } else {
                button
            });
        }
        view.child(row).child(editor.status.clone()).child(
            self.inline_action()
                .id("eye-close")
                .child("返回显示设置")
                .on_click(cx.listener(|p, _, _, cx| {
                    p.display.eyefinity = None;
                    cx.notify();
                })),
        )
    }
}
