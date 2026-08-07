//! TUI configuration interface — MD3-themed ratatui UI.
//!
//! Replaces the legacy `config_tui_old.rs` (8400-line raw crossterm).

mod theme;
pub mod pages;

use crate::config::{ActiveProviderModelConfig, AppConfig};
use crate::llm::ThinkingVariantPreferences;
use crate::paths::PersonaPaths;
use anyhow::Result;
use pages::t;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};
pub fn run(paths: &PersonaPaths) -> Result<bool> {
    AppConfig::init_files(paths)?;
    crate::models_cache::try_load(paths);
    crate::models_cache::spawn_background_refresh(paths.clone());
    let config = AppConfig::load_or_default(paths)?;
    let thinking_variants = ThinkingVariantPreferences::load(paths);
    App::new(paths, config, thinking_variants)?.run()
}

// ── Application state ──────────────────────────────────────────────────

enum Screen {
    MainMenu,
    TextModel,
    MultimodalModel,
    SubagentTiers,
    Providers,
    Plugins,
    Prompts,
    Platforms,
    GlobalSettings,
}

struct App<'a> {
    paths: &'a PersonaPaths,
    config: AppConfig,
    thinking_variants: ThinkingVariantPreferences,
    screen: Screen,
    main_menu_state: ListState,
    quit: bool,
    dirty: bool,
    pristine_config: String,
    // Sub-pages
    text_model: pages::text_model::TextModelPage,
    multimodal: pages::multimodal::MultimodalPage,
    subagent: pages::subagent::SubagentPage,
    providers: pages::providers::ProvidersPage,
    plugins: pages::plugins::PluginsPage,
    prompts: pages::prompts::PromptsPage,
    platforms: pages::platforms::PlatformsPage,
    global: pages::global::GlobalPage,
}

impl<'a> App<'a> {
    fn new(
        paths: &'a PersonaPaths,
        config: AppConfig,
        thinking_variants: ThinkingVariantPreferences,
    ) -> Result<Self> {
        let pristine_config =
            serde_json::to_string(&config).unwrap_or_default();
        let mut main_menu_state = ListState::default();
        main_menu_state.select(Some(0));
        Ok(Self {
            paths,
            config,
            thinking_variants,
            screen: Screen::MainMenu,
            main_menu_state,
            quit: false,
            dirty: false,
            pristine_config,
            text_model: pages::text_model::TextModelPage::new(),
            multimodal: pages::multimodal::MultimodalPage::default(),
            subagent: pages::subagent::SubagentPage::default(),
            providers: pages::providers::ProvidersPage::new(),
            plugins: pages::plugins::PluginsPage::new(),
            prompts: pages::prompts::PromptsPage::new(),
            platforms: pages::platforms::PlatformsPage::new(),
            global: pages::global::GlobalPage::default(),
        })
    }

    fn run(mut self) -> Result<bool> {
        let mut terminal = ratatui::init();
        let result = self.run_loop(&mut terminal);
        ratatui::restore();
        result
    }

    fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<bool> {
        while !self.quit {
            terminal.draw(|frame| self.draw(frame))?;
            if let Some(event) = self.handle_input()? {
                match event {
                    AppEvent::SelectMainMenu(idx) => {
                        self.main_menu_state.select(Some(idx));
                        match idx {
                            0 => self.screen = Screen::TextModel,
                            1 => self.screen = Screen::MultimodalModel,
                            2 => self.screen = Screen::SubagentTiers,
                            3 => self.screen = Screen::Providers,
                            4 => self.screen = Screen::Plugins,
                            5 => self.screen = Screen::Prompts,
                            6 => self.screen = Screen::Platforms,
                            7 => self.screen = Screen::GlobalSettings,
                            8 => {
                                self.quit = true;
                                self.config.save(self.paths)?;
                                self.thinking_variants.save(self.paths)?;
                                return Ok(true);
                            }
                            _ => {}
                        }
                    }
                    AppEvent::Back => self.screen = Screen::MainMenu,
                    AppEvent::None => {}
                    AppEvent::Quit => {
                        self.quit = true;
                        if self.dirty
                            || self.thinking_variants.is_dirty()
                            || serde_json::to_string(&self.config).ok()
                                .as_deref()
                                != Some(&self.pristine_config)
                        {
                            self.config.save(self.paths)?;
                            self.thinking_variants.save(self.paths)?;
                            return Ok(true);
                        }
                        return Ok(false);
                    }
                }
            }
        }
        Ok(false)
    }

    // ── Drawing ────────────────────────────────────────────────────────

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let theme = theme::md3_theme();

        // Status bar at bottom
        let main_area = Rect {
            height: area.height.saturating_sub(1),
            ..area
        };
        let status_area = Rect {
            y: area.height.saturating_sub(1),
            height: 1,
            ..area
        };

        match self.screen {
            Screen::MainMenu => self.draw_main_menu(frame, main_area, &theme),
            Screen::TextModel => self.draw_text_model(frame, main_area, &theme),
            Screen::MultimodalModel => self.draw_multimodal(frame, main_area, &theme),
            Screen::SubagentTiers => self.draw_subagent(frame, main_area, &theme),
            Screen::Providers => self.draw_providers(frame, main_area, &theme),
            Screen::GlobalSettings => self.draw_global(frame, main_area, &theme),
            Screen::Plugins => self.draw_plugins(frame, main_area, &theme),
            Screen::Platforms => self.draw_platforms(frame, main_area, &theme),
            Screen::Prompts => self.draw_prompts(frame, main_area, &theme),
        }

        // ── Status bar ─────────────────────────────────────────────────
        let help_text = match self.screen {
            Screen::MainMenu => t(" q/Esc quit   ↑↓ navigate   Enter select ", " q/Esc 退出   ↑↓ 导航   Enter 选择 "),
            _ => t(" Esc back  ↑↓/PgUp/PgDn navigate  Enter confirm  ←/→ choice ", " Esc 返回  ↑↓/翻页 导航  Enter 确认  ←/→ 切换选项 "),
        };
        let help = Span::styled(
            help_text,
            Style::default()
                .fg(theme.surface_dim_fg)
                .bg(theme.surface_dim_bg),
        );
        let mode = Span::styled(
            format!(" {} ", screen_title(&self.screen)),
            Style::default()
                .fg(theme.primary_fg)
                .bg(theme.primary_bg)
                .add_modifier(Modifier::BOLD),
        );
        let status = Line::from(vec![mode, help]);
        frame.render_widget(
            Paragraph::new(status).style(Style::default().bg(theme.surface_dim_bg)),
            status_area,
        );
    }

    fn draw_main_menu(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(52),
                Constraint::Fill(1),
            ])
            .split(area);

        let centered = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(22), Constraint::Fill(1)])
            .split(horizontal[1]);

        let active = active_label(&self.config);
        let multimodal = active_multimodal_label(&self.config);

        let items: Vec<ListItem> = vec![
            format!("  {} ({})", t("Text model", "文本模型"), active),
            format!("  {} ({})", t("Multimodal model", "多模态模型"), multimodal),
            format!("  {} ({})", t("Subagent tier pools", "子代理档位池"), subagent_tiers_label(&self.config)),
            format!("  {}", t("Providers and models", "供应商和模型")),
            format!("  {}", t("Plugins", "插件配置")),
            format!("  {}", t("Custom prompts", "自定义提示词")),
            format!("  {} ({})", t("Platforms", "接入平台"), platforms_label(&self.config)),
            format!("  {}", t("Global settings", "全局参数设置")),
            format!("  {}", t("Save and quit", "保存并退出")),
        ]
        .into_iter()
        .map(|s| ListItem::new(s).style(Style::default().fg(theme.on_surface)))
        .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(" POS 配置 ")
                    .title_alignment(Alignment::Center)
                    .border_style(Style::default().fg(theme.outline))
                    .style(Style::default().bg(theme.surface_bg)),
            )
            .highlight_style(
                Style::default()
                    .fg(theme.primary_fg)
                    .bg(theme.primary_container_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▌ ");

        frame.render_stateful_widget(list, centered[1], &mut self.main_menu_state);
    }

    fn draw_text_model(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let labels: Vec<String> = self
            .config
            .active_provider_model_choices()
            .iter()
            .map(|c| c.label())
            .collect();
        draw_model_select(
            frame,
            area,
            theme,
            "文本模型",
            &labels,
            &active_label(&self.config),
            self.text_model.editing,
            &self.text_model.edit_buffer,
            &mut self.text_model.state,
        );
    }

    fn draw_multimodal(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let labels: Vec<String> = self
            .config
            .active_multimodal_provider_model_choices()
            .iter()
            .map(|c| c.label())
            .collect();
        draw_model_select(
            frame,
            area,
            theme,
            "多模态模型",
            &labels,
            &active_multimodal_label(&self.config),
            self.multimodal.editing,
            &self.multimodal.edit_buffer,
            &mut self.multimodal.state,
        );
    }

    fn draw_subagent(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let tier = self.subagent.active_tier();
        let labels: Vec<String> = self
            .config
            .provider_model_choices()
            .into_iter()
            .map(|choice| {
                let selected = self.config.is_subagent_tier_model(
                    tier,
                    &choice.provider_id,
                    &choice.model,
                );
                format!(
                    "{} {}",
                    pages::subagent::choice_mark(selected),
                    choice.label()
                )
            })
            .collect();

        let inner = Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(3),
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!(
                " 子代理档位池 — {} ({}) ",
                pages::subagent::tier_labels()[self.subagent.tab_index.clamp(0, 2)],
                pages::subagent::tier_hint(tier)
            ))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(theme.outline))
            .style(Style::default().bg(theme.surface_bg));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner);

        // ── Tabs ────────────────────────────────────────────────────────
        let tabs = ratatui::widgets::Tabs::new(pages::subagent::tier_labels())
            .block(Block::bordered().border_type(BorderType::Rounded))
            .style(Style::default().fg(theme.on_surface_variant))
            .highlight_style(
                Style::default()
                    .fg(theme.primary_fg)
                    .bg(theme.primary_container_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .select(self.subagent.tab_index.clamp(0, 2));
        frame.render_widget(tabs, layout[0]);

        let hint = Line::from(format!(
            " Tab/←→ 切换档位  Enter 选入/移出模型  Esc 返回  — 档位说明: {}",
            pages::subagent::tier_hint(tier)
        ));
        frame.render_widget(
            Paragraph::new(hint).style(Style::default().fg(theme.on_surface_variant)),
            layout[1],
        );

        // ── Model list ─────────────────────────────────────────────────
        let items: Vec<ListItem> = if labels.is_empty() {
            vec![ListItem::new("  （无可用模型，请先配置供应商）")
                .style(Style::default().fg(theme.on_surface_variant))]
        } else {
            labels
                .iter()
                .map(|label| {
                    ListItem::new(Line::from(format!(" {label}")))
                        .style(Style::default().fg(theme.on_surface))
                })
                .collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(theme.primary_fg)
                    .bg(theme.primary_container_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▌ ");

        frame.render_stateful_widget(list, layout[2], &mut self.subagent.state);
    }

    fn draw_providers(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let inner = Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(3),
        };

        let selected = self.providers.state.selected().unwrap_or(0);
        let editing = self.providers.editing;
        let viewing = self.providers.viewing;

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(t("Providers and models", "供应商和模型"))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(theme.outline))
            .style(Style::default().bg(theme.surface_bg));

        let (rows, hint): (Vec<String>, Line) = if viewing || editing {
            // ── Field overview mode ─────────────────────────────────────
            let Some(provider) = self.config.providers.get(selected) else {
                return;
            };
            let rows = pages::providers::provider_field_rows(provider);
            let hint = if let Some(error) = &self.providers.error_msg {
                Line::from(format!(" ⚠ {error}"))
            } else if editing {
                Line::from(t(
                    " Type edit  Enter next  ↑↓ field  Esc back ",
                    " 输入修改  Enter 下一项  ↑↓ 切换字段  Esc 返回 ",
                ))
            } else {
                Line::from(format!(
                    " {} {} — {}",
                    t("Provider:", "供应商:"),
                    provider.id,
                    t("Enter to edit field, Esc back to list", "Enter 编辑字段  Esc 返回供应商列表")
                ))
            };
            (rows, hint)
        } else {
            // ── Provider list mode ─────────────────────────────────────
            let rows = pages::providers::provider_rows(
                &self.config.providers,
                &self.config.active_provider,
            );
            let hint = if self.providers.confirming_delete {
                Line::from(t(" Confirm delete? y / n ", " 确认删除? y 删除  n 取消 "))
            } else {
                Line::from(t(
                    " ↑↓ Navigate  Enter Edit  a Add  d Delete  s Set Active  Esc Back ",
                    " ↑↓ 导航  Enter 编辑  a 添加  d 删除  s 设为当前  Esc 返回 ",
                ))
            };
            (rows, hint)
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner);

        let hint_style = if self.providers.error_msg.is_some() {
            Style::default().fg(theme.error_fg).bg(theme.error_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.on_surface_variant)
        };
        frame.render_widget(Paragraph::new(hint).style(hint_style), layout[0]);

        let edit_row = self.providers.edit_field;
        let items: Vec<ListItem> = if rows.is_empty() {
            vec![ListItem::new(t("  (No providers, press a to add)", "  （无供应商，按 a 添加）"))
                .style(Style::default().fg(theme.on_surface_variant))]
        } else {
            rows.iter()
                .enumerate()
                .map(|(i, row)| {
                    if editing && i == edit_row {
                        // Inline edit: current row shows the edit buffer.
                        let label = row.split('=').next().unwrap_or(row).trim().to_string();
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!(" {label} = "),
                                Style::default().fg(theme.primary_fg).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                self.providers.edit_buffer.clone(),
                                Style::default().fg(theme.primary_fg).add_modifier(Modifier::UNDERLINED),
                            ),
                        ]))
                        .style(Style::default().bg(theme.primary_container_bg))
                    } else {
                        ListItem::new(Line::from(format!(" {row}")))
                            .style(Style::default().fg(theme.on_surface))
                    }
                })
                .collect()
        };

        // Field list: use persistent field_state so ratatui maintains scroll offset.
        pages::sync_field_state(&mut self.providers.field_state, self.providers.edit_field.min(rows.len().saturating_sub(1)));
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(theme.primary_fg)
                    .bg(theme.primary_container_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▌ ");

        frame.render_stateful_widget(list, layout[1], &mut self.providers.field_state);

        // Footer: position.
        let count = if viewing || editing {
            pages::providers::EDITABLE_FIELDS.len()
        } else {
            self.config.providers.len()
        };
        let pos_idx = if viewing || editing {
            self.providers.edit_field
        } else {
            self.providers.state.selected().unwrap_or(0)
        };
        frame.render_widget(
            Paragraph::new(Line::from(pages::position_label(pos_idx, count)))
                .style(Style::default().fg(theme.on_surface_variant)),
            layout[2],
        );
    }

    fn draw_global(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let inner = Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(3),
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(t("Global settings", "全局参数设置"))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(theme.outline))
            .style(Style::default().bg(theme.surface_bg));

        let rows = pages::global::global_rows(&self.config);
        let editing = self.global.editing;
        let edit_row = self.global.state.selected().unwrap_or(0);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner);

        let hint = if let Some(error) = &self.global.error_msg {
            Line::from(format!(" ⚠ {error}"))
        } else if editing {
            Line::from(t(
                " Type edit  Enter next  ←/→ choice  ↑↓ field  Esc back ",
                " 输入修改  Enter 下一项  ←/→ 切换选项  ↑↓ 切换字段  Esc 返回 ",
            ))
        } else {
            Line::from(t(" ↑↓ Navigate  Enter Edit  Esc Back ", " ↑↓ 导航  Enter 编辑  Esc 返回 "))
        };
        let hint_style = if self.global.error_msg.is_some() {
            Style::default().fg(theme.error_fg).bg(theme.error_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.on_surface_variant)
        };
        frame.render_widget(
            Paragraph::new(hint).style(hint_style),
            layout[0],
        );

        let items: Vec<ListItem> = rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                if editing && i == edit_row {
                    // Inline edit: current row shows the edit buffer.
                    let label = row.split('=').next().unwrap_or(row).trim().to_string();
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!(" {label} = "),
                            Style::default().fg(theme.primary_fg).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            self.global.edit_buffer.clone(),
                            Style::default().fg(theme.primary_fg).add_modifier(Modifier::UNDERLINED),
                        ),
                    ]))
                    .style(Style::default().bg(theme.primary_container_bg))
                } else {
                    ListItem::new(Line::from(format!(" {row}")))
                        .style(Style::default().fg(theme.on_surface))
                }
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(theme.primary_fg)
                    .bg(theme.primary_container_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▌ ");

        frame.render_stateful_widget(list, layout[1], &mut self.global.state);

        // Footer: position + current field type hint.
        let field_idx = self.global.state.selected().unwrap_or(0);
        let choices = pages::global::field_choices(field_idx);
        let help_text = if !choices.is_empty() {
            format!(
                " {} {}/{}  [{}]",
                t("Field", "字段"),
                field_idx + 1,
                pages::global::GLOBAL_FIELDS.len(),
                choices.join(" / ")
            )
        } else if pages::global::is_bool_field(field_idx) {
            format!(
                " {} {}/{}  (←/→ toggle)",
                t("Field", "字段"),
                field_idx + 1,
                pages::global::GLOBAL_FIELDS.len()
            )
        } else {
            format!(
                " {} {}/{}",
                t("Field", "字段"),
                field_idx + 1,
                pages::global::GLOBAL_FIELDS.len()
            )
        };
        frame.render_widget(
            Paragraph::new(Line::from(help_text)).style(Style::default().fg(theme.on_surface_variant)),
            layout[2],
        );
    }

    fn draw_plugins(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let inner = Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(3),
        };

        // ── API Quota mode ──────────────────────────────────────────────
        if self.plugins.quota_active {
            self.draw_quota(frame, inner, theme);
            return;
        }

        // ── Field overview + inline edit mode ────────────────────────────
        if self.plugins.viewing || self.plugins.editing {
            let selected = self.plugins.state.selected().unwrap_or(0);
            let plugin_id = pages::plugins::plugins().get(selected).map(|p| p.0).unwrap_or("?");
            let rows = self.plugins.field_rows(&self.config);
            let editing = self.plugins.editing;

            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(format!(" {plugin_id} "))
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(theme.outline))
                .style(Style::default().bg(theme.surface_bg));

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(3),
                    Constraint::Length(1),
                ])
                .split(inner);

            let hint = if let Some(error) = &self.plugins.error_msg {
                Line::from(format!(" ⚠ {error}"))
            } else if editing {
                Line::from(t(
                    " Type edit  Enter next  ←/→ choice  ↑↓ field  Esc back ",
                    " 输入修改  Enter 下一项  ←/→ 切换选项  ↑↓ 切换字段  Esc 返回 ",
                ))
            } else {
                Line::from(t(" Enter to edit field  Esc back to list ", " Enter 编辑字段  Esc 返回插件列表 "))
            };
            let hint_style = if self.plugins.error_msg.is_some() {
                Style::default().fg(theme.error_fg).bg(theme.error_bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.on_surface_variant)
            };
            frame.render_widget(Paragraph::new(hint).style(hint_style), layout[0]);

            let edit_row = self.plugins.edit_field;
            let items: Vec<ListItem> = rows
                .iter()
                .enumerate()
                .map(|(i, row)| {
                    if editing && i == edit_row {
                        let label = row.split('=').next().unwrap_or(row).trim().to_string();
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!(" {label} = "),
                                Style::default().fg(theme.primary_fg).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                self.plugins.edit_buffer.clone(),
                                Style::default().fg(theme.primary_fg).add_modifier(Modifier::UNDERLINED),
                            ),
                        ]))
                        .style(Style::default().bg(theme.primary_container_bg))
                    } else {
                        ListItem::new(Line::from(format!(" {row}")))
                            .style(Style::default().fg(theme.on_surface))
                    }
                })
                .collect();

            pages::sync_field_state(&mut self.plugins.field_state, self.plugins.edit_field.min(rows.len().saturating_sub(1)));
            let list = List::new(items)
                .block(block)
                .highlight_style(
                    Style::default()
                        .fg(theme.primary_fg)
                        .bg(theme.primary_container_bg)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(" ▌ ");
            frame.render_stateful_widget(list, layout[1], &mut self.plugins.field_state);

            frame.render_widget(
                Paragraph::new(Line::from(pages::position_label(self.plugins.edit_field, rows.len())))
                    .style(Style::default().fg(theme.on_surface_variant)),
                layout[2],
            );
            return;
        }

        // ── List mode ──────────────────────────────────────────────────
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(t("Plugins", "插件配置"))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(theme.outline))
            .style(Style::default().bg(theme.surface_bg));

        let rows = pages::plugins::plugin_rows(&self.config);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner);

        let hint = Line::from(t(" ↑↓ navigate  Space toggle  Enter detail  Esc back ", " ↑↓ 导航  空格 启用/禁用  Enter 详情  Esc 返回 "));
        frame.render_widget(
            Paragraph::new(hint).style(Style::default().fg(theme.on_surface_variant)),
            layout[0],
        );

        let items: Vec<ListItem> = rows
            .iter()
            .map(|row| {
                ListItem::new(Line::from(format!(" {row}")))
                    .style(Style::default().fg(theme.on_surface))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(theme.primary_fg)
                    .bg(theme.primary_container_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▌ ");

        frame.render_stateful_widget(list, layout[1], &mut self.plugins.state);
    }

    fn draw_quota(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let provider = pages::plugins::QUOTA_PROVIDERS[self.plugins.quota_provider];
        let accounts = pages::plugins::quota_accounts(&self.config, self.plugins.quota_provider);
        let account_idx = self.plugins.edit_field.min(accounts.len().saturating_sub(1));
        let editing = self.plugins.editing;

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(area);

        // Header
        let header = Line::from(vec![
            Span::styled(
                format!(" {provider} API Quota "),
                Style::default().fg(theme.primary_fg).bg(theme.primary_container_bg).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("  {} {}", t("accounts", "个账号"), accounts.len())),
        ]);
        frame.render_widget(Paragraph::new(header), layout[0]);

        let hint = if editing {
            let field_name = if self.plugins.quota_field_idx == 0 { "name" } else { "API key" };
            Line::from(format!(" {}: {}  Enter apply  Esc back ", t("Editing", "编辑"), field_name))
        } else {
            Line::from(t(
                " ↑↓ navigate  Enter edit  Tab switch provider  a add  d delete  Esc back ",
                " ↑↓ 导航  Enter 编辑  Tab 切换供应商  a 添加  d 删除  Esc 返回 ",
            ))
        };
        frame.render_widget(
            Paragraph::new(hint).style(Style::default().fg(theme.on_surface_variant)),
            layout[1],
        );

        // Account list
        let items: Vec<ListItem> = if accounts.is_empty() {
            vec![ListItem::new(t("  (no accounts, press a to add)", "  （无账号，按 a 添加）"))
                .style(Style::default().fg(theme.on_surface_variant))]
        } else {
            accounts.iter().enumerate().map(|(i, account)| {
                let mark = if i == account_idx && editing { "▶ " } else { "  " };
                let name_display = if editing && i == account_idx {
                    format!("{}{} = {}", mark, account.name, self.plugins.edit_buffer)
                } else {
                    let key_hint = if account.api_key.is_empty() { "" } else { " ***" };
                    format!("{mark}{}{key_hint}", account.name)
                };
                ListItem::new(Line::from(name_display))
                    .style(Style::default().fg(theme.on_surface))
            }).collect()
        };

        let list = List::new(items)
            .block(Block::bordered().border_type(BorderType::Rounded).title(format!(" {provider} ")).title_alignment(Alignment::Center)
                .border_style(Style::default().fg(theme.outline)).style(Style::default().bg(theme.surface_bg)))
            .highlight_style(Style::default().fg(theme.primary_fg).bg(theme.primary_container_bg).add_modifier(Modifier::BOLD))
            .highlight_symbol(" ▌ ");

        let mut state = self.plugins.state.clone();
        state.select(Some(account_idx));
        frame.render_stateful_widget(list, layout[2], &mut state);
        self.plugins.state = state;

        // Footer
        let footer = if editing {
            Line::from(t(" ↑↓ switch field (name / key)", " ↑↓ 切换字段 (名称 / 密钥)"))
        } else {
            Line::from(format!(" {} {}", t("Provider:", "供应商:"), provider))
        };
        frame.render_widget(
            Paragraph::new(footer).style(Style::default().fg(theme.on_surface_variant)),
            layout[3],
        );
    }

    fn draw_platforms(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let inner = Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(3),
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(t("Platforms", "接入平台"))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(theme.outline))
            .style(Style::default().bg(theme.surface_bg));

        let selected = self.platforms.state.selected().unwrap_or(0);
        let editing = self.platforms.editing;
        let viewing = self.platforms.viewing;

        let (rows, hint): (Vec<String>, Line) = if viewing || editing {
            // ── Field overview mode ─────────────────────────────────────
            let platform_name = pages::platforms::PLATFORMS.get(selected).map(|p| p.1).unwrap_or("?");
            let rows = pages::platforms::platform_field_rows(&self.config, selected);
            let hint = if let Some(error) = &self.platforms.error_msg {
                Line::from(format!(" ⚠ {error}"))
            } else if editing {
                Line::from(t(
                    " Type edit  Enter next  ←/→ choice  ↑↓ field  Esc back ",
                    " 输入修改  Enter 下一项  ←/→ 切换选项  ↑↓ 切换字段  Esc 返回 ",
                ))
            } else {
                Line::from(format!(" {} {platform_name} — {}", t("Enter to edit", "Enter 编辑字段"), t("Esc back to list", "Esc 返回平台列表")))
            };
            (rows, hint)
        } else {
            // ── Platform list mode ──────────────────────────────────────
            let rows = pages::platforms::platform_rows(&self.config);
            let hint = Line::from(t(" ↑↓ Navigate  Space Toggle  Enter Edit Fields  Esc Back ", " ↑↓ 导航  空格 启用/禁用  Enter 编辑字段  Esc 返回 "));
            (rows, hint)
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner);

        let hint_style = if self.platforms.error_msg.is_some() {
            Style::default().fg(theme.error_fg).bg(theme.error_bg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.on_surface_variant)
        };
        frame.render_widget(Paragraph::new(hint).style(hint_style), layout[0]);

        let edit_row = self.platforms.edit_field;
        let items: Vec<ListItem> = rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                if editing && i == edit_row {
                    // Inline edit: current row shows the edit buffer.
                    let label = row.split('=').next().unwrap_or(row).trim().to_string();
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!(" {label} = "),
                            Style::default().fg(theme.primary_fg).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            self.platforms.edit_buffer.clone(),
                            Style::default().fg(theme.primary_fg).add_modifier(Modifier::UNDERLINED),
                        ),
                    ]))
                    .style(Style::default().bg(theme.primary_container_bg))
                } else {
                    ListItem::new(Line::from(format!(" {row}")))
                        .style(Style::default().fg(theme.on_surface))
                }
            })
            .collect();

        // Field list: use persistent field_state so ratatui maintains scroll offset;
        // self.platforms.state keeps the entity (platform) selection unchanged.
        pages::sync_field_state(&mut self.platforms.field_state, self.platforms.edit_field.min(rows.len().saturating_sub(1)));
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(theme.primary_fg)
                    .bg(theme.primary_container_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▌ ");

        frame.render_stateful_widget(list, layout[1], &mut self.platforms.field_state);

        // Footer: position (field position in overview/edit, entity position in list).
        let count = if viewing || editing {
            pages::platforms::platform_field_rows(&self.config, selected).len()
        } else {
            pages::platforms::PLATFORMS.len()
        };
        let pos_idx = if viewing || editing {
            self.platforms.edit_field
        } else {
            self.platforms.state.selected().unwrap_or(0)
        };
        frame.render_widget(
            Paragraph::new(Line::from(pages::position_label(pos_idx, count)))
                .style(Style::default().fg(theme.on_surface_variant)),
            layout[2],
        );
    }

    fn draw_prompts(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let inner = Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(3),
        };

        // ── Creating / Renaming mode ────────────────────────────────────
        if self.prompts.creating || self.prompts.renaming {

            let layout = Layout::default().direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(1), Constraint::Length(3), Constraint::Length(1)])
                .split(inner);

            let label = Line::from(format!("{}: {}", t("Name", "名称"), self.prompts.edit_buffer));
            frame.render_widget(Paragraph::new(label), layout[0]);

            let hint = Line::from(t(" Enter to confirm  Esc to cancel ", " Enter 确认  Esc 取消 "));
            frame.render_widget(Paragraph::new(hint).style(Style::default().fg(theme.on_surface_variant)), layout[1]);

            let input_box = Paragraph::new(self.prompts.edit_buffer.clone())
                .block(Block::bordered().border_type(BorderType::Rounded))
                .style(Style::default().fg(theme.on_surface).bg(theme.surface_dim_bg));
            frame.render_widget(input_box, layout[2]);
            return;
        }

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(t("Custom prompts / Personas", "自定义提示词 / 人格"))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(theme.outline))
            .style(Style::default().bg(theme.surface_bg));

        let active = &self.config.prompt.active_persona;
        let rows = pages::prompts::persona_rows(self.paths, active);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner);

        let hint = if self.prompts.confirming_delete {
            Line::from(t(" Confirm delete? y / n ", " 确认删除? y 删除  n 取消 "))
        } else {
            Line::from(t(" ↑↓ Navigate  Enter Activate  n New  r Rename  d Delete  Esc Back ", " ↑↓ 导航  Enter 激活  n 新建  r 重命名  d 删除  Esc 返回 "))
        };
        frame.render_widget(
            Paragraph::new(hint).style(Style::default().fg(theme.on_surface_variant)),
            layout[0],
        );

        let items: Vec<ListItem> = if rows.is_empty() {
            vec![ListItem::new(t("  (No personas, press n to create)", "  （无可用人格，按 n 新建）"))
                .style(Style::default().fg(theme.on_surface_variant))]
        } else {
            rows.iter()
                .map(|(row, _, _)| {
                    ListItem::new(Line::from(format!(" {row}")))
                        .style(Style::default().fg(theme.on_surface))
                })
                .collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(theme.primary_fg)
                    .bg(theme.primary_container_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ▌ ");

        frame.render_stateful_widget(list, layout[1], &mut self.prompts.state);

        let footer_text = if active.is_empty() {
            format!("{} {}", t("Current: (none)", "当前: (未激活)"), pages::position_label(self.prompts.state.selected().unwrap_or(0), rows.len()))
        } else {
            format!("{}: {} | {}", t("Current", "当前"), active, pages::position_label(self.prompts.state.selected().unwrap_or(0), rows.len()))
        };
        let footer = Line::from(footer_text);
        frame.render_widget(
            Paragraph::new(footer).style(Style::default().fg(theme.on_surface_variant)),
            layout[2],
        );
    }

    // ── Input ──────────────────────────────────────────────────────────

    fn handle_input(&mut self) -> Result<Option<AppEvent>> {
        use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

        if !event::poll(std::time::Duration::from_millis(100))? {
            return Ok(None);
        }

        match event::read()? {
            Event::Key(KeyEvent {
                code, kind: KeyEventKind::Press, ..
            }) => match self.screen {
                Screen::MainMenu => match code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(Some(AppEvent::Quit));
                    }
                    KeyCode::Up | KeyCode::Char('k')
                    | KeyCode::Down | KeyCode::Char('j')
                    | KeyCode::PageUp | KeyCode::PageDown
                    | KeyCode::Home | KeyCode::End => {
                        let i = self.main_menu_state.selected().unwrap_or(0);
                        if let Some(next) = pages::nav_index(code, i, MAIN_MENU_ITEMS) {
                            self.main_menu_state.select(Some(next));
                        }
                    }
                    KeyCode::Enter => {
                        let i = self.main_menu_state.selected().unwrap_or(0);
                        return Ok(Some(AppEvent::SelectMainMenu(i)));
                    }
                    _ => {}
                },
                Screen::TextModel => return Ok(Some(self.handle_text_model_key(code))),
                Screen::MultimodalModel => return Ok(Some(self.handle_multimodal_key(code))),
                Screen::SubagentTiers => return Ok(Some(self.handle_subagent_key(code))),
                Screen::Providers => return Ok(Some(self.handle_providers_key(code))),
                Screen::GlobalSettings => return Ok(Some(self.handle_global_key(code))),
                Screen::Plugins => return Ok(Some(self.handle_plugins_key(code))),
                Screen::Platforms => return Ok(Some(self.handle_platforms_key(code))),
                Screen::Prompts => return Ok(Some(self.handle_prompts_key(code))),
            },
            _ => {}
        }
        Ok(None)
    }

    fn handle_text_model_key(&mut self, code: crossterm::event::KeyCode) -> AppEvent {
        use crossterm::event::KeyCode;
        if self.text_model.editing {
            return match code {
                KeyCode::Esc => {
                    self.text_model.editing = false;
                    self.text_model.edit_buffer.clear();
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let parsed =
                        pages::text_model::parse_model_input(&self.text_model.edit_buffer);
                    self.text_model.editing = false;
                    self.text_model.edit_buffer.clear();
                    if let Some(model) = parsed {
                        self.config.active_provider_models = Some(vec![model]);
                        self.dirty = true;
                    }
                    AppEvent::None
                }
                KeyCode::Backspace => {
                    self.text_model.edit_buffer.pop();
                    AppEvent::None
                }
                KeyCode::Char(c) => {
                    self.text_model.edit_buffer.push(c);
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        match code {
            KeyCode::Esc | KeyCode::Char('q') => AppEvent::Back,
            KeyCode::Up | KeyCode::Char('k') => {
                let choices = self.config.active_provider_model_choices();
                let i = self.text_model.state.selected().unwrap_or(0);
                self.text_model
                    .state
                    .select(Some(i.saturating_sub(1)));
                if !choices.is_empty() {
                    self.text_model.state.select(Some(
                        self.text_model.state.selected().unwrap_or(0).min(choices.len() - 1),
                    ));
                }
                AppEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let choices = self.config.active_provider_model_choices();
                let i = self.text_model.state.selected().unwrap_or(0);
                let next = (i + 1).min(choices.len().saturating_sub(1));
                self.text_model.state.select(Some(next));
                AppEvent::None
            }
            KeyCode::Enter => {
                let choices = self.config.active_provider_model_choices();
                if let Some(i) = self.text_model.state.selected() {
                    if let Some(choice) = choices.get(i) {
                        self.config.active_provider_models = Some(vec![
                            ActiveProviderModelConfig {
                                provider_id: choice.provider_id.clone(),
                                model: choice.model.clone(),
                            },
                        ]);
                        self.dirty = true;
                    }
                }
                AppEvent::None
            }
            KeyCode::Char('n') => {
                self.text_model.editing = true;
                self.text_model.edit_buffer.clear();
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }

    fn handle_multimodal_key(&mut self, code: crossterm::event::KeyCode) -> AppEvent {
        use crossterm::event::KeyCode;

        if self.multimodal.editing {
            return match code {
                KeyCode::Esc => {
                    self.multimodal.editing = false;
                    self.multimodal.edit_buffer.clear();
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let parsed =
                        pages::multimodal::parse_multimodal_input(&self.multimodal.edit_buffer);
                    self.multimodal.editing = false;
                    self.multimodal.edit_buffer.clear();
                    if let Some(model) = parsed {
                        self.config.active_multimodal_provider_models = Some(vec![model]);
                        self.dirty = true;
                    }
                    AppEvent::None
                }
                KeyCode::Backspace => {
                    self.multimodal.edit_buffer.pop();
                    AppEvent::None
                }
                KeyCode::Char(c) => {
                    self.multimodal.edit_buffer.push(c);
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        match code {
            KeyCode::Esc | KeyCode::Char('q') => AppEvent::Back,
            KeyCode::Up | KeyCode::Char('k') => {
                let choices = self.config.active_multimodal_provider_model_choices();
                let i = self.multimodal.state.selected().unwrap_or(0);
                self.multimodal
                    .state
                    .select(Some(i.saturating_sub(1)));
                if !choices.is_empty() {
                    self.multimodal.state.select(Some(
                        self.multimodal.state.selected().unwrap_or(0).min(choices.len() - 1),
                    ));
                }
                AppEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let choices = self.config.active_multimodal_provider_model_choices();
                let i = self.multimodal.state.selected().unwrap_or(0);
                let next = (i + 1).min(choices.len().saturating_sub(1));
                self.multimodal.state.select(Some(next));
                AppEvent::None
            }
            KeyCode::Enter => {
                let choices = self.config.active_multimodal_provider_model_choices();
                if let Some(i) = self.multimodal.state.selected() {
                    if let Some(choice) = choices.get(i) {
                        self.config.active_multimodal_provider_models = Some(vec![
                            ActiveProviderModelConfig {
                                provider_id: choice.provider_id.clone(),
                                model: choice.model.clone(),
                            },
                        ]);
                        self.dirty = true;
                    }
                }
                AppEvent::None
            }
            KeyCode::Char('n') => {
                self.multimodal.editing = true;
                self.multimodal.edit_buffer.clear();
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }

    fn handle_subagent_key(&mut self, code: crossterm::event::KeyCode) -> AppEvent {
        use crossterm::event::KeyCode;

        match code {
            KeyCode::Esc | KeyCode::Char('q') => AppEvent::Back,
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                let next = (self.subagent.tab_index + 1).min(2);
                self.subagent.tab_index = next;
                self.subagent.state.select(Some(0));
                AppEvent::None
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                let prev = self.subagent.tab_index.saturating_sub(1);
                self.subagent.tab_index = prev;
                self.subagent.state.select(Some(0));
                AppEvent::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.subagent.state.selected().unwrap_or(0);
                self.subagent.state.select(Some(i.saturating_sub(1)));
                AppEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.subagent.state.selected().unwrap_or(0);
                let choices = self.config.provider_model_choices();
                let next = (i + 1).min(choices.len().saturating_sub(1));
                self.subagent.state.select(Some(next));
                AppEvent::None
            }
            KeyCode::Enter => {
                let tier = self.subagent.active_tier();
                let choices = self.config.provider_model_choices();
                if let Some(i) = self.subagent.state.selected() {
                    if let Some(choice) = choices.get(i) {
                        let result = self.config.toggle_subagent_tier_model(
                            tier,
                            &choice.provider_id,
                            &choice.model,
                        );
                        if result.is_ok() {
                            self.dirty = true;
                        }
                    }
                }
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }

    fn handle_providers_key(&mut self, code: crossterm::event::KeyCode) -> AppEvent {
        use crossterm::event::KeyCode;

        // ── Inline edit mode ───────────────────────────────────────────
        if self.providers.editing {
            return match code {
                KeyCode::Esc => {
                    self.providers.editing = false;
                    self.providers.edit_buffer.clear();
                    self.providers.error_msg = None;
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let selected = self.providers.state.selected().unwrap_or(0);
                    let field = self.providers.edit_field;
                    let value = self.providers.edit_buffer.clone();
                    if let Some(provider) = self.config.providers.get_mut(selected) {
                        let changed = pages::providers::apply_field(provider, field, &value);
                        if changed {
                            self.dirty = true;
                            self.providers.error_msg = None;
                        } else {
                            self.providers.error_msg = Some(t("Invalid value for this field", "该字段的值无效"));
                        }
                    }
                    // Continuous edit: jump to next field.
                    self.providers.edit_field = (field + 1)
                        .min(pages::providers::EDITABLE_FIELDS.len() - 1);
                    self.reload_provider_edit_buffer();
                    AppEvent::None
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('n') => {
                    if let Some(next) = pages::move_field_index(
                        code,
                        self.providers.edit_field,
                        pages::providers::EDITABLE_FIELDS.len(),
                    ) {
                        self.providers.edit_field = next;
                        pages::sync_field_state(&mut self.providers.field_state, next);
                        self.reload_provider_edit_buffer();
                    }
                    AppEvent::None
                }
                KeyCode::Backspace => {
                    self.providers.edit_buffer.pop();
                    AppEvent::None
                }
                KeyCode::Char(c) => {
                    self.providers.edit_buffer.push(c);
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        // ── Field overview mode ────────────────────────────────────────
        if self.providers.viewing {
            return match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.providers.viewing = false;
                    AppEvent::None
                }
                KeyCode::Enter => {
                    self.reload_provider_edit_buffer();
                    self.providers.editing = true;
                    AppEvent::None
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j')
                | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                    if let Some(next) = pages::move_field_index(
                        code,
                        self.providers.edit_field,
                        pages::providers::EDITABLE_FIELDS.len(),
                    ) {
                        self.providers.edit_field = next;
                        pages::sync_field_state(&mut self.providers.field_state, next);
                    }
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        // ── Confirm delete mode ─────────────────────────────────────────
        if self.providers.confirming_delete {
            return match code {
                KeyCode::Char('y') => {
                    let selected = self.providers.state.selected().unwrap_or(0);
                    let removed_id = self
                        .config
                        .providers
                        .get(selected)
                        .map(|p| p.id.clone());
                    if let Some(id) = removed_id {
                        if self.config.active_provider == id {
                            self.config.active_provider.clear();
                        }
                        self.config.providers.remove(selected);
                        self.dirty = true;
                        let i = self.providers.state.selected().unwrap_or(0);
                        self.providers
                            .state
                            .select(Some(i.saturating_sub(1)));
                    }
                    self.providers.confirming_delete = false;
                    AppEvent::None
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.providers.confirming_delete = false;
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        // ── Provider list mode ─────────────────────────────────────────
        match code {
            KeyCode::Esc | KeyCode::Char('q') => AppEvent::Back,
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.providers.state.selected().unwrap_or(0);
                self.providers.state.select(Some(i.saturating_sub(1)));
                AppEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.providers.state.selected().unwrap_or(0);
                let next = (i + 1).min(self.config.providers.len().saturating_sub(1));
                self.providers.state.select(Some(next));
                AppEvent::None
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                let i = self.providers.state.selected().unwrap_or(0);
                if let Some(next) = pages::nav_index(code, i, self.config.providers.len()) {
                    self.providers.state.select(Some(next));
                }
                AppEvent::None
            }
            KeyCode::Enter => {
                // Enter: enter field overview of the selected provider.
                self.providers.viewing = true;
                self.providers.edit_field = 0;
                pages::sync_field_state(&mut self.providers.field_state, 0);
                self.providers.error_msg = None;
                AppEvent::None
            }
            KeyCode::Char('a') => {
                self.add_provider_from_template();
                AppEvent::None
            }
            KeyCode::Char('d') => {
                if !self.config.providers.is_empty() {
                    self.providers.confirming_delete = true;
                }
                AppEvent::None
            }
            KeyCode::Char('s') => {
                if let Some(i) = self.providers.state.selected() {
                    if let Some(provider) = self.config.providers.get(i) {
                        self.config.active_provider = provider.id.clone();
                        self.dirty = true;
                    }
                }
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }

    /// 将选中供应商的当前字段值载入编辑缓冲。
    fn reload_provider_edit_buffer(&mut self) {
        let selected = self.providers.state.selected().unwrap_or(0);
        self.providers.edit_buffer.clear();
        if let Some(provider) = self.config.providers.get(selected) {
            self.providers.edit_buffer =
                pages::providers::field_value(provider, self.providers.edit_field);
        }
    }

    /// 从模板列表添加下一个未配置的供应商。
    fn add_provider_from_template(&mut self) {
        let templates = crate::config::ProviderConfig::default_templates();
        let existing: Vec<&str> = self
            .config
            .providers
            .iter()
            .map(|p| p.id.as_str())
            .collect();
        let Some(template) = templates
            .into_iter()
            .find(|template| !existing.contains(&template.id.as_str()))
        else {
            return;
        };
        let new_id = template.id.clone();
        self.config.providers.push(template);
        self.config.providers.sort_by(|a, b| a.id.cmp(&b.id));
        self.dirty = true;
        if let Some(i) = self
            .config
            .providers
            .iter()
            .position(|p| p.id == new_id)
        {
            self.providers.state.select(Some(i));
        }
    }

    fn handle_global_key(&mut self, code: crossterm::event::KeyCode) -> AppEvent {
        use crossterm::event::KeyCode;

        if self.global.editing {
            return match code {
                KeyCode::Esc => {
                    self.global.editing = false;
                    self.global.edit_buffer.clear();
                    self.global.error_msg = None;
                    AppEvent::None
                }
                KeyCode::Right | KeyCode::Left => {
                    let field = self.global.state.selected().unwrap_or(0);
                    let choices = pages::global::field_choices(field);
                    let direction = if code == KeyCode::Right { 1 } else { -1 };
                    let value = if !choices.is_empty() {
                        // Choices: cycle forward/backward.
                        let current_index = choices
                            .iter()
                            .position(|c| c == &self.global.edit_buffer)
                            .unwrap_or(0);
                        let len = choices.len() as isize;
                        let next = (current_index as isize + direction + len) % len;
                        choices[next as usize].to_string()
                    } else if pages::global::is_bool_field(field) {
                        // Bool: flip.
                        (!self.global.edit_buffer.parse::<bool>().unwrap_or(false)).to_string()
                    } else {
                        return AppEvent::None;
                    };
                    if pages::global::apply_field(&mut self.config, field, &value) {
                        self.dirty = true;
                        self.global.edit_buffer = value;
                        self.global.error_msg = None;
                    }
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let field = self.global.state.selected().unwrap_or(0);
                    let is_bool_or_choices = pages::global::is_bool_field(field)
                        || !pages::global::field_choices(field).is_empty();
                    if is_bool_or_choices {
                        // Value already applied via ←/→; just move to next field.
                        let next = (field + 1).min(pages::global::GLOBAL_FIELDS.len() - 1);
                        self.global.state.select(Some(next));
                        self.global.edit_buffer = pages::global::field_value(&self.config, next);
                        return AppEvent::None;
                    }
                    let value = self.global.edit_buffer.clone();
                    if pages::global::apply_field(&mut self.config, field, &value) {
                        self.dirty = true;
                        self.global.error_msg = None;
                        // Continuous edit: jump to next field.
                        let next = (field + 1).min(pages::global::GLOBAL_FIELDS.len() - 1);
                        self.global.state.select(Some(next));
                        self.global.edit_buffer = pages::global::field_value(&self.config, next);
                    } else {
                        self.global.error_msg = Some(t("Invalid value for this field", "该字段的值无效"));
                    }
                    AppEvent::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let field = self.global.state.selected().unwrap_or(0);
                    let prev = field.saturating_sub(1);
                    self.global.state.select(Some(prev));
                    self.global.edit_buffer = pages::global::field_value(&self.config, prev);
                    self.global.error_msg = None;
                    AppEvent::None
                }
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('n') => {
                    let field = self.global.state.selected().unwrap_or(0);
                    let next = (field + 1).min(pages::global::GLOBAL_FIELDS.len() - 1);
                    self.global.state.select(Some(next));
                    self.global.edit_buffer = pages::global::field_value(&self.config, next);
                    self.global.error_msg = None;
                    AppEvent::None
                }
                KeyCode::Backspace => {
                    self.global.edit_buffer.pop();
                    AppEvent::None
                }
                KeyCode::Char(c) => {
                    self.global.edit_buffer.push(c);
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        match code {
            KeyCode::Esc | KeyCode::Char('q') => AppEvent::Back,
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.global.state.selected().unwrap_or(0);
                self.global.state.select(Some(i.saturating_sub(1)));
                AppEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.global.state.selected().unwrap_or(0);
                let next = (i + 1).min(pages::global::GLOBAL_FIELDS.len() - 1);
                self.global.state.select(Some(next));
                AppEvent::None
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                let i = self.global.state.selected().unwrap_or(0);
                if let Some(next) = pages::nav_index(code, i, pages::global::GLOBAL_FIELDS.len()) {
                    self.global.state.select(Some(next));
                }
                AppEvent::None
            }
            KeyCode::Enter => {
                let field = self.global.state.selected().unwrap_or(0);
                self.global.edit_buffer =
                    pages::global::field_value(&self.config, field);
                self.global.editing = true;
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }

    fn handle_plugins_key(&mut self, code: crossterm::event::KeyCode) -> AppEvent {
        use crossterm::event::KeyCode;

        // ── API Quota mode ──────────────────────────────────────────────
        if self.plugins.quota_active {
            return self.handle_quota_key(code);
        }

        // ── Field overview mode ────────────────────────────────────────
        if self.plugins.viewing {
            let field_count = self.plugins.current_fields(&self.config).len();
            return match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.plugins.viewing = false;
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let fields = self.plugins.current_fields(&self.config);
                    self.plugins.edit_buffer = fields.get(self.plugins.edit_field).map(|f| f.value.clone()).unwrap_or_default();
                    self.plugins.editing = true;
                    self.plugins.error_msg = None;
                    AppEvent::None
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j')
                | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                    if let Some(next) = pages::move_field_index(code, self.plugins.edit_field, field_count) {
                        self.plugins.edit_field = next;
                        pages::sync_field_state(&mut self.plugins.field_state, next);
                    }
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        // ── Inline edit mode ───────────────────────────────────────────
        if self.plugins.editing {
            let field_count = self.plugins.current_fields(&self.config).len();
            let field_idx = self.plugins.edit_field.min(field_count.saturating_sub(1));
            return match code {
                KeyCode::Esc => {
                    self.plugins.editing = false;
                    self.plugins.edit_buffer.clear();
                    self.plugins.error_msg = None;
                    AppEvent::None
                }
                KeyCode::Right | KeyCode::Left => {
                    let fields = self.plugins.current_fields(&self.config);
                    if field_idx >= fields.len() { return AppEvent::None; }
                    let current = &fields[field_idx];
                    let direction = if code == KeyCode::Right { 1 } else { -1 };
                    let mut updated = fields.clone();
                    if !current.choices.is_empty() {
                        let current_index = current.choices.iter().position(|x| x == &current.value).unwrap_or(0);
                        let len = current.choices.len() as isize;
                        let next = (current_index as isize + direction + len) % len;
                        updated[field_idx].value = current.choices[next as usize].clone();
                    } else if current.is_bool {
                        updated[field_idx].value = (!pages::plugins::parse_bool_field(&current.value).unwrap_or(false)).to_string();
                    } else {
                        return AppEvent::None;
                    }
                    if pages::plugins::apply_plugin_fields(
                        &mut self.config,
                        self.plugins.state.selected().unwrap_or(0),
                        &updated,
                    ).is_ok() {
                        self.dirty = true;
                        self.plugins.edit_buffer = updated[field_idx].value.clone();
                        self.plugins.error_msg = None;
                    }
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let fields = self.plugins.current_fields(&self.config);
                    if field_idx >= fields.len() { return AppEvent::None; }
                    let current = &fields[field_idx];
                    let mut updated = fields.clone();
                    if current.is_bool || !current.choices.is_empty() {
                        // Value already applied via ←/→; just move to next field.
                        let next = (field_idx + 1).min(field_count.saturating_sub(1));
                        self.plugins.edit_field = next;
                        self.plugins.edit_buffer = self.plugins.current_fields(&self.config)
                            .get(next).map(|f| f.value.clone()).unwrap_or_default();
                        return AppEvent::None;
                    }
                    updated[field_idx].value = self.plugins.edit_buffer.clone();
                    if pages::plugins::apply_plugin_fields(
                        &mut self.config,
                        self.plugins.state.selected().unwrap_or(0),
                        &updated,
                    ).is_ok() {
                        self.dirty = true;
                        self.plugins.error_msg = None;
                        let next = (field_idx + 1).min(field_count.saturating_sub(1));
                        self.plugins.edit_field = next;
                        self.plugins.edit_buffer = self.plugins.current_fields(&self.config)
                            .get(next).map(|f| f.value.clone()).unwrap_or_default();
                    } else {
                        self.plugins.error_msg = Some(t("Invalid value for this field", "该字段的值无效"));
                    }
                    AppEvent::None
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('n') => {
                    if let Some(next) = pages::move_field_index(code, self.plugins.edit_field, field_count) {
                        self.plugins.edit_field = next;
                        pages::sync_field_state(&mut self.plugins.field_state, next);
                        self.plugins.edit_buffer = self.plugins.current_fields(&self.config)
                            .get(next).map(|f| f.value.clone()).unwrap_or_default();
                    }
                    AppEvent::None
                }
                KeyCode::Backspace => {
                    self.plugins.edit_buffer.pop();
                    AppEvent::None
                }
                KeyCode::Char(c) => {
                    self.plugins.edit_buffer.push(c);
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        // ── List mode ──────────────────────────────────────────────────
        match code {
            KeyCode::Esc | KeyCode::Char('q') => AppEvent::Back,
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.plugins.state.selected().unwrap_or(0);
                self.plugins.state.select(Some(i.saturating_sub(1)));
                AppEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.plugins.state.selected().unwrap_or(0);
                let next = (i + 1).min(pages::plugins::plugins().len() - 1);
                self.plugins.state.select(Some(next));
                AppEvent::None
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                let i = self.plugins.state.selected().unwrap_or(0);
                if let Some(next) = pages::nav_index(code, i, pages::plugins::plugins().len()) {
                    self.plugins.state.select(Some(next));
                }
                AppEvent::None
            }
            KeyCode::Char(' ') => {
                let i = self.plugins.state.selected().unwrap_or(0);
                pages::plugins::toggle_plugin(&mut self.config, i);
                self.dirty = true;
                AppEvent::None
            }
            KeyCode::Enter => {
                let i = self.plugins.state.selected().unwrap_or(0);
                if i == 13 {
                    // api_quota: enter quota management mode
                    self.plugins.quota_active = true;
                    self.plugins.edit_field = 0;
                    self.plugins.edit_buffer.clear();
                    self.plugins.editing = false;
                    return AppEvent::None;
                }
                // Enter field overview of the selected plugin.
                self.plugins.viewing = true;
                self.plugins.edit_field = 0;
                pages::sync_field_state(&mut self.plugins.field_state, 0);
                self.plugins.edit_buffer.clear();
                self.plugins.error_msg = None;
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }

    fn handle_quota_key(&mut self, code: crossterm::event::KeyCode) -> AppEvent {
        use crossterm::event::KeyCode;

        if self.plugins.editing {
            return match code {
                KeyCode::Esc => {
                    self.plugins.editing = false;
                    self.plugins.edit_buffer.clear();
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let value = self.plugins.edit_buffer.clone();
                    let idx = self.plugins.edit_field;
                    pages::plugins::quota_set_account_field(
                        &mut self.config,
                        self.plugins.quota_provider,
                        idx,
                        self.plugins.quota_field_idx,
                        &value,
                    );
                    self.dirty = true;
                    self.plugins.editing = false;
                    self.plugins.edit_buffer.clear();
                    AppEvent::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.plugins.quota_field_idx = self.plugins.quota_field_idx.saturating_sub(1);
                    AppEvent::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.plugins.quota_field_idx = (self.plugins.quota_field_idx + 1).min(1);
                    AppEvent::None
                }
                KeyCode::Backspace => { self.plugins.edit_buffer.pop(); AppEvent::None }
                KeyCode::Char(c) => { self.plugins.edit_buffer.push(c); AppEvent::None }
                _ => AppEvent::None,
            };
        }

        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.plugins.quota_active = false;
                AppEvent::None
            }
            KeyCode::Tab => {
                self.plugins.quota_provider = (self.plugins.quota_provider + 1) % 2;
                self.plugins.edit_field = 0;
                AppEvent::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.plugins.edit_field = self.plugins.edit_field.saturating_sub(1);
                AppEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = pages::plugins::quota_accounts(&self.config, self.plugins.quota_provider).len();
                let next = (self.plugins.edit_field + 1).min(count.saturating_sub(1));
                self.plugins.edit_field = next;
                AppEvent::None
            }
            KeyCode::Enter => {
                let idx = self.plugins.edit_field;
                let count = pages::plugins::quota_accounts(&self.config, self.plugins.quota_provider).len();
                if idx < count {
                    let field_val = pages::plugins::quota_account_field(
                        &self.config, self.plugins.quota_provider, idx, self.plugins.quota_field_idx,
                    );
                    self.plugins.edit_buffer = field_val;
                    self.plugins.editing = true;
                }
                AppEvent::None
            }
            KeyCode::Char('a') => {
                pages::plugins::quota_add_account(&mut self.config, self.plugins.quota_provider);
                self.dirty = true;
                AppEvent::None
            }
            KeyCode::Char('d') => {
                let idx = self.plugins.edit_field;
                if pages::plugins::quota_delete_account(&mut self.config, self.plugins.quota_provider, idx).is_some() {
                    self.dirty = true;
                }
                let count = pages::plugins::quota_accounts(&self.config, self.plugins.quota_provider).len();
                if self.plugins.edit_field >= count.saturating_sub(1) {
                    self.plugins.edit_field = count.saturating_sub(2);
                }
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }

    fn handle_platforms_key(&mut self, code: crossterm::event::KeyCode) -> AppEvent {
        use crossterm::event::KeyCode;
        let selected = self.platforms.state.selected().unwrap_or(0);

        // ── Inline edit mode ───────────────────────────────────────────
        if self.platforms.editing {
            let field_count = pages::platforms::platform_fields(&self.config, selected).len();
            return match code {
                KeyCode::Esc => {
                    self.platforms.editing = false;
                    self.platforms.edit_buffer.clear();
                    self.platforms.error_msg = None;
                    AppEvent::None
                }
                KeyCode::Right | KeyCode::Left
                    if pages::platforms::platform_field_is_bool(&self.config, selected, self.platforms.edit_field) =>
                {
                    let field = self.platforms.edit_field;
                    let current = self.platforms.edit_buffer.parse::<bool>().unwrap_or(false);
                    let new_val = (!current).to_string();
                    if pages::platforms::apply_platform_field(&mut self.config, selected, field, &new_val) {
                        self.dirty = true;
                        self.platforms.edit_buffer =
                            pages::platforms::platform_field_value(&self.config, selected, field);
                        self.platforms.error_msg = None;
                    }
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let field = self.platforms.edit_field.min(field_count.saturating_sub(1));
                    let is_bool = pages::platforms::platform_field_is_bool(&self.config, selected, field);
                    if is_bool {
                        // Value already applied via ←/→; just move to next field.
                        let next = (field + 1).min(field_count.saturating_sub(1));
                        self.platforms.edit_field = next;
                        self.platforms.edit_buffer =
                            pages::platforms::platform_field_value(&self.config, selected, next);
                        return AppEvent::None;
                    }
                    let value = self.platforms.edit_buffer.clone();
                    if pages::platforms::apply_platform_field(&mut self.config, selected, field, &value) {
                        self.dirty = true;
                        self.platforms.error_msg = None;
                        // Continuous edit: jump to next field.
                        let next = (field + 1).min(field_count.saturating_sub(1));
                        self.platforms.edit_field = next;
                        self.platforms.edit_buffer =
                            pages::platforms::platform_field_value(&self.config, selected, next);
                    } else {
                        self.platforms.error_msg = Some(t("Invalid value for this field", "该字段的值无效"));
                    }
                    AppEvent::None
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('n') => {
                    if let Some(next) = pages::move_field_index(code, self.platforms.edit_field, field_count) {
                        self.platforms.edit_field = next;
                        pages::sync_field_state(&mut self.platforms.field_state, next);
                        self.platforms.edit_buffer =
                            pages::platforms::platform_field_value(&self.config, selected, next);
                        self.platforms.error_msg = None;
                    }
                    AppEvent::None
                }
                KeyCode::Backspace => {
                    self.platforms.edit_buffer.pop();
                    AppEvent::None
                }
                KeyCode::Char(c) => {
                    self.platforms.edit_buffer.push(c);
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        // ── Field overview mode ────────────────────────────────────────
        if self.platforms.viewing {
            let field_count = pages::platforms::platform_field_rows(&self.config, selected).len();
            return match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.platforms.viewing = false;
                    AppEvent::None
                }
                KeyCode::Enter => {
                    self.platforms.edit_buffer = pages::platforms::platform_field_value(
                        &self.config, selected, self.platforms.edit_field,
                    );
                    self.platforms.editing = true;
                    AppEvent::None
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j')
                | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                    if let Some(next) = pages::move_field_index(code, self.platforms.edit_field, field_count) {
                        self.platforms.edit_field = next;
                        pages::sync_field_state(&mut self.platforms.field_state, next);
                    }
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        // ── Platform list mode ─────────────────────────────────────────
        match code {
            KeyCode::Esc | KeyCode::Char('q') => AppEvent::Back,
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.platforms.state.selected().unwrap_or(0);
                self.platforms.state.select(Some(i.saturating_sub(1)));
                AppEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.platforms.state.selected().unwrap_or(0);
                let next = (i + 1).min(pages::platforms::PLATFORMS.len() - 1);
                self.platforms.state.select(Some(next));
                AppEvent::None
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                let i = self.platforms.state.selected().unwrap_or(0);
                if let Some(next) = pages::nav_index(code, i, pages::platforms::PLATFORMS.len()) {
                    self.platforms.state.select(Some(next));
                }
                AppEvent::None
            }
            KeyCode::Char(' ') => {
                let i = self.platforms.state.selected().unwrap_or(0);
                pages::platforms::toggle_platform(&mut self.config, i);
                self.dirty = true;
                AppEvent::None
            }
            KeyCode::Enter => {
                self.platforms.viewing = true;
                self.platforms.edit_field = 0;
                pages::sync_field_state(&mut self.platforms.field_state, 0);
                self.platforms.edit_buffer.clear();
                self.platforms.error_msg = None;
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }

    fn handle_prompts_key(&mut self, code: crossterm::event::KeyCode) -> AppEvent {
        use crossterm::event::KeyCode;

        // ── Creating / Renaming mode ────────────────────────────────────
        if self.prompts.creating {
            return match code {
                KeyCode::Esc => {
                    self.prompts.creating = false;
                    self.prompts.edit_buffer.clear();
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let name = self.prompts.edit_buffer.trim().to_string();
                    self.prompts.creating = false;
                    self.prompts.edit_buffer.clear();
                    if !name.is_empty() {
                        if let Err(e) = pages::prompts::create_persona(self.paths, &name) {
                            // Silently fail — persona may already exist
                            let _ = e;
                        } else {
                            self.config.prompt.active_persona = name;
                            self.dirty = true;
                        }
                    }
                    AppEvent::None
                }
                KeyCode::Backspace => { self.prompts.edit_buffer.pop(); AppEvent::None }
                KeyCode::Char(c) => { self.prompts.edit_buffer.push(c); AppEvent::None }
                _ => AppEvent::None,
            };
        }

        if self.prompts.renaming {
            return match code {
                KeyCode::Esc => {
                    self.prompts.renaming = false;
                    self.prompts.edit_buffer.clear();
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let new_name = self.prompts.edit_buffer.trim().to_string();
                    self.prompts.renaming = false;
                    let rows = pages::prompts::persona_rows(self.paths, &self.config.prompt.active_persona);
                    let old_name = self.prompts.state.selected()
                        .and_then(|i| rows.get(i).map(|r| r.1.clone()))
                        .unwrap_or_default();
                    self.prompts.edit_buffer.clear();
                    if !new_name.is_empty() && !old_name.is_empty() {
                        if pages::prompts::rename_persona(self.paths, &old_name, &new_name).is_ok() {
                            if self.config.prompt.active_persona == old_name {
                                self.config.prompt.active_persona = new_name;
                            }
                            self.dirty = true;
                        }
                    }
                    AppEvent::None
                }
                KeyCode::Backspace => { self.prompts.edit_buffer.pop(); AppEvent::None }
                KeyCode::Char(c) => { self.prompts.edit_buffer.push(c); AppEvent::None }
                _ => AppEvent::None,
            };
        }

        // ── Confirm delete mode ─────────────────────────────────────────
        if self.prompts.confirming_delete {
            return match code {
                KeyCode::Char('y') => {
                    let rows = pages::prompts::persona_rows(self.paths, &self.config.prompt.active_persona);
                    let name = self.prompts.state.selected()
                        .and_then(|i| rows.get(i).map(|r| r.1.clone()))
                        .unwrap_or_default();
                    if !name.is_empty() {
                        let _ = pages::prompts::delete_persona(self.paths, &name);
                        if self.config.prompt.active_persona == name {
                            self.config.prompt.active_persona.clear();
                        }
                        self.dirty = true;
                    }
                    self.prompts.confirming_delete = false;
                    AppEvent::None
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.prompts.confirming_delete = false;
                    AppEvent::None
                }
                _ => AppEvent::None,
            };
        }

        // ── List mode ──────────────────────────────────────────────────
        match code {
            KeyCode::Esc | KeyCode::Char('q') => AppEvent::Back,
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.prompts.state.selected().unwrap_or(0);
                self.prompts.state.select(Some(i.saturating_sub(1)));
                AppEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let rows =
                    pages::prompts::persona_rows(self.paths, &self.config.prompt.active_persona);
                let i = self.prompts.state.selected().unwrap_or(0);
                let next = (i + 1).min(rows.len().saturating_sub(1));
                self.prompts.state.select(Some(next));
                AppEvent::None
            }
            KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End => {
                let rows =
                    pages::prompts::persona_rows(self.paths, &self.config.prompt.active_persona);
                let i = self.prompts.state.selected().unwrap_or(0);
                if let Some(next) = pages::nav_index(code, i, rows.len()) {
                    self.prompts.state.select(Some(next));
                }
                AppEvent::None
            }
            KeyCode::Enter => {
                let rows =
                    pages::prompts::persona_rows(self.paths, &self.config.prompt.active_persona);
                if let Some(i) = self.prompts.state.selected() {
                    if let Some((_, name, _)) = rows.get(i) {
                        self.config.prompt.active_persona = name.clone();
                        self.dirty = true;
                    }
                }
                AppEvent::None
            }
            KeyCode::Char('n') => {
                self.prompts.creating = true;
                self.prompts.edit_buffer.clear();
                AppEvent::None
            }
            KeyCode::Char('r') => {
                let rows = pages::prompts::persona_rows(self.paths, &self.config.prompt.active_persona);
                let name = self.prompts.state.selected()
                    .and_then(|i| rows.get(i).map(|r| r.1.clone()))
                    .unwrap_or_default();
                self.prompts.renaming = true;
                self.prompts.edit_buffer = name;
                AppEvent::None
            }
            KeyCode::Char('d') => {
                self.prompts.confirming_delete = true;
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }
}

const MAIN_MENU_ITEMS: usize = 9;

enum AppEvent {
    SelectMainMenu(usize),
    Back,
    Quit,
    None,
}

// ── Helpers ────────────────────────────────────────────────────────────

fn screen_title(screen: &Screen) -> String {
    match screen {
        Screen::MainMenu => t("Main menu", "主菜单"),
        Screen::TextModel => t("Text model", "文本模型"),
        Screen::MultimodalModel => t("Multimodal model", "多模态模型"),
        Screen::SubagentTiers => t("Subagent tiers", "子代理档位"),
        Screen::Providers => t("Providers", "供应商"),
        Screen::Plugins => t("Plugins", "插件配置"),
        Screen::Prompts => t("Prompts", "提示词"),
        Screen::Platforms => t("Platforms", "接入平台"),
        Screen::GlobalSettings => t("Global settings", "全局设置"),
    }
}

fn active_label(config: &AppConfig) -> String {
    config
        .active_provider_model_choices()
        .first()
        .map(|c| c.label())
        .unwrap_or_else(|| t("Not configured", "未配置"))
}

fn active_multimodal_label(config: &AppConfig) -> String {
    let choices = config.active_multimodal_provider_model_choices();
    if choices.is_empty() {
        t("Not configured", "未配置")
    } else if choices.len() == 1 {
        choices[0].label()
    } else {
        format!("{} {}", choices.len(), t("models", "个模型"))
    }
}

/// 通用模型选择列表渲染（文本模型/多模态模型共用）。
#[allow(clippy::too_many_arguments)]
fn draw_model_select(
    frame: &mut Frame,
    area: Rect,
    theme: &theme::Theme,
    title: &str,
    labels: &[String],
    current: &str,
    editing: bool,
    edit_buffer: &str,
    state: &mut ListState,
) {
    let inner = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(3),
    };

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(theme.outline))
        .style(Style::default().bg(theme.surface_bg));

    let title_line = Line::from(vec![
        Span::raw(" 当前: "),
        Span::styled(
            current,
            Style::default()
                .fg(theme.primary_fg)
                .bg(theme.primary_container_bg)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let hint = if editing {
        Line::from(format!(
            " 输入 provider/model 后按 Enter 应用（当前输入: {edit_buffer}）"
        ))
    } else {
        Line::from(" ↑↓ 导航  Enter 选择  n 输入新模型  Esc 返回 ")
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(title_line).block(Block::new()),
        layout[0],
    );

    let items: Vec<ListItem> = if labels.is_empty() {
        vec![ListItem::new("  （无可用模型，请先配置供应商）")
            .style(Style::default().fg(theme.on_surface_variant))]
    } else {
        labels
            .iter()
            .map(|label| {
                ListItem::new(Line::from(format!(" {label}")))
                    .style(Style::default().fg(theme.on_surface))
            })
            .collect()
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(theme.primary_fg)
                .bg(theme.primary_container_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▌ ");

    frame.render_stateful_widget(list, layout[2], state);

    let input_style = if editing {
        Style::default().fg(theme.on_surface).bg(theme.surface_dim_bg)
    } else {
        Style::default().fg(theme.on_surface_variant)
    };
    frame.render_widget(
        Paragraph::new(hint).style(input_style),
        layout[3],
    );
}

fn subagent_tiers_label(config: &AppConfig) -> String {
    let cheap = config.subagent_tiers.cheap.len();
    let balanced = config.subagent_tiers.balanced.len();
    let strong = config.subagent_tiers.strong.len();
    if cheap + balanced + strong == 0 {
        t("Inherit main pool", "继承主池")
    } else {
        format!("{cheap}/{balanced}/{strong}")
    }
}

fn platforms_label(config: &AppConfig) -> String {
    let mut active = Vec::new();
    if config.platforms.qq.enabled {
        active.push("QQ");
    }
    if config.platforms.telegram.as_ref().is_some_and(|t| t.enabled) {
        active.push("TG");
    }
    if config.platforms.qq_official.as_ref().is_some_and(|q| q.enabled) {
        active.push("QQ官");
    }
    if active.is_empty() {
        t("All disabled", "全部关闭")
    } else {
        active.join(",")
    }
}
