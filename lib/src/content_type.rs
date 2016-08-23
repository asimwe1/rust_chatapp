pub use mime::{Mime, TopLevel, SubLevel};

use std::str::FromStr;
use mime::{Param};
use self::TopLevel::{Text, Application};
use self::SubLevel::{Json, Html};

#[derive(Debug, Clone)]
pub struct ContentType(pub TopLevel, pub SubLevel, pub Option<Vec<Param>>);

impl ContentType {
    #[inline(always)]
    pub fn of(t: TopLevel, s: SubLevel) -> ContentType {
        ContentType(t, s, None)
    }

    #[inline(always)]
    pub fn any() -> ContentType {
        ContentType::of(TopLevel::Star, SubLevel::Star)
    }

    pub fn is_json(&self) -> bool {
        match *self {
            ContentType(Application, Json, _) => true,
            _ => false,
        }
    }

    pub fn is_any(&self) -> bool {
        match *self {
            ContentType(TopLevel::Star, SubLevel::Star, None) => true,
            _ => false,
        }
    }

    pub fn is_ext(&self) -> bool {
        if let TopLevel::Ext(_) = self.0 {
            true
        } else if let SubLevel::Ext(_) = self.1 {
            true
        } else {
            false
        }
    }

    pub fn is_html(&self) -> bool {
        match *self {
            ContentType(Text, Html, _) => true,
            _ => false,
        }
    }
}

impl Into<Mime> for ContentType {
    fn into(self) -> Mime {
        Mime(self.0, self.1, self.2.unwrap_or_default())
    }
}

impl From<Mime> for ContentType {
    fn from(mime: Mime) -> ContentType {
        let params = match mime.2.len() {
            0 => None,
            _ => Some(mime.2)
        };

        ContentType(mime.0, mime.1, params)
    }
}

impl FromStr for ContentType {
    type Err = ();

    fn from_str(raw: &str) -> Result<ContentType, ()> {
        let mime = Mime::from_str(raw)?;
        Ok(ContentType::from(mime))
    }
}
