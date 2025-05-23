//! Interacting with the system pasteboard/clipboard.
//! 
//! This is largly copied from Druid's implementation.
pub use crate::backend::clipboard as backend;

/// A handle to the system clipboard.
///
/// To get access to the global clipboard, call [`SystemTaskHandle::clipboard`].
///
///
/// # Working with text
///
/// Copying and pasting text is simple, using [`Clipboard::put_string`] and
/// [`Clipboard::get_string`]. If this is all you need, you're in luck.
///
/// # Advanced usage
///
/// When working with data more complicated than plaintext, you will generally
/// want to make that data available in multiple formats.
///
/// For instance, if you are writing an image editor, you may have a preferred
/// private format, that preserves metadata or layer information; but in order
/// to interoperate with your user's other programs, you might also make your
/// data available as an SVG, for other editors, and a bitmap image for applications
/// that can accept general image data.
///
/// ## `FormatId`entifiers
///
/// In order for other applications to find data we put on the clipboard,
/// (and for us to use data from other applications) we need to use agreed-upon
/// identifiers for our data types. On macOS, these should be
/// [`Universal Type Identifier`]s; on other platforms they appear to be
/// mostly [MIME types]. Several common types are exposed as constants on
/// [`ClipboardFormat`], these `const`s are set per-platform.
///
/// When defining custom formats, you should use the correct identifier for
/// the current platform.
///
/// ## Setting custom data
///
/// To put custom data on the clipboard, you create a [`ClipboardFormat`] for
/// each type of data you support. You are responsible for ensuring that the
/// data is already correctly serialized.
///
///
/// ### `ClipboardFormat` for text
///
/// If you wish to put text on the clipboard in addition to other formats,
/// take special care to use `ClipboardFormat::TEXT` as the [`FormatId`]. On
/// windows, we treat this identifier specially, and make sure the data is
/// encoded as a wide string; all other data going into and out of the
/// clipboard is treated as an array of bytes.
///
/// # Examples
///
/// ## Getting and setting text:
///
/// ```no_run
/// use jihaz::sys_task::{SystemTaskHandle, Clipboard};
///
/// let mut clipboard = SystemTaskHandle::global().clipboard();
/// clipboard.put_string("watch it there pal");
/// if let Some(contents) = clipboard.get_string() {
///     assert_eq!("what it there pal", contents.as_str());
/// }
///
/// ```
///
///  ## Copying multi-format data
///
///  ```no_run
/// use jihaz::sys_task::{SystemTaskHandle, Clipboard, ClipboardFormat};
///
/// let mut clipboard = SystemTaskHandle::global().clipboard();
///
/// let custom_type_id = "io.xieditor.path-clipboard-type";
///
/// let formats = [
///     ClipboardFormat::new(custom_type_id, make_custom_data()),
///     ClipboardFormat::new(ClipboardFormat::SVG, make_svg_data()),
///     ClipboardFormat::new(ClipboardFormat::PDF, make_pdf_data()),
/// ];
///
/// clipboard.put_formats(&formats);
///
/// # fn make_custom_data() -> Vec<u8> { unimplemented!() }
/// # fn make_svg_data() -> Vec<u8> { unimplemented!() }
/// # fn make_pdf_data() -> Vec<u8> { unimplemented!() }
///  ```
/// ## Supporting multi-format paste
///
/// ```no_run
/// use jihaz::sys_task::{SystemTaskHandle, Clipboard, ClipboardFormat};
///
/// let clipboard = SystemTaskHandle::global().clipboard();
///
/// let custom_type_id = "io.xieditor.path-clipboard-type";
/// let supported_types = &[custom_type_id, ClipboardFormat::SVG, ClipboardFormat::PDF];
/// let best_available_type = clipboard.preferred_format(supported_types);
///
/// if let Some(format) = best_available_type {
///     let data = clipboard.get_format(format).expect("I promise not to unwrap in production");
///     do_something_with_data(format, data)
/// }
///
/// # fn do_something_with_data(_: &str, _: Vec<u8>) {}
/// ```
///
/// [`SystemTaskHandle::clipboard`]: crate::SystemTaskHandle::clipboard
/// [`Universal Type Identifier`]: https://escapetech.eu/manuals/qdrop/uti.html
/// [MIME types]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types
#[derive(Debug, Clone)]
pub struct Clipboard(pub(crate) backend::Clipboard);

impl Clipboard {
    /// Put a string onto the system clipboard.
    pub fn put_string(&mut self, s: impl AsRef<str>) {
        self.0.put_string(s);
    }

    /// Put multi-format data on the system clipboard.
    pub fn put_formats(&mut self, formats: &[ClipboardFormat]) {
        self.0.put_formats(formats)
    }

    /// Get a string from the system clipboard, if one is available.
    pub fn get_string(&self) -> Option<String> {
        self.0.get_string()
    }

    /// Given a list of supported clipboard types, returns the supported type which has
    /// highest priority on the system clipboard, or `None` if no types are supported.
    pub fn preferred_format(&self, formats: &[FormatId]) -> Option<FormatId> {
        self.0.preferred_format(formats)
    }

    /// Return data in a given format, if available.
    ///
    /// It is recommended that the [`FormatId`] argument be a format returned by
    /// [`Clipboard::preferred_format`].
    ///
    /// [`Clipboard::preferred_format`]: struct.Clipboard.html#method.preferred_format
    /// [`FormatId`]: type.FormatId.html
    pub fn get_format(&self, format: FormatId) -> Option<Vec<u8>> {
        self.0.get_format(format)
    }

    /// For debugging: print the resolved identifiers for each type currently
    /// on the clipboard.
    #[doc(hidden)]
    pub fn available_type_names(&self) -> Vec<String> {
        self.0.available_type_names()
    }
}

/// A type identifier for the system clipboard.
///
/// These should be [`UTI` strings] on macOS, and (by convention?) [MIME types] elsewhere.
///
/// [`UTI` strings]: https://escapetech.eu/manuals/qdrop/uti.html
/// [MIME types]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types
pub type FormatId = &'static str;

/// Data coupled with a type identifier.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "wayland", allow(dead_code))]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct ClipboardFormat {
    pub(crate) identifier: FormatId,
    pub(crate) data: Vec<u8>,
}

impl ClipboardFormat {
    /// Create a new `ClipboardFormat` with the given `FormatId` and bytes.
    ///
    /// You are responsible for ensuring that this data can be interpreted
    /// as the provided format.
    pub fn new(identifier: FormatId, data: impl Into<Vec<u8>>) -> Self {
        let data = data.into();
        ClipboardFormat { identifier, data }
    }
}

impl From<String> for ClipboardFormat {
    fn from(src: String) -> ClipboardFormat {
        let data = src.into_bytes();
        ClipboardFormat::new(ClipboardFormat::TEXT, data)
    }
}

impl From<&str> for ClipboardFormat {
    fn from(src: &str) -> ClipboardFormat {
        src.to_string().into()
    }
}

impl From<backend::Clipboard> for Clipboard {
    fn from(src: backend::Clipboard) -> Clipboard {
        Clipboard(src)
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        impl ClipboardFormat {
            pub const PDF: &'static str = "com.adobe.pdf";
            pub const TEXT: &'static str = "public.utf8-plain-text";
            pub const SVG: &'static str = "public.svg-image";
        }
    } else {
        impl ClipboardFormat {
            cfg_if::cfg_if! {
                if #[cfg(any(target_os = "freebsd", target_os = "linux", target_os = "openbsd"))] {
                    // trial and error; this is the most supported string type for gtk?
                    pub const TEXT: &'static str = "UTF8_STRING";
                } else {
                    pub const TEXT: &'static str = "text/plain";
                }
            }
            pub const PDF: &'static str = "application/pdf";
            pub const SVG: &'static str = "image/svg+xml";
        }
    }
}
