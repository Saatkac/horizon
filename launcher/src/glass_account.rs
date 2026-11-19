//! Settings' Account card in the glass look, as the mockup's: who plays now (a big head, the
//! name, how), and the Microsoft card beside it with its white Sign in with Microsoft button.
//!
//! Not in the mockup, kept in its patterns: the Microsoft accounts already signed in (in the
//! Microsoft card, with Play as, Refresh and Sign out), the sign-in's code, progress and problems.

use super::*;
use crate::glass_ui::{self as ui, white, Tone};
use horizon_theme::glass::{ink, radius};

impl Launcher {
    pub(super) fn glass_account(&mut self, cx: &mut Context<Self>) -> Div {
        let wide = self.window_width >= crate::glass_home::WIDE;
        let offline = self.glass_offline_card(cx);
        // Signed out, "who plays" would only repeat the Microsoft card's call to sign in.
        if self.accounts.selected().is_none() {
            return div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(self.glass_microsoft_card(cx).w_full())
                .child(offline);
        }
        let left = self.glass_who_plays();
        let right = self.glass_microsoft_card(cx);
        let cards = if wide {
            div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .child(left.flex_1().min_w(px(280.)).p(px(20.)))
                .child(right.flex_none().w(px(360.)))
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(left.w_full().min_w_0().p(px(20.)))
                .child(right.w_full())
        };
        div().flex().flex_col().gap(px(8.)).child(cards).child(offline)
    }

    /// Adds the name typed into Settings' offline field: refused with the reason when it isn't
    /// a player name, or is already a profile.
    pub(super) fn add_offline_profile(&mut self) {
        let typed = self.offline_input.text().trim().to_owned();
        match self.offline.add(&typed) {
            Ok(_) => {
                self.offline_input.set(String::new());
                self.offline_error = self.offline.save().err();
                // The owner gate's screen would hide the new profile: Play reads it again.
                self.owner = None;
            }
            Err(why) => self.offline_error = Some(why),
        }
    }

    /// The offline profiles: a name plays without a Microsoft account, in single player, on the
    /// LAN and on offline-mode servers. The chosen one takes Play until the Microsoft account
    /// is chosen again.
    fn glass_offline_card(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let names = self.offline.profiles.clone();
        let selected = self.offline.selected_name().map(str::to_owned);
        let focused = self.offline_focused;
        let mut rows = div().flex().flex_col();
        for name in &names {
            let on = selected.as_deref() == Some(name.as_str());
            let pick = name.clone();
            let remove = name.clone();
            rows = rows.child(
                div()
                    .id(SharedString::from(format!("offline-{name}")))
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .h(px(40.))
                    .px(px(8.))
                    .rounded(px(radius::ROW))
                    .cursor_pointer()
                    .hover(|s| s.bg(white(0.05)))
                    .when(on, |d| d.bg(white(0.07)))
                    .child(ui::dot(if on { Tone::Live } else { Tone::Off }))
                    .child(
                        ui::text(14., FontWeight::MEDIUM, if on { 0.92 } else { 0.66 })
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(SharedString::from(name.clone())),
                    )
                    .when(on, |d| {
                        d.child(
                            ui::text(12., FontWeight::NORMAL, 0.45)
                                .child(tr("Plays now")),
                        )
                    })
                    // Removing a profile says so once, quietly: the list is a player's own names.
                    .child(
                        div()
                            .id(SharedString::from(format!("offline-remove-{name}")))
                            .flex_none()
                            .size(px(26.))
                            .rounded(px(7.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(white(0.09)))
                            .tooltip(ui::tip(tr("Remove this offline profile")))
                            .child(ui::icon("delete-02", 15., white(ink::NOTE)))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                cx.stop_propagation();
                                this.offline.remove(&remove);
                                this.offline_error = this.offline.save().err();
                                this.owner = None;
                                cx.notify();
                            })),
                    )
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.offline.select(Some(&pick));
                        if let Err(e) = this.offline.save() {
                            this.offline_error = Some(e);
                        } else {
                            this.offline_error = None;
                            this.owner = None;
                        }
                        cx.notify();
                    })),
            );
        }
        let field = div()
            .id("offline-name")
            .track_focus(&self.offline_focus)
            .flex()
            .items_center()
            .flex_1()
            .min_w_0()
            .h(px(40.))
            .px(px(12.))
            .rounded(px(radius::ROW))
            .border_1()
            .border_color(if focused { t.accent } else { white(ui::edge::REST) })
            .bg(white(0.04))
            .cursor_text()
            .child(line_edit::content(
                &self.offline_input,
                &t,
                focused,
                false,
                false,
                tr("Offline name"),
            ))
            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                window.focus(&this.offline_focus, cx);
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
                match line_edit::handle(&mut this.offline_input, ev, cx) {
                    line_edit::Key::Pass => return,
                    line_edit::Key::Escape => window.blur(cx),
                    line_edit::Key::Enter => this.add_offline_profile(),
                    _ => {}
                }
                cx.stop_propagation();
                cx.notify();
            }));
        let field = line_edit::mouse(field, cx, |this| &mut this.offline_input);
        let add = ui::button(
            "offline-add".into(),
            tr("Add").to_string().into(),
            Kind::Primary,
            &t,
        ).on_click(
            cx.listener(|this, _: &ClickEvent, _, cx| {
                this.add_offline_profile();
                cx.notify();
            }),
        );
        let microsoft = selected.map(|_| {
            ui::button(
                "offline-microsoft".into(),
                tr("Use Microsoft account").to_string().into(),
                Kind::Secondary,
                &t,
            )
            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                this.offline.select(None);
                if let Err(e) = this.offline.save() {
                    this.offline_error = Some(e);
                }
                cx.notify();
            }))
        });
        div()
            .flex()
            .flex_col()
            .rounded(px(16.))
            .bg(white(0.045))
            .border_1()
            .border_color(white(ui::edge::REST))
            .p(px(24.))
            .child(
                div()
                    .font_family(horizon_theme::glass::DISPLAY_FONT)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(18.))
                    .text_color(white(ink::FULL))
                    .child(tr("Offline profiles")),
            )
            .child(
                ui::text(14., FontWeight::NORMAL, 0.65)
                    .mt(px(6.))
                    .line_height(px(21.))
                    .child(tr(
                        "Play without a Microsoft account: single player, LAN, and servers \
                         that don't check accounts. Online-mode servers need Microsoft.",
                    )),
            )
            .when(!names.is_empty(), |d| d.child(rows.mt(px(14.))))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .mt(px(if names.is_empty() { 16. } else { 8. }))
                    .child(field)
                    .child(add),
            )
            .children(
                self.offline_error
                    .clone()
                    .map(|e| ui::text(12., FontWeight::NORMAL, 0.8).mt(px(8.)).child(e)),
            )
            .children(microsoft.map(|b| div().mt(px(12.)).child(b)))
    }

    /// The player now.
    fn glass_who_plays(&self) -> Div {
        let t = self.theme;
        let (uuid, name, status) = match (self.accounts.selected(), self.offline.selected_name()) {
            (_, Some(offline)) => (
                None,
                offline.to_owned(),
                (
                    Tone::Warn,
                    tr("Offline profile · single player and offline servers"),
                ),
            ),
            (Some(a), None) => (
                Some(a.uuid.clone()),
                a.name.clone(),
                (Tone::Live, tr("Microsoft account · online servers too")),
            ),
            (None, None) => (
                None,
                tr("Sign in with Microsoft").to_string(),
                (Tone::Off, tr(account::NEEDS_OWNER)),
            ),
        };
        let head = div()
            .flex_none()
            .size(px(56.))
            .rounded(px(14.))
            .overflow_hidden()
            .child(
                self.profile_heads
                    .draw(uuid.as_deref(), 56., self.scale, &t),
            );
        let header = div().flex().items_center().gap(px(16.)).child(head).child(
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .gap(px(10.))
                .child(
                    div()
                        .font_family(horizon_theme::glass::DISPLAY_FONT)
                        .font_weight(FontWeight::SEMIBOLD)
                        // A compact summary: the top bar and the account rows name the
                        // player too, so this is no page headline.
                        .text_size(px(22.))
                        .line_height(px(28.))
                        .text_color(white(ink::FULL))
                        .truncate()
                        .child(name),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap(px(8.))
                        .child(ui::dot(status.0).mt(px(7.)))
                        .child(
                            ui::text(14., FontWeight::NORMAL, 0.65)
                                .flex_1()
                                .min_w_0()
                                .line_height(px(21.))
                                .child(status.1),
                        ),
                ),
        );

        div().flex().flex_col().child(header)
    }

    /// The Microsoft card: the accounts signed in here, then adding one (the mockup's card).
    fn glass_microsoft_card(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let has_accounts = !self.accounts.accounts.is_empty();
        let (rows, _) = if has_accounts {
            ui::flat_rows(|| self.microsoft_rows(cx))
        } else {
            (div(), 0)
        };
        let (title, detail, action) = self.sign_in_parts(cx);
        let configured = account::client_id().is_some();
        let idle = self.sign_in_state.is_none()
            || matches!(self.sign_in_state, Some(SignInState::Done(_)));
        let first = !has_accounts && self.sign_in_target.is_none();
        let heading = if first {
            tr("Play online with your Microsoft account").to_string()
        } else {
            title
        };
        let note: AnyElement = if first && configured && self.sign_in_state.is_none() {
            div()
                .child(tr(
                    "Sign in with the account that owns Minecraft: Java Edition. Your name and \
                     skin come along to online servers, like in other launchers.",
                ))
                .into_any_element()
        } else {
            detail.into_any_element()
        };
        // Adding an account is the mockup's white button; signing one in again, the code and
        // the waiting keep their own buttons.
        let button: Option<AnyElement> = if configured && idle && self.sign_in_target.is_none() {
            Some(
                keyboard(
                    div()
                        .id("glass-sign-in")
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(10.))
                        .h(px(48.))
                        .w_full()
                        .mt(px(24.))
                        .rounded(px(radius::ROW))
                        .bg(gpui::white())
                        // On a light plate, Microsoft's white button keeps an edge.
                        .when(ui::light(), |d| d.border_1().border_color(ui::white(0.15)))
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.9))
                        .font_family(horizon_theme::glass::FONT)
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(gpui::rgb(0x100a1f))
                        .child(microsoft::logo(18.))
                        .child(tr("Sign in with Microsoft")),
                    &t,
                )
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.start_sign_in();
                    cx.notify();
                }))
                .into_any_element(),
            )
        } else {
            action.map(|a| div().mt(px(20.)).child(a).into_any_element())
        };
        div()
            .flex()
            .flex_col()
            .rounded(px(16.))
            .bg(white(0.045))
            .border_1()
            .border_color(white(ui::edge::REST))
            .p(px(24.))
            .when(has_accounts, |d| {
                d.child(rows.mx(px(-16.)).mt(px(-16.)).mb(px(16.)))
            })
            .child(microsoft::logo(36.))
            .child(
                div()
                    .mt(px(20.))
                    .font_family(horizon_theme::glass::DISPLAY_FONT)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(22.))
                    .line_height(px(27.5))
                    .text_color(white(ink::FULL))
                    .child(heading),
            )
            .child(
                ui::text(14., FontWeight::NORMAL, 0.65)
                    .mt(px(8.))
                    .line_height(px(21.7))
                    .child(note),
            )
            .children(button)
            .when(configured, |d| {
                d.child(
                    ui::text(12., FontWeight::NORMAL, 0.45)
                        .mt(px(12.))
                        .line_height(px(18.))
                        .child(tr(
                            "Microsoft's own page opens in your browser. Horizon never sees your \
                             password.",
                        )),
                )
            })
    }
}
