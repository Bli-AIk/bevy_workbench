//! Internationalization support using Fluent.

use bevy::prelude::*;
use fluent_bundle::FluentResource;
use fluent_bundle::concurrent::FluentBundle;
use std::sync::Arc;
use unic_langid::LanguageIdentifier;

/// Supported interface languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Locale {
    En,
    ZhCn,
}

impl Default for Locale {
    fn default() -> Self {
        Self::from_system()
    }
}

impl Locale {
    pub const ALL: &[Locale] = &[Locale::En, Locale::ZhCn];

    pub fn label(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::ZhCn => "简体中文",
        }
    }

    /// Detect the best locale from the system language setting.
    pub fn from_system() -> Self {
        if let Some(tag) = sys_locale::get_locale() {
            let lower = tag.to_lowercase();
            if lower.starts_with("zh") {
                return Locale::ZhCn;
            }
        }
        Locale::En
    }

    fn lang_id(&self) -> LanguageIdentifier {
        match self {
            Locale::En => "en".parse().unwrap(),
            Locale::ZhCn => "zh-CN".parse().unwrap(),
        }
    }

    fn ftl_source(&self) -> &'static str {
        match self {
            Locale::En => include_str!("../locales/en.ftl"),
            Locale::ZhCn => include_str!("../locales/zh-CN.ftl"),
        }
    }
}

/// Resource providing localized strings.
#[derive(Resource)]
pub struct I18n {
    bundle: Arc<FluentBundle<FluentResource>>,
    pub locale: Locale,
    /// Custom FTL sources registered by user panels (indexed by Locale).
    custom_sources: Vec<(Locale, String)>,
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Locale::default())
    }
}

impl I18n {
    pub fn new(locale: Locale) -> Self {
        let bundle = Self::build_bundle(locale, &[]);
        Self {
            bundle: Arc::new(bundle),
            locale,
            custom_sources: Vec::new(),
        }
    }

    /// Register a custom FTL source for a given locale.
    /// The bundle is rebuilt immediately if the locale matches.
    pub fn add_custom_source(&mut self, locale: Locale, ftl: impl Into<String>) {
        self.custom_sources.push((locale, ftl.into()));
        self.bundle = Arc::new(Self::build_bundle(self.locale, &self.custom_sources));
    }

    /// Change the active locale.
    pub fn set_locale(&mut self, locale: Locale) {
        if self.locale != locale {
            self.locale = locale;
            self.bundle = Arc::new(Self::build_bundle(locale, &self.custom_sources));
        }
    }

    /// Get a localized string by message ID.
    pub fn t(&self, id: &str) -> String {
        let msg = self.bundle.get_message(id);
        match msg {
            Some(msg) => {
                let pattern = msg.value().expect("message has no value");
                let mut errors = vec![];
                self.bundle
                    .format_pattern(pattern, None, &mut errors)
                    .to_string()
            }
            None => id.to_string(),
        }
    }

    fn build_bundle(locale: Locale, custom: &[(Locale, String)]) -> FluentBundle<FluentResource> {
        let lang_id = locale.lang_id();
        let source = locale.ftl_source();
        let resource = FluentResource::try_new(source.to_string()).expect("valid FTL resource");
        let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
        bundle.add_resource(resource).expect("add FTL resource");

        // Add custom sources matching this locale
        for (loc, ftl) in custom {
            if *loc == locale
                && let Ok(res) = FluentResource::try_new(ftl.clone())
            {
                let _ = bundle.add_resource(res);
            }
        }

        bundle
    }
}
