use crate::types::LimitedString;
#[cfg(feature = "defmt")]
use defmt::Format;
use defmt_or_log::derive_format_or_debug;
use thiserror::Error;

pub mod client;
pub mod server;

pub const HEADER_USER_AGENT: &str = "User-Agent";
pub const HEADER_ACCEPT: &str = "Accept";
pub const HEADER_CONTENT_TYPE: &str = "Content-Type";
pub const DEFAULT_USER_AGENT_VALUE: &str = "EPD_Frame_RS";
pub const CONTENT_TYPE_TEXT_HTML: &str = "text/html; charset=utf-8";
pub const CONTENT_TYPE_TEXT_PLAIN: &str = "text/plain; charset=utf-8";
pub const CONTENT_TYPE_APPLICATION_JSON: &str = "application/json";
pub const CONTENT_TYPE_APPLICATION_JAVASCRIPT: &str = "application/javascript";

#[derive(Debug, Error)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum HttpError {
    #[error("Http client error: {0:?}")]
    HttpClientError(LimitedString<128>),
    #[error("Http error status: {0:?}")]
    HttpStatusError(u16),
    #[error("Deserialization error: {0:?}")]
    DeserializationError(LimitedString<128>),
}

#[derive_format_or_debug]
pub struct Header<'a>(pub (&'a str, &'a str));

impl<'a> Header<'a> {
    pub fn new(name: &'a str, value: &'a str) -> Self {
        Self((name, value))
    }

    pub fn default_user_agent() -> Self {
        Self::new(HEADER_USER_AGENT, DEFAULT_USER_AGENT_VALUE)
    }
    pub fn user_agent(value: &'a str) -> Self {
        Self::new(HEADER_USER_AGENT, value)
    }

    pub fn accept(value: &'a str) -> Self {
        Self::new(HEADER_ACCEPT, value)
    }

    pub fn content_type(value: &'a str) -> Self {
        Self::new(HEADER_CONTENT_TYPE, value)
    }
}

impl<'a> From<Header<'a>> for (&'a str, &'a str) {
    fn from(Header(h): Header<'a>) -> Self {
        h
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[derive_format_or_debug]
#[repr(u8)]
pub enum ContentType {
    TextHtml,
    TextPlain,
    ApplicationJson,
}

impl From<ContentType> for Header<'static> {
    fn from(value: ContentType) -> Self {
        match value {
            ContentType::TextHtml => Header::content_type(CONTENT_TYPE_TEXT_HTML),
            ContentType::TextPlain => Header::content_type(CONTENT_TYPE_TEXT_PLAIN),
            ContentType::ApplicationJson => Header::content_type(CONTENT_TYPE_APPLICATION_JSON),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[derive_format_or_debug]
pub struct StatusCode(pub u16);

impl StatusCode {
    pub fn is_successful(&self) -> bool {
        (200..=299).contains(&self.0)
    }
}
