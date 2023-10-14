use args::Arguments;
use clap::Parser;
use directories::BaseDirs;
use inflector::{self, Inflector};
use notify_rust::Notification;
use std::{path::PathBuf, process::Command};
use std::path::Path;
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;
use serde::{Deserialize, Serialize};

mod args;
mod errors;

const ICON: &str = "image-jpeg";
const NAME: &str = env!("CARGO_PKG_NAME");

const DEFAULT_RESOLUTION: &str = "UHD";
const DEFAULT_MARKET: &str = "en-US";

// RESOLUTIONS and MARKETS ref https://github.com/neffo/bing-wallpaper-gnome-extension/blob/64d516aaf17fda563e4dd2f856e6fa6fa5edc176/utils.js#L42
const RESOLUTIONS: [&str; 8]  = ["auto", "UHD", "1920x1200", "1920x1080", "1366x768", "1280x720", "1024x768", "800x600"];
const MARKETS: [&str; 57]  = ["auto", "ar-XA", "da-DK", "de-AT", "de-CH", "de-DE", "en-AU", "en-CA", "en-GB",
"en-ID", "en-IE", "en-IN", "en-MY", "en-NZ", "en-PH", "en-SG", "en-US", "en-WW", "en-XA", "en-ZA", "es-AR",
"es-CL", "es-ES", "es-MX", "es-US", "es-XL", "et-EE", "fi-FI", "fr-BE", "fr-CA", "fr-CH", "fr-FR",
"he-IL", "hr-HR", "hu-HU", "it-IT", "ja-JP", "ko-KR", "lt-LT", "lv-LV", "nb-NO", "nl-BE", "nl-NL",
"pl-PL", "pt-BR", "pt-PT", "ro-RO", "ru-RU", "sk-SK", "sl-SL", "sv-SE", "th-TH", "tr-TR", "uk-UA",
"zh-CN", "zh-HK", "zh-TW"];

#[derive(Serialize, Deserialize, Debug)]
struct ImageObject {
    pub title: String,
    pub url: String,
    pub urlbase: String,
    pub startdate: String,
    pub enddate: String,
    pub resolution: Option<String>,
    pub wp: bool,
}

/// sample response
/// Object {"images": Array [
/// Object {"bot": Number(1), "copyright": String("Vieste on the Gargano peninsula, Apulia, Italy (© Pilat666/Getty Images)"),
/// "copyrightlink": String("https://www.bing.com/search?q=Vieste&form=hpcapt&filters=HpDate%3a%2220231013_0700%22"),
/// "drk": Number(1), "enddate": String("20231014"), "fullstartdate": String("202310130700"),
/// "hs": Array [], "hsh": String("c01dedec87d8be3c1b1332686c8fe269"),
/// "quiz": String("/search?q=Bing+homepage+quiz&filters=WQOskey:%22HPQuiz_20231013_ViesteItaly%22&FORM=HPQUIZ"), "startdate": String("20231013"),
/// "title": String("Life on the edge"),
/// "top": Number(1), "url": String("/th?id=OHR.ViesteItaly_EN-US0948108910_1920x1080.jpg&rf=LaDigue_1920x1080.jpg&pid=hp"),
/// "urlbase": String("/th?id=OHR.ViesteItaly_EN-US0948108910"),
/// "wp": Bool(true)}], "market": Object {"mkt": String("en-US")}, "tooltips": Object {"loading": String("Loading..."),
/// "next": String("Next image"), "previous": String("Previous image"), "walle": String("This image is not available to download as wallpaper."),
/// "walls": String("Download this image. Use of this image is restricted to wallpaper only.")}}
#[derive(Serialize, Deserialize, Debug)]
struct Response {
    pub images: Option<Vec<ImageObject>>,
}

impl ImageObject {
    /// get save filename use the same logic as neffo/bing-wallpaper-gnome-extension
    // https://github.com/neffo/bing-wallpaper-gnome-extension/blob/64d516aaf17fda563e4dd2f856e6fa6fa5edc176/extension.js#L812C10-L812C10
    // this.imageURL = BingURL + image.urlbase + '_' + resolution + '.jpg'; // generate image url for user's resolution
    // this.filename = toFilename(BingWallpaperDir, image.startdate, image.urlbase, resolution);
    /// function toFilename(wallpaperDir, startdate, imageURL, resolution) {
    //     return wallpaperDir + startdate + '-' + imageURL.replace(/^.*[\\\/]/, '').replace('th?id=OHR.', '') + '_' + resolution + '.jpg';
    // }
    pub fn get_save_filename(&self) -> anyhow::Result<String> {
        // url demo: /th?id=OHR.ViesteItaly_EN-US0948108910_1920x1080.jpg&rf=LaDigue_1920x1080.jpg&pid=hp
        // urlbase demo: /th?id=OHR.ViesteItaly_EN-US0948108910
        if let Some(rs) = std::string::String::from(self.urlbase.clone()).rsplit_once("/") {
            if rs.1.starts_with("th?id=OHR.") {
                let mut cleaned_filename = rs.1.strip_prefix("th?id=OHR.").unwrap();
                let cleaned_filename = cleaned_filename.replace("..", "_");
                return Ok(format!("{}-{}_{}.jpg", self.startdate, cleaned_filename, self.resolution.clone().unwrap_or(DEFAULT_RESOLUTION.to_string())));
            }
        }
        Err(anyhow::format_err!("can not parse urlbase {}  to filename", self.urlbase.clone()))
    }

    pub fn get_download_url(&self, resolution: &str) -> String {
        let resolution = if RESOLUTIONS.contains(&resolution) {
            resolution
        } else {
            println!("resolution {} not in {:?}, use default {}", resolution, RESOLUTIONS, DEFAULT_RESOLUTION);
            DEFAULT_RESOLUTION
        };
        format!("https://bing.com{}_{}.jpg", self.urlbase, resolution)
    }
}

fn get_api_url(mkt: &str) -> String {
    let mkt = if MARKETS.contains(&mkt) {
        mkt
    } else {
        println!("market {} not in {:?}, use default {}", mkt, MARKETS, DEFAULT_MARKET);
        DEFAULT_MARKET
    };
    format!("https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mbl=1&mkt={}", mkt)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();
    let mode = mode(args.mode);

    let api_url = get_api_url(&args.market.as_ref().map_or(DEFAULT_MARKET.to_owned(), |x|x.to_owned()));

    println!("begin request api {} ...", &api_url);

    // https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.json
    let client = reqwest::Client::new();
    let response: Response = client
        .get(api_url.clone())
        .send()
        .await
        .map_err(|err| errors::Error::Domain(api_url.to_owned() + ", err: " + &err.to_string()))?
        .json::<Response>()
        .await
        .map_err(|err| errors::Error::ImageRequest(api_url.to_owned() + ", err: " +  &err.to_string()))?;

    println!("response: {:?}", &response);

    // temporary value is freed at the end of this statement
    // let image = response.images.unwrap().first().unwrap();
    let binding = response.images.unwrap();
    let image = binding.first().unwrap();

    println!("image: {:?}", image);

    let destination = save_image(&image, args.backup_dir).await?;

    if let Some(custom_command) = args.custom_command {
        let command_args = custom_command.iter().map(|arg|arg.replace("%", &destination.to_string_lossy().into_owned()));
        println!("command_args: {:?}", command_args.clone().collect::<Vec<_>>());
        // loop command_args and exec sh -c command
        for arg in command_args {
            println!("begin exec command: {:?}", arg);
            Command::new("sh").arg("-c").arg(arg).spawn()?;
        }
    } else {
        Command::new("feh").arg(mode).arg(&destination).spawn()?;
    };

    if !args.silent {
        send_notification(
            &NAME.to_pascal_case(),
            &format!("Wallpaper successfully Set.\nTitle: {0}", image.title),
            ICON,
        )?;
    }

    Ok(())
}

async fn save_image(image: &ImageObject, backup_dir: Option<String>) -> anyhow::Result<PathBuf> {
    let base_dirs = BaseDirs::new().ok_or_else(||anyhow::format_err!("BaseDirs::new() failed"))?;
    //return Err(errors::Error::Directory.into());
    let image_url = image.get_download_url(DEFAULT_RESOLUTION);
    println!("Downloading {} ...", image_url);
    let response = reqwest::get(image_url).await?;

    let save_path = if let Some(backup_dir) = backup_dir {
        let home_dir = base_dirs.home_dir();
        let backup_dir = backup_dir.replace("~", home_dir.to_str().ok_or_else(||anyhow::format_err!("home_dir to_str failed"))?);
        let mut full_path = PathBuf::from(backup_dir);
        let dir_path = Path::new(&full_path);
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path)?
        }
        let file_name = PathBuf::from(image.get_save_filename()?);
        full_path.push(file_name);
        full_path
    } else {
        let mut full_path = base_dirs.data_local_dir().to_path_buf();
        let file_name = PathBuf::from(".wallpaper.jpg");

        full_path.push(file_name);
        full_path
    };

    println!("filepath: {:?}", &save_path);

    let mut file = tokio::fs::File::create(&save_path).await?;

    let content = response.bytes_stream().map(|result| {
        result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
    });

    tokio::io::copy(&mut StreamReader::new(content), &mut file).await?;

    Ok(save_path)
}

fn send_notification(summary: &str, body: &str, icon: &str) -> anyhow::Result<()> {
    Notification::new()
        .summary(summary)
        .body(body)
        .icon(icon)
        .show()?;

    Ok(())
}

fn mode(value: args::Modes) -> &'static str {
    match value {
        args::Modes::Center => "--bg-center",
        args::Modes::Fill => "--bg-fill",
        args::Modes::Max => "--bg-max",
        args::Modes::Scale => "--bg-scale",
        args::Modes::Tile => "--bg-tile",
    }
}
