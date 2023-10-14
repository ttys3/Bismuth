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

    /// Call custom wallpaper command
    #[clap(long, short)]
    pub custom_command: Option<Vec<String>>,

    /// Backup dir to copy daily wallpaper to
    #[clap(long, short)]
    pub backup_dir: Option<String>,

    /// Specify the market to use for the wallpaper
    /// see https://github.com/neffo/bing-wallpaper-gnome-extension/blob/64d516aaf17fda563e4dd2f856e6fa6fa5edc176/extension.js#L628
    #[clap(long, short)]
    pub market: Option<String>,

    /// Specify the resolution to use for the wallpaper
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
