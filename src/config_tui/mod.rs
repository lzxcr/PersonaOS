//! TUI configuration interface — MD3-themed ratatui UI.
//!
//! Replaces the legacy `config_tui_old.rs` (8400-line raw crossterm).

mod theme;
pub mod pages;

use crate::config::{ActiveProviderModelConfig, AppConfig};
use crate::llm::ThinkingVariantPreferences;
use crate::paths::PersonaPaths;
use anyhow::Result;
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
    Quit,
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
            providers: pages::providers::ProvidersPage::default(),
            plugins: pages::plugins::PluginsPage::default(),
            prompts: pages::prompts::PromptsPage::default(),
            platforms: pages::platforms::PlatformsPage::default(),
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
                        self.screen = match idx {
                            0 => Screen::TextModel,
                            1 => Screen::MultimodalModel,
                            2 => Screen::SubagentTiers,
                            3 => Screen::Providers,
                            4 => Screen::Plugins,
                            5 => Screen::Prompts,
                            6 => Screen::Platforms,
                            7 => Screen::GlobalSettings,
                            8 => Screen::Quit,
                            _ => Screen::MainMenu,
                        };
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
            Screen::Quit => self.draw_quit(frame, main_area, &theme),
            _ => self.draw_placeholder(frame, main_area, &theme),
        }

        // ── Status bar ─────────────────────────────────────────────────
        let help = Span::styled(
            " q/Esc 退出   ↑↓ 导航   Enter 选择 ",
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
            format!(" 文本模型 (当前: {active})"),
            format!(" 多模态模型 (当前: {multimodal})"),
            format!(" 子代理档位池 ({})", subagent_tiers_label(&self.config)),
            " 供应商和模型".to_string(),
            " 插件配置".to_string(),
            " 自定义提示词".to_string(),
            format!(" IM 平台 ({})", platforms_label(&self.config)),
            " 全局参数设置".to_string(),
            " 保存并退出".to_string(),
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
            .highlight_symbol(" ▸ ");

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
            .highlight_symbol(" ▸ ");

        frame.render_stateful_widget(list, layout[2], &mut self.subagent.state);
    }

    fn draw_providers(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let inner = Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(3),
        };

        // ── Edit mode: field form ──────────────────────────────────────
        if self.providers.editing {
            let Some(selected) = self.providers.state.selected() else {
                return;
            };
            let Some(provider) = self.config.providers.get(selected) else {
                return;
            };
            let field = pages::providers::EDITABLE_FIELDS
                [self.providers.edit_field.min(pages::providers::EDITABLE_FIELDS.len() - 1)];

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ])
                .split(inner);

            let value = pages::providers::field_value(provider, self.providers.edit_field);
            let value_display = if field == "api_key" {
                // 展示脱敏后的 key 或环境变量名。
                if value.starts_with("sk-") {
                    format!("{}…", &value[..6.min(value.len())])
                } else {
                    value.clone()
                }
            } else {
                value.clone()
            };

            let label_line = Line::from(vec![
                Span::styled(
                    format!(" {field} "),
                    Style::default()
                        .fg(theme.primary_fg)
                        .bg(theme.primary_container_bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" 当前: {value_display}")),
            ]);
            frame.render_widget(Paragraph::new(label_line), layout[0]);

            let hint = Line::from(format!(
                " 输入新值后按 Enter 应用  ↑↓ 切换字段  n 下一字段  Esc 返回列表 "
            ));
            frame.render_widget(
                Paragraph::new(hint).style(Style::default().fg(theme.on_surface_variant)),
                layout[1],
            );

            let input_box = Paragraph::new(self.providers.edit_buffer.clone())
                .block(Block::bordered().border_type(BorderType::Rounded))
                .style(Style::default().fg(theme.on_surface).bg(theme.surface_dim_bg));
            frame.render_widget(input_box, layout[2]);

            let help_line = Line::from(format!(
                " 字段 {} / {} ",
                self.providers.edit_field + 1,
                pages::providers::EDITABLE_FIELDS.len()
            ));
            frame.render_widget(
                Paragraph::new(help_line).style(Style::default().fg(theme.on_surface_variant)),
                layout[3],
            );
            return;
        }

        // ── List mode ──────────────────────────────────────────────────
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" 供应商和模型 ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(theme.outline))
            .style(Style::default().bg(theme.surface_bg));

        let rows = pages::providers::provider_rows(
            &self.config.providers,
            &self.config.active_provider,
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner);

        let hint = if self.providers.confirming_delete {
            Line::from(" 确认删除? y 删除  n 取消 ")
        } else {
            Line::from(" ↑↓ 导航  Enter 编辑  a 添加  d 删除  s 设为当前  Esc 返回 ")
        };
        frame.render_widget(
            Paragraph::new(hint).style(Style::default().fg(theme.on_surface_variant)),
            layout[0],
        );

        let items: Vec<ListItem> = if rows.is_empty() {
            vec![ListItem::new("  （无供应商，按 a 添加）")
                .style(Style::default().fg(theme.on_surface_variant))]
        } else {
            rows.iter()
                .map(|row| {
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
            .highlight_symbol(" ▸ ");

        frame.render_stateful_widget(list, layout[1], &mut self.providers.state);

        let footer = Line::from(format!(" 当前: {}", self.config.active_provider));
        frame.render_widget(
            Paragraph::new(footer).style(Style::default().fg(theme.on_surface_variant)),
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

        if self.global.editing {
            let field = pages::global::GLOBAL_FIELDS
                [self.global.state.selected().unwrap_or(0)
                    .min(pages::global::GLOBAL_FIELDS.len() - 1)];
            let value = pages::global::field_value(&self.config, self.global.state.selected().unwrap_or(0));

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ])
                .split(inner);

            let label_line = Line::from(vec![
                Span::styled(
                    format!(" {field} "),
                    Style::default()
                        .fg(theme.primary_fg)
                        .bg(theme.primary_container_bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" 当前: {value}")),
            ]);
            frame.render_widget(Paragraph::new(label_line), layout[0]);

            let hint = Line::from(" 输入新值后按 Enter 应用  ↑↓ 切换字段  Esc 返回 ");
            frame.render_widget(
                Paragraph::new(hint).style(Style::default().fg(theme.on_surface_variant)),
                layout[1],
            );

            let input_box = Paragraph::new(self.global.edit_buffer.clone())
                .block(Block::bordered().border_type(BorderType::Rounded))
                .style(Style::default().fg(theme.on_surface).bg(theme.surface_dim_bg));
            frame.render_widget(input_box, layout[2]);

            let help_line = Line::from(format!(
                " 字段 {} / {}  (bool: true/false)",
                self.global.state.selected().unwrap_or(0) + 1,
                pages::global::GLOBAL_FIELDS.len()
            ));
            frame.render_widget(
                Paragraph::new(help_line).style(Style::default().fg(theme.on_surface_variant)),
                layout[3],
            );
            return;
        }

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" 全局参数设置 ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(theme.outline))
            .style(Style::default().bg(theme.surface_bg));

        let rows = pages::global::global_rows(&self.config);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(inner);

        let hint = Line::from(" ↑↓ 导航  Enter 编辑  Esc 返回 ");
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
            .highlight_symbol(" ▸ ");

        frame.render_stateful_widget(list, layout[1], &mut self.global.state);
    }

    fn draw_placeholder(&self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!(" {} ", screen_title(&self.screen)))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(theme.outline))
            .style(Style::default().bg(theme.surface_bg));

        let text = Paragraph::new(
            "此页面正在迁移到 ratatui...\n\n按 Esc 返回主菜单"
        )
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.on_surface));

        frame.render_widget(text, area);
    }

    fn draw_quit(&mut self, frame: &mut Frame, area: Rect, theme: &theme::Theme) {
        self.quit = true;
        self.draw_placeholder(frame, area, theme);
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
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = self.main_menu_state.selected().unwrap_or(0);
                        self.main_menu_state
                            .select(Some(i.saturating_sub(1)));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = self.main_menu_state.selected().unwrap_or(0);
                        self.main_menu_state.select(Some(
                            (i + 1).min(MAIN_MENU_ITEMS - 1),
                        ));
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
                _ => match code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        return Ok(Some(AppEvent::Back));
                    }
                    _ => {}
                },
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

        // ── Edit mode ──────────────────────────────────────────────────
        if self.providers.editing {
            return match code {
                KeyCode::Esc => {
                    self.providers.editing = false;
                    self.providers.edit_buffer.clear();
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
                        }
                    }
                    // 若编辑的是 id，保持选中位置。
                    self.providers.editing = false;
                    self.providers.edit_buffer.clear();
                    AppEvent::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.providers.edit_field =
                        self.providers.edit_field.saturating_sub(1);
                    self.reload_provider_edit_buffer();
                    AppEvent::None
                }
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('n') => {
                    self.providers.edit_field = (self.providers.edit_field + 1)
                        .min(pages::providers::EDITABLE_FIELDS.len() - 1);
                    self.reload_provider_edit_buffer();
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

        // ── List mode ──────────────────────────────────────────────────
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
            KeyCode::Enter => {
                self.reload_provider_edit_buffer();
                self.providers.editing = true;
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
                    AppEvent::None
                }
                KeyCode::Enter => {
                    let field = self.global.state.selected().unwrap_or(0);
                    let value = self.global.edit_buffer.clone();
                    if pages::global::apply_field(&mut self.config, field, &value) {
                        self.dirty = true;
                    }
                    self.global.editing = false;
                    self.global.edit_buffer.clear();
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
}

const MAIN_MENU_ITEMS: usize = 9;

enum AppEvent {
    SelectMainMenu(usize),
    Back,
    Quit,
    None,
}

// ── Helpers ────────────────────────────────────────────────────────────

fn screen_title(screen: &Screen) -> &'static str {
    match screen {
        Screen::MainMenu => "主菜单",
        Screen::TextModel => "文本模型",
        Screen::MultimodalModel => "多模态模型",
        Screen::SubagentTiers => "子代理档位",
        Screen::Providers => "供应商",
        Screen::Plugins => "插件配置",
        Screen::Prompts => "提示词",
        Screen::Platforms => "IM 平台",
        Screen::GlobalSettings => "全局设置",
        Screen::Quit => "退出",
    }
}

fn active_label(config: &AppConfig) -> String {
    config
        .active_provider_model_choices()
        .first()
        .map(|c| c.label())
        .unwrap_or_else(|| "未配置".to_string())
}

fn active_multimodal_label(config: &AppConfig) -> String {
    let choices = config.active_multimodal_provider_model_choices();
    if choices.is_empty() {
        "未配置".to_string()
    } else if choices.len() == 1 {
        choices[0].label()
    } else {
        format!("{} 个模型", choices.len())
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
        .highlight_symbol(" ▸ ");

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
        "继承主池".to_string()
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
        "全部关闭".to_string()
    } else {
        active.join(",")
    }
}
