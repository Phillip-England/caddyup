use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{App, Field, Mode, field_display};

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_header(frame, root[0], app);
    draw_body(frame, root[1], app);
    draw_footer(frame, root[2], app);

    if app.mode == Mode::Edit {
        draw_edit_popup(frame, app);
    }
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let text = vec![Line::from(vec![
        Span::styled(
            "caddyup",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::raw(app.document.path.display().to_string()),
    ])];

    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn draw_body(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(36), Constraint::Percentage(64)])
        .split(area);

    draw_sites(frame, chunks[0], app);
    draw_fields(frame, chunks[1], app);
}

fn draw_sites(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .document
        .config
        .sites
        .iter()
        .map(|site| ListItem::new(site.title()))
        .collect();
    let mut state = ListState::default().with_selected(Some(app.selected_site));

    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().title("Servers").borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
            .highlight_symbol("> "),
        area,
        &mut state,
    );
}

fn draw_fields(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let Some(site) = app.current_site() else {
        return;
    };

    let items: Vec<ListItem> = Field::ALL
        .iter()
        .map(|field| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<18}", field.label()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(field_display(site, *field)),
            ]))
        })
        .collect();
    let mut state = ListState::default().with_selected(Some(app.selected_field));

    frame.render_stateful_widget(
        List::new(items)
            .block(
                Block::default()
                    .title("Configuration")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> "),
        area,
        &mut state,
    );
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let help = match app.mode {
        Mode::Navigate => "arrows/hjkl move  enter edit/toggle  a add  d delete  s save  q quit",
        Mode::Edit => "enter apply  esc cancel",
    };
    let text = vec![
        Line::from(help),
        Line::from(Span::styled(
            app.status.clone(),
            Style::default().fg(Color::Green),
        )),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_edit_popup(frame: &mut Frame<'_>, app: &App) {
    let area = centered_rect(60, 20, frame.area());
    let field = Field::ALL[app.selected_field];
    let text = vec![
        Line::from(format!("{}:", field.label())),
        Line::from(app.input.clone()),
    ];

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().title("Edit").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
