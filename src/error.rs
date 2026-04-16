use crate::xml;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("XML parse error at {loc}: {message}")]
    XmlParse { loc: xml::Location, message: String },

    #[error("validation error at {loc}: {message}")]
    Validation { loc: xml::Location, message: String },
}

impl Error {
    pub(crate) fn xml(loc: xml::Location, message: impl Into<String>) -> Self {
        Self::XmlParse {
            loc,
            message: message.into(),
        }
    }

    pub(crate) fn validation(loc: xml::Location, message: impl Into<String>) -> Self {
        Self::Validation {
            loc,
            message: message.into(),
        }
    }
}
