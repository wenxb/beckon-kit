use gpui::{ClipboardItem, Context, KeyDownEvent, Window};

use super::{TextField, TextFieldEvent};

impl TextField {
    pub(super) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || event.prefer_character_input || self.buffer.composing() {
            return;
        }
        let modifiers = event.keystroke.modifiers;
        let key = event.keystroke.key.as_str();
        let editable = !self.read_only;
        if key == "enter" && self.buffer.multiline() {
            if modifiers.control || modifiers.platform {
                cx.emit(TextFieldEvent::Submitted(
                    self.buffer.text().to_owned().into(),
                ));
            } else if !modifiers.alt && editable && self.buffer.replace(None, "\n") {
                self.changed(cx);
            }
            cx.notify();
            cx.stop_propagation();
            return;
        }
        if modifiers.secondary() && !modifiers.alt {
            match key {
                "a" => self.buffer.select_all(),
                "c" | "x" => {
                    let selection = self.buffer.selection();
                    if !self.masked && !selection.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(
                            self.buffer.text()[selection].into(),
                        ));
                        if key == "x" && editable && self.buffer.replace(None, "") {
                            self.changed(cx);
                        }
                    }
                }
                "v" if editable => {
                    if let Some(item) = cx.read_from_clipboard()
                        && let Some(text) = item.text()
                        && self.buffer.replace(None, &text)
                    {
                        self.changed(cx);
                    }
                }
                "z" if editable => {
                    let changed = if modifiers.shift {
                        self.buffer.redo()
                    } else {
                        self.buffer.undo()
                    };
                    if changed {
                        self.changed(cx);
                    }
                }
                "y" if editable && !cfg!(target_os = "macos") => {
                    if self.buffer.redo() {
                        self.changed(cx);
                    }
                }
                "backspace" | "delete" if editable => {
                    if cfg!(target_os = "macos") {
                        let position = if self.buffer.multiline() {
                            self.geometry
                                .as_ref()
                                .map_or(self.buffer.head(), |geometry| {
                                    geometry.row_edge(self.buffer.head(), key == "delete")
                                })
                        } else if key == "delete" {
                            self.buffer.text().len()
                        } else {
                            0
                        };
                        self.buffer.move_to(position, true);
                        if self.buffer.replace(None, "") {
                            self.changed(cx);
                        }
                    } else if self.buffer.delete(key == "delete", true) {
                        self.changed(cx);
                    }
                }
                _ => {
                    if !self.navigation(key, modifiers) {
                        return;
                    }
                }
            }
        } else if !modifiers.platform && (!modifiers.alt || cfg!(target_os = "macos")) {
            match key {
                "backspace" if editable => {
                    let word = if cfg!(target_os = "macos") {
                        modifiers.alt
                    } else {
                        modifiers.control
                    };
                    if self.buffer.delete(false, word) {
                        self.changed(cx);
                    }
                }
                "delete" if editable => {
                    let word = if cfg!(target_os = "macos") {
                        modifiers.alt
                    } else {
                        modifiers.control
                    };
                    if self.buffer.delete(true, word) {
                        self.changed(cx);
                    }
                }
                "enter" if !modifiers.modified() => cx.emit(TextFieldEvent::Submitted(
                    self.buffer.text().to_owned().into(),
                )),
                _ => {
                    if !self.navigation(key, modifiers) {
                        return;
                    }
                }
            }
        } else {
            return;
        }
        cx.notify();
        self.reveal_caret = true;
        cx.stop_propagation();
    }

    fn navigation(&mut self, key: &str, modifiers: gpui::Modifiers) -> bool {
        let word = if cfg!(target_os = "macos") {
            modifiers.alt
        } else {
            modifiers.control
        };
        let line = cfg!(target_os = "macos") && modifiers.platform;
        if self.buffer.multiline() {
            if matches!(key, "up" | "down" | "pageup" | "pagedown") {
                if let Some(geometry) = &self.geometry {
                    let position = geometry.position(self.buffer.head());
                    let x = *self
                        .preferred_x
                        .get_or_insert(position.x - geometry.origin.x);
                    let distance = if matches!(key, "pageup" | "pagedown") {
                        geometry.bounds.size.height
                    } else {
                        geometry.line_height
                    };
                    let y = position.y
                        + if matches!(key, "down" | "pagedown") {
                            distance
                        } else {
                            -distance
                        };
                    let index = geometry.index_at(gpui::point(
                        geometry.origin.x + x,
                        y + geometry.line_height / 2.0,
                    ));
                    self.buffer
                        .move_to(self.buffer.snap_grapheme(index), modifiers.shift);
                }
                return true;
            }
            if (matches!(key, "home" | "end") && !modifiers.control && !modifiers.platform)
                || (line && matches!(key, "left" | "right"))
            {
                if let Some(geometry) = &self.geometry {
                    let end = matches!(key, "end" | "right");
                    self.buffer
                        .move_to(geometry.row_edge(self.buffer.head(), end), modifiers.shift);
                }
                self.preferred_x = None;
                return true;
            }
        }
        self.preferred_x = None;
        match key {
            "left" if line => self.buffer.move_to(0, modifiers.shift),
            "right" if line => self
                .buffer
                .move_to(self.buffer.text().len(), modifiers.shift),
            "left" => self.buffer.move_horizontal(false, modifiers.shift, word),
            "right" => self.buffer.move_horizontal(true, modifiers.shift, word),
            "home" => self.buffer.move_to(0, modifiers.shift),
            "end" => self
                .buffer
                .move_to(self.buffer.text().len(), modifiers.shift),
            _ => return false,
        }
        true
    }
}
