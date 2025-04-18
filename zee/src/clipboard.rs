use std::sync::Arc;

use crate::error::Result;

pub trait Clipboard {
    fn get_contents(&self) -> Result<String>;
    fn set_contents(&self, contents: String) -> Result<()>;
}

pub fn create() -> Result<Arc<dyn Clipboard>> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "system-clipboard")] {
            system::create()
        } else {
            local::create()
        }
    }
}

#[cfg(feature = "system-clipboard")]
mod system {
    use wl_clipboard_rs::{paste::{self, get_contents, ClipboardType, Seat}, copy::{self, Options, Source}};
    use std::{sync::Arc, io::Read};

    use crate::error::Result;

    pub(crate) fn create() -> Result<Arc<dyn super::Clipboard>> {
        Ok(Arc::new(SystemClipboard))
    }

    struct SystemClipboard;

    impl super::Clipboard for SystemClipboard {
        fn get_contents(&self) -> Result<String> {
            let result = get_contents(ClipboardType::Regular, Seat::Unspecified, paste::MimeType::Text);
            Ok(match result {
                Ok((mut pipe, _)) => {
                    let mut contents = vec![];
                    pipe.read_to_end(&mut contents).unwrap();
                    String::from_utf8_lossy(&contents).to_string()
                }
                _ => {String::from("wa")}
            })
        }

        fn set_contents(&self, contents: String) -> Result<()> {
            let opts = Options::new();
            opts.copy(Source::Bytes(contents.into_bytes().into()), copy::MimeType::Autodetect).unwrap();
            Ok(())
        }
    }
}

#[cfg(not(feature = "system-clipboard"))]
mod local {
    use parking_lot::RwLock;
    use std::sync::Arc;

    use super::Clipboard;
    use crate::error::Result;

    pub(crate) fn create() -> Result<Arc<dyn Clipboard>> {
        Ok(Arc::new(LocalClipboard::new()))
    }

    struct LocalClipboard {
        contents: RwLock<String>,
    }

    impl LocalClipboard {
        fn new() -> Self {
            Self {
                contents: RwLock::new(String::new()),
            }
        }
    }

    impl Clipboard for LocalClipboard {
        fn get_contents(&self) -> Result<String> {
            Ok(self.contents.read().clone())
        }

        fn set_contents(&self, contents: String) -> Result<()> {
            *self.contents.write() = contents;
            Ok(())
        }
    }
}
