use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Arguments {
    /// Disables notifications.
    #[clap(long, short, action)]
    pub silent: bool,

    /// Specifies the scaling options for Feh.
    ///
    /// Available modes:
    ///  - `Center`: Centers the image on the screen without scaling.
    ///  - `Fill`: Scales the image to fit the screen and preserves aspect ratio.
    ///  - `Max`: Scales the image to the maximum size with black borders on one side.
    ///  - `Scale`: Fills the screen but doesn't preserve the aspect raio.
    ///  - `Tile`: Tiles the image on the screen.
    #[clap(long, short, value_enum, default_value_t = Modes::Fill)]
    pub mode: Modes,

    /// Call custom wallpaper command, this flag can repeat multiple times
    #[clap(long, short)]
    pub custom_command: Option<Vec<String>>,

    /// Backup dir to copy daily wallpaper to
    #[clap(long, short)]
    pub backup_dir: Option<String>,

    /// Specify the market to use for the wallpaper.
    /// Available mkt:
    ///     auto, ar-XA, da-DK, de-AT, de-CH, de-DE, en-AU, en-CA, en-GB, en-ID,
    ///     en-IE, en-IN, en-MY, en-NZ, en-PH, en-SG, en-US, en-WW, en-XA, en-ZA,
    ///     es-AR, es-CL, es-ES, es-MX, es-US, es-XL, et-EE, fi-FI, fr-BE, fr-CA,
    ///     fr-CH, fr-FR, he-IL, hr-HR, hu-HU, it-IT, ja-JP, ko-KR, lt-LT, lv-LV,
    ///     nb-NO, nl-BE, nl-NL, pl-PL, pt-BR, pt-PT, ro-RO, ru-RU, sk-SK, sl-SL,
    ///     sv-SE, th-TH, tr-TR, uk-UA, zh-CN, zh-HK, zh-TW
    #[clap(long = "mkt")]
    pub market: Option<String>,

    /// Specify the resolution to use for the wallpaper.
    /// Available resolution:
    ///     auto
    ///     UHD
    ///     1920x1200
    ///     1920x1080
    ///     1366x768
    ///     1280x720
    ///     1024x768
    ///     800x600
    #[clap(long, short)]
    pub resolution: Option<String>,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum Modes {
    Center,
    Fill,
    Max,
    Scale,
    Tile,
}
