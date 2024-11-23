#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileExtension {
    Html,
    Css,
    Js,
    Png,
    Jpg,
    Jpeg,
    Gif,
    Svg,
    Ico,
    Json,
    Woff,
    Woff2,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum MimeType {
    TextHtml,
    TextCss,
    ApplicationJavaScript,
    ImagePng,
    ImageJpeg,
    ImageGif,
    ImageSvg,
    ImageIcon,
    ApplicationJson,
    FontWoff,
    FontWoff2,
    ApplicationOctet,
}

impl MimeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TextHtml => "text/html",
            Self::TextCss => "text/css",
            Self::ApplicationJavaScript => "application/javascript",
            Self::ImagePng => "image/png",
            Self::ImageJpeg => "image/jpeg",
            Self::ImageGif => "image/gif",
            Self::ImageSvg => "image/svg+xml",
            Self::ImageIcon => "image/x-icon",
            Self::ApplicationJson => "application/json",
            Self::FontWoff => "font/woff",
            Self::FontWoff2 => "font/woff2",
            Self::ApplicationOctet => "application/octet-stream",
        }
    }
}

impl FileExtension {
    pub fn from_str(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "html" => Self::Html,
            "css" => Self::Css,
            "js" => Self::Js,
            "png" => Self::Png,
            "jpg" => Self::Jpg,
            "jpeg" => Self::Jpeg,
            "gif" => Self::Gif,
            "svg" => Self::Svg,
            "ico" => Self::Ico,
            "json" => Self::Json,
            "woff" => Self::Woff,
            "woff2" => Self::Woff2,
            _ => Self::Unknown,
        }
    }

    pub fn mime_type(&self) -> MimeType {
        match self {
            Self::Html => MimeType::TextHtml,
            Self::Css => MimeType::TextCss,
            Self::Js => MimeType::ApplicationJavaScript,
            Self::Png => MimeType::ImagePng,
            Self::Jpg | Self::Jpeg => MimeType::ImageJpeg,
            Self::Gif => MimeType::ImageGif,
            Self::Svg => MimeType::ImageSvg,
            Self::Ico => MimeType::ImageIcon,
            Self::Json => MimeType::ApplicationJson,
            Self::Woff => MimeType::FontWoff,
            Self::Woff2 => MimeType::FontWoff2,
            Self::Unknown => MimeType::ApplicationOctet,
        }
    }
}

pub fn guess_mime_type(path: &str) -> MimeType {
    path.split('.')
        .last()
        .map(FileExtension::from_str)
        .unwrap_or(FileExtension::Unknown)
        .mime_type()
}
