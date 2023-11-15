# Bismuth

> [**Bismuth**](https://en.wikipedia.org/wiki/Bismuth) is a chemical element with the symbol **Bi** and atomic number 83.

> **ビスマス**（英語: bismuth `[ˈbɪzməθ]`）あるいは**蒼鉛**（そうえん）は、原子番号83の元素。元素記号は **Bi**（ラテン語: Bismuthumから）。第15族元素の一つ。

![Bismuth](https://upload.wikimedia.org/wikipedia/commons/thumb/e/ef/Bismuth_crystals_and_1cm3_cube.jpg/500px-Bismuth_crystals_and_1cm3_cube.jpg)


## Table of Contents

- [About](#about)
- [Dependencies](#dependencies)
  - [Arch](#for-arch)
  - [Debian, Ubuntu and Mint](#for-debian-ubuntu-and-mint)
- [Installation](#installation)
- [Usage](#usage)
  - [Commands](#commands)
  - [silent](#silent)
  - [mode](#mode-mode)
  - [custom command](#custom-command-custom_command)
- [To-Do](#to-do)

## About
Bismuth is a lightweight Rust script that sets your desktop wallpaper to the latest daily Bing image. 

## Dependencies

|Dependency|Link                                              |
|----------|--------------------------------------------------|
|feh       |[Github](https://github.com/derf/feh)             |


### For Fedora Linux

```shell
dnf install feh
```

### For Arch
```
paru -S feh
```
```
yay -S feh
```
```
sudo pacman -S feh
```

### For Debian, Ubuntu and Mint
```
sudo apt install feh
```

## Installation

1\. Clone the repository and cd into it.
```
git clone "https://github.com/ttys3/Bismuth"
cd Bismuth
```
2\. Build Bismuth
```
cargo build --release
```

## Usage
Here's an example usage of the script.
```
bismuth --silent --mode max
```

---

Or you can also simply do;

Which calls `feh` using `--bg-fill` option as default.
```
bismuth
```

---

Here's an example of a custom command, in this case using `swaybg`.

`%` represents the file destination.
```
bismuth -c 'swaybg --image %'
```

The wallpaper gets saved at `$HOME/.local/share/.wallpaper.jpg`.

### Commands
| Command                                     | Description                                                              |
|---------------------------------------------|--------------------------------------------------------------------------|
| `--silent`, `-s`                            | Disable notifications.                                                   |
| `--mode`, `-m` `<MODE>`                     | Set feh scaling options.                                                 |
| `--custom-command`, `-c` `<CUSTOM_COMMAND>` | Set background using custom command, this flag can repeat multiple times |
| `--help`, `-h`                              | Display help information.                                                |
| `--version`, `-V`                           | Show the version of the script.                                          |
| `-b, --backup-dir <BACKUP_DIR>`             | Backup dir to copy daily wallpaper to                                    |
| `--mkt <MARKET>`                            | Specify the market to use for the wallpaper                              |
| `-r, --resolution <RESOLUTION>`             | Specify the resolution to use for the wallpaper.                         |

### `--help`
```
Usage: bismuth [OPTIONS]

Options:
  -s, --silent
          Disables notifications
  -m, --mode <MODE>
          Specifies the scaling options for Feh [default: fill] [possible values: center, fill, max, scale, tile]
  -c, --custom-command <CUSTOM_COMMAND>
          Call custom wallpaper command, this flag can repeat multiple times
  -b, --backup-dir <BACKUP_DIR>
          Backup dir to copy daily wallpaper to
      --mkt <MARKET>
          Specify the market to use for the wallpaper. Available mkt: auto, ar-XA, da-DK, de-AT, de-CH, de-DE, en-AU, en-CA, en-GB, en-ID, en-IE, en-IN, en-MY, en-NZ, en-PH, en-SG, en-US, en-WW, en-XA, en-ZA, es-AR, es-CL, es-ES, es-MX, es-US, es-XL, et-EE, fi-FI, fr-BE, fr-CA, fr-CH, fr-FR, he-IL, hr-HR, hu-HU, it-IT, ja-JP, ko-KR, lt-LT, lv-LV, nb-NO, nl-BE, nl-NL, pl-PL, pt-BR, pt-PT, ro-RO, ru-RU, sk-SK, sl-SL, sv-SE, th-TH, tr-TR, uk-UA, zh-CN, zh-HK, zh-TW
  -r, --resolution <RESOLUTION>
          Specify the resolution to use for the wallpaper. Available resolution: auto UHD 1920x1200 1920x1080 1366x768 1280x720 1024x768 800x600
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

### `--silent`
Disables notifications when the wallpaper is successfully set.

### `--mode <MODE>`
- `center`: Centers the image on the screen without scaling.
- `fill`: Scales the image to fit the screen and preserves aspect ratio.
- `max`: Scales the image to the maximum size with black borders on one side.
- `scale`: Fills the screen, but doesn't preserve the aspect ratio.
- `tile`: Tiles the image on the screen.

### `--custom-command <CUSTOM_COMMAND>`
Sets wallpaper using a custom command.

Example `bismuth -c "swaybg --image %"` 

The `%` symbol is important as it signifies the file destination.

### To-Do
- [x] Save image as `.wallpaper.jpg` for `.fehbg`.
- [x] Custom command support.

### credits

this repo is originally forked from [thejayduck/Bismuth](https://github.com/thejayduck/Bismuth)