use crate::{
    caddyfile::CaddyDocument,
    model::{CaddyConfig, Site},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Field {
    Address,
    Upstream,
    RateLimitEnabled,
    RateLimitEvents,
    RateLimitWindow,
    RateLimitZone,
    EncodeZstd,
    EncodeGzip,
    TlsInternal,
}

impl Field {
    pub const ALL: [Self; 9] = [
        Self::Address,
        Self::Upstream,
        Self::RateLimitEnabled,
        Self::RateLimitEvents,
        Self::RateLimitWindow,
        Self::RateLimitZone,
        Self::EncodeZstd,
        Self::EncodeGzip,
        Self::TlsInternal,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Address => "Server address",
            Self::Upstream => "Reverse proxy",
            Self::RateLimitEnabled => "Rate limit",
            Self::RateLimitEvents => "Events",
            Self::RateLimitWindow => "Window",
            Self::RateLimitZone => "Zone",
            Self::EncodeZstd => "Encode zstd",
            Self::EncodeGzip => "Encode gzip",
            Self::TlsInternal => "TLS internal",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Navigate,
    Edit,
}

#[derive(Clone, Debug)]
pub struct App {
    pub document: CaddyDocument,
    pub selected_site: usize,
    pub selected_field: usize,
    pub mode: Mode,
    pub input: String,
    pub status: String,
    pub should_quit: bool,
}

impl App {
    pub fn new(mut document: CaddyDocument) -> Self {
        if document.config.sites.is_empty() {
            document.config = CaddyConfig::with_example_site();
        }

        let path = document.path.display().to_string();
        Self {
            document,
            selected_site: 0,
            selected_field: 0,
            mode: Mode::Navigate,
            input: String::new(),
            status: format!("Loaded {path}"),
            should_quit: false,
        }
    }

    pub fn current_site(&self) -> Option<&Site> {
        self.document.config.sites.get(self.selected_site)
    }

    pub fn move_site(&mut self, amount: isize) {
        let len = self.document.config.sites.len();
        if len == 0 {
            self.selected_site = 0;
            return;
        }
        self.selected_site = wrap_index(self.selected_site, len, amount);
    }

    pub fn move_field(&mut self, amount: isize) {
        self.selected_field = wrap_index(self.selected_field, Field::ALL.len(), amount);
    }

    pub fn add_site(&mut self) {
        self.document.config.sites.push(Site::default());
        self.selected_site = self.document.config.sites.len() - 1;
        self.selected_field = 0;
        self.status = "Added server".to_string();
    }

    pub fn delete_site(&mut self) {
        if self.document.config.sites.len() <= 1 {
            self.status = "Keep at least one server; edit it or add another first".to_string();
            return;
        }

        self.document.config.sites.remove(self.selected_site);
        self.selected_site = self
            .selected_site
            .min(self.document.config.sites.len().saturating_sub(1));
        self.status = "Deleted server".to_string();
    }

    pub fn begin_edit(&mut self) {
        let field = Field::ALL[self.selected_field];
        if self.toggle_field(field) {
            return;
        }

        self.input = self.field_value(field);
        self.mode = Mode::Edit;
        self.status = format!("Editing {}", field.label());
    }

    pub fn cancel_edit(&mut self) {
        self.mode = Mode::Navigate;
        self.input.clear();
        self.status = "Edit cancelled".to_string();
    }

    pub fn commit_edit(&mut self) {
        let field = Field::ALL[self.selected_field];
        let value = self.input.trim().to_string();
        let Some(site) = self.document.config.sites.get_mut(self.selected_site) else {
            return;
        };

        match field {
            Field::Address => site.address = empty_fallback(value, "example.com"),
            Field::Upstream => site.upstream = empty_fallback(value, "127.0.0.1:3000"),
            Field::RateLimitEvents => {
                site.rate_limit_events = value.parse().unwrap_or(site.rate_limit_events);
            }
            Field::RateLimitWindow => site.rate_limit_window = empty_fallback(value, "1m"),
            Field::RateLimitZone => site.rate_limit_zone = empty_fallback(value, "dynamic"),
            Field::RateLimitEnabled
            | Field::EncodeZstd
            | Field::EncodeGzip
            | Field::TlsInternal => {}
        }

        self.input.clear();
        self.mode = Mode::Navigate;
        self.status = format!("Updated {}", field.label());
    }

    pub fn save(&mut self) {
        match self.document.save() {
            Ok(()) => self.status = format!("Saved {}", self.document.path.display()),
            Err(err) => self.status = format!("Save failed: {err:#}"),
        }
    }

    fn toggle_field(&mut self, field: Field) -> bool {
        let Some(site) = self.document.config.sites.get_mut(self.selected_site) else {
            return false;
        };

        match field {
            Field::RateLimitEnabled => site.rate_limit_enabled = !site.rate_limit_enabled,
            Field::EncodeZstd => site.encode_zstd = !site.encode_zstd,
            Field::EncodeGzip => site.encode_gzip = !site.encode_gzip,
            Field::TlsInternal => site.tls_internal = !site.tls_internal,
            _ => return false,
        }
        self.status = format!("Toggled {}", field.label());
        true
    }

    fn field_value(&self, field: Field) -> String {
        let Some(site) = self.current_site() else {
            return String::new();
        };

        match field {
            Field::Address => site.address.clone(),
            Field::Upstream => site.upstream.clone(),
            Field::RateLimitEnabled => bool_label(site.rate_limit_enabled).to_string(),
            Field::RateLimitEvents => site.rate_limit_events.to_string(),
            Field::RateLimitWindow => site.rate_limit_window.clone(),
            Field::RateLimitZone => site.rate_limit_zone.clone(),
            Field::EncodeZstd => bool_label(site.encode_zstd).to_string(),
            Field::EncodeGzip => bool_label(site.encode_gzip).to_string(),
            Field::TlsInternal => bool_label(site.tls_internal).to_string(),
        }
    }
}

pub fn field_display(site: &Site, field: Field) -> String {
    match field {
        Field::Address => site.address.clone(),
        Field::Upstream => site.upstream.clone(),
        Field::RateLimitEnabled => bool_label(site.rate_limit_enabled).to_string(),
        Field::RateLimitEvents => site.rate_limit_events.to_string(),
        Field::RateLimitWindow => site.rate_limit_window.clone(),
        Field::RateLimitZone => site.rate_limit_zone.clone(),
        Field::EncodeZstd => bool_label(site.encode_zstd).to_string(),
        Field::EncodeGzip => bool_label(site.encode_gzip).to_string(),
        Field::TlsInternal => bool_label(site.tls_internal).to_string(),
    }
}

fn bool_label(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

fn empty_fallback(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn wrap_index(index: usize, len: usize, amount: isize) -> usize {
    (index as isize + amount).rem_euclid(len as isize) as usize
}
