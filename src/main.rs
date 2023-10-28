use anyhow::Context;
use args::Arguments;
use chrono::TimeZone;
use clap::Parser;
use directories::BaseDirs;
use inflector::{self, Inflector};
use log::{debug, info};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{path::PathBuf, process::Command};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;

mod args;
mod errors;

const ICON: &str = "image-jpeg";
const NAME: &str = env!("CARGO_PKG_NAME");

const DEFAULT_RESOLUTION: &str = "UHD";
const DEFAULT_MARKET: &str = "en-US";

// RESOLUTIONS and MARKETS ref https://github.com/neffo/bing-wallpaper-gnome-extension/blob/64d516aaf17fda563e4dd2f856e6fa6fa5edc176/utils.js#L42
const RESOLUTIONS: [&str; 8] = [
    "auto",
    "UHD",
    "1920x1200",
    "1920x1080",
    "1366x768",
    "1280x720",
    "1024x768",
    "800x600",
];
const MARKETS: [&str; 57] = [
    "auto", "ar-XA", "da-DK", "de-AT", "de-CH", "de-DE", "en-AU", "en-CA", "en-GB", "en-ID",
    "en-IE", "en-IN", "en-MY", "en-NZ", "en-PH", "en-SG", "en-US", "en-WW", "en-XA", "en-ZA",
    "es-AR", "es-CL", "es-ES", "es-MX", "es-US", "es-XL", "et-EE", "fi-FI", "fr-BE", "fr-CA",
    "fr-CH", "fr-FR", "he-IL", "hr-HR", "hu-HU", "it-IT", "ja-JP", "ko-KR", "lt-LT", "lv-LV",
    "nb-NO", "nl-BE", "nl-NL", "pl-PL", "pt-BR", "pt-PT", "ro-RO", "ru-RU", "sk-SK", "sl-SL",
    "sv-SE", "th-TH", "tr-TR", "uk-UA", "zh-CN", "zh-HK", "zh-TW",
];

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ImageObject {
    pub startdate: String,     // example: 20231013
    pub fullstartdate: String, // example: 202310131500
    pub enddate: String,       // example: 20231014
    pub url: String, // example: /th?id=OHR.RailwayDay2023_JA-JP6915793143_1920x1080.jpg&rf=LaDigue_1920x1080.jpg&pid=hp
    pub urlbase: String, // example: /th?id=OHR.RailwayDay2023_JA-JP6915793143
    pub copyright: String, // example: 第三只見川橋梁を渡る列車, 福島県 大沼郡 三島町 (© DoctorEgg/Getty Images)
    pub copyrightlink: String, // example: https://www.bing.com/search?q=%E5%8F%AA%E8%A6%8B%E7%B7%9A&form=hpcapt&filters=HpDate%3a%2220231013_1500%22
    pub title: String,         // example: 今日は鉄道の日
    pub quiz: String, // example: /search?q=Bing+homepage+quiz&filters=WQOskey:%22HPQuiz_20231013_RailwayDay2023%22&FORM=HPQUIZ
    pub wp: bool,
    pub hsh: String, // example 693bc6e04e2867a01a8cbf5c2acfc44c
    pub drk: i64,    // example: 1
    pub top: i64,    // example: 1
    pub bot: i64,    // example: 1
    pub hs: Vec<serde_json::Value>,

    // for internal usage
    pub resolution: Option<String>,
    pub market: Option<String>,
    pub file_hash : Option<String>, // why not just use `hsh` field? because when we download, we use different resolution, so the file hash will be different
}

/// sample response
/// run `curl -Ss "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US" | jq` to get sample result
/// ```json
/// {
///   "market": {
///     "mkt": "ja-JP"
///   },
///   "images": [
///     {
///       "startdate": "20231013",
///       "fullstartdate": "202310131500",
///       "enddate": "20231014",
///       "url": "/th?id=OHR.RailwayDay2023_JA-JP6915793143_1920x1080.jpg&rf=LaDigue_1920x1080.jpg&pid=hp",
///       "urlbase": "/th?id=OHR.RailwayDay2023_JA-JP6915793143",
///       "copyright": "第三只見川橋梁を渡る列車, 福島県 大沼郡 三島町 (© DoctorEgg/Getty Images)",
///       "copyrightlink": "https://www.bing.com/search?q=%E5%8F%AA%E8%A6%8B%E7%B7%9A&form=hpcapt&filters=HpDate%3a%2220231013_1500%22",
///       "title": "今日は鉄道の日",
///       "quiz": "/search?q=Bing+homepage+quiz&filters=WQOskey:%22HPQuiz_20231013_RailwayDay2023%22&FORM=HPQUIZ",
///       "wp": true,
///       "hsh": "693bc6e04e2867a01a8cbf5c2acfc44c",
///       "drk": 1,
///       "top": 1,
///       "bot": 1,
///       "hs": []
///     }
///   ],
///   "tooltips": {
///     "loading": "読み込み中...",
///     "previous": "前の画像へ",
///     "next": "次の画像へ",
///     "walle": "この画像を壁紙としてダウンロードすることはできません。",
///     "walls": "この画像をダウンロードできます。画像の用途は壁紙に限定されています。"
///   }
/// }
/// ```
///
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
        if let Some(rs) = self.urlbase.clone().rsplit_once('/') {
            if rs.1.starts_with("th?id=OHR.") {
                let cleaned_filename = rs.1.strip_prefix("th?id=OHR.").unwrap();
                let cleaned_filename = cleaned_filename.replace("..", "_");
                return Ok(format!(
                    "{}-{}_{}.jpg",
                    self.startdate,
                    cleaned_filename,
                    self.resolution
                        .clone()
                        .unwrap_or(DEFAULT_RESOLUTION.to_string())
                ));
            }
        }
        Err(anyhow::format_err!(
            "can not parse urlbase {}  to filename",
            self.urlbase.clone()
        ))
    }

    pub fn get_download_url(&self, resolution: &str) -> String {
        let resolution = if RESOLUTIONS.contains(&resolution) {
            resolution
        } else {
            println!(
                "resolution {} not in {:?}, use default {}",
                resolution, RESOLUTIONS, DEFAULT_RESOLUTION
            );
            DEFAULT_RESOLUTION
        };
        format!("https://bing.com{}_{}.jpg", self.urlbase, resolution)
    }
}

fn get_api_url(mkt: &str) -> String {
    let mkt = if MARKETS.contains(&mkt) {
        mkt
    } else {
        println!(
            "market {} not in {:?}, use default {}",
            mkt, MARKETS, DEFAULT_MARKET
        );
        DEFAULT_MARKET
    };
    format!(
        "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mbl=1&mkt={}",
        mkt
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // set default log level https://github.com/rust-cli/env_logger/issues/47#issuecomment-607475404
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let args = Arguments::parse();
    let mode = mode(args.mode);

    if let Ok(image) = get_cached_api_data().await {
        debug!("read cached api data success: {:?}", image);
        // check if image file match our need
        // generate date format like: 20231013
        let actual_date = chrono::Local::now().format("%Y%m%d").to_string();

        // parse full start date like: 202310131500
        let full_start_date =
            chrono::NaiveDateTime::parse_from_str(&image.fullstartdate, "%Y%m%d%H%M")
                .context("parse fullstartdate failed")?;
        debug!("full_start_date: {}", full_start_date);

        // Converts the local NaiveDateTime to the timezone-aware DateTime if possible.
        let full_start_date = chrono::Local.from_local_datetime(&full_start_date).unwrap();
        debug!("full_start_date from_local_datetime: {:?}", full_start_date);
        let full_start_date = full_start_date + chrono::Duration::hours(24);
        let now_date = chrono::Local::now();
        debug!(
            "full_start_date +24h: {}, now: {}",
            full_start_date, now_date
        );
        // let date_ok = actual_date == image.enddate || actual_date == image.startdate;
        // check if: current time > full_start_date + 24 hours
        let date_ok = if now_date < full_start_date {
            debug!("cached api data is not expired, do nothing. image: {:?}, {} (full_start_date) < {} (now_date)", image, full_start_date, now_date);
            true
        } else {
            info!("cached api data is expired, continue to request api. image: {:?}, full_start_date: {}", image, full_start_date);
            false
        };
        if args.resolution == image.resolution && args.market == image.market && date_ok {
            info!(
                "cached api data match our need, do nothing. image: {:?}",
                image
            );
            return Ok(());
        } else {
            info!("cached api data not match our need, continue to request api. image: {:?}, args.resolution: {:?}, args.market: {:?}, actual_date: {}",
                     image, args.resolution, args.market, actual_date);
        }
    } else {
        debug!("no cached api data, continue to request api");
    }

    let api_url = get_api_url(
        &args
            .market
            .as_ref()
            .map_or(DEFAULT_MARKET.to_owned(), |x| x.to_owned()),
    );

    debug!("begin request api {} ...", &api_url);

    // https://docs.rs/reqwest/latest/reqwest/struct.Response.html#method.json
    let client = reqwest::Client::new();
    let body = client
        .get(api_url.clone())
        .send()
        .await
        .map_err(|err| errors::Error::Domain(api_url.to_owned() + ", err: " + &err.to_string()))?
        .text()
        .await
        .map_err(|err| {
            errors::Error::ImageRequest(api_url.to_owned() + ", err: " + &err.to_string())
        })?;

    debug!("response body: {:?}", &body);

    let response: Response = serde_json::from_str(&body).context("parse response json failed")?;

    // temporary value is freed at the end of this statement
    // let image = response.images.unwrap().first().unwrap();
    let binding = response.images.unwrap();
    let mut image: ImageObject = binding.first().cloned().unwrap();

    debug!("image: {:?}", image);

    image.resolution = args.resolution.clone();
    image.market = args.market.clone();
    save_cached_api_data(&image).await?;

    let destination = save_image(&image, args.backup_dir).await?;
    if destination.is_none() {
        info!("image file exists, do nothing");
        return Ok(());
    }

    let destination = destination.unwrap();

    if let Some(custom_command) = args.custom_command {
        let command_args = custom_command
            .iter()
            .map(|arg| arg.replace('%', &destination.to_string_lossy()));
        debug!(
            "command_args: {:?}",
            command_args.clone().collect::<Vec<_>>()
        );
        // loop command_args and exec sh -c command
        for arg in command_args {
            info!("begin exec command: {:?}", arg);
            Command::new("sh").arg("-c").arg(arg).spawn()?;
        }
    } else {
        Command::new("feh").arg(mode).arg(&destination).spawn().map_err(|e|errors::Error::Feh(e.to_string()))?;
    };

    if !args.silent {
        send_notification(
            &NAME.to_pascal_case(),
            &format!("Wallpaper successfully Set.\nTitle: {0}", image.title),
            ICON,
            destination.to_str().unwrap(),
        )?;
    }

    Ok(())
}

async fn save_image(image: &ImageObject, backup_dir: Option<String>) -> anyhow::Result<Option<PathBuf>> {
    let base_dirs = BaseDirs::new().ok_or(errors::Error::Directory)?;
    //return Err(errors::Error::Directory.into());
    let image_url = image.get_download_url(DEFAULT_RESOLUTION);
    info!("Downloading {} ...", image_url);
    let response = reqwest::get(image_url).await?;

    let save_path = if let Some(backup_dir) = backup_dir {
        let home_dir = base_dirs.home_dir();
        let backup_dir = backup_dir.replace(
            '~',
            home_dir
                .to_str()
                .ok_or_else(|| anyhow::format_err!("home_dir to_str failed"))?,
        );
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

    info!("filepath: {:?}", &save_path);

    if let Ok(metadata) = std::fs::metadata(&save_path) {
        if metadata.is_file() {
            info!("file exists, skip download: {:?}", &save_path);
            return Ok(None);
        }
    }

    let mut file = tokio::fs::File::create(&save_path).await?;

    let content = response
        .bytes_stream()
        .map(|result| result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)));

    tokio::io::copy(&mut StreamReader::new(content), &mut file).await?;

    Ok(Some(save_path))
}

async fn save_cached_api_data(response: &ImageObject) -> anyhow::Result<()> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::format_err!("BaseDirs::new() failed"))?;
    let mut full_path = base_dirs.data_local_dir().to_path_buf();
    let file_name = PathBuf::from(".wallpaper.json");

    full_path.push(file_name);

    let mut file = tokio::fs::File::create(&full_path).await?;

    let content = serde_json::to_string(response)?;

    file.write_all(content.as_bytes()).await?;

    Ok(())
}

async fn get_cached_api_data() -> anyhow::Result<ImageObject> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::format_err!("BaseDirs::new() failed"))?;
    let mut full_path = base_dirs.data_local_dir().to_path_buf();
    let file_name = PathBuf::from(".wallpaper.json");

    full_path.push(file_name);

    let file = tokio::fs::File::open(&full_path).await?;
    let mut reader = tokio::io::BufReader::new(file);

    let mut contents = String::new();
    reader.read_to_string(&mut contents).await?;

    let image: ImageObject = serde_json::from_str(&contents)?;

    Ok(image)
}

fn send_notification(
    summary: &str,
    body: &str,
    icon: &str,
    image_path: &str,
) -> anyhow::Result<()> {
    Notification::new()
        .summary(summary)
        .body(body)
        .image_path(image_path)
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
