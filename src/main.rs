use ::rocket::async_main;
use anyhow::Result;
use std::{path::PathBuf, time::Duration};

use clap::Parser;
use esp_idf_part::PartitionTable;
use espflash::{elf::ElfFirmwareImage, flasher::FlashSize::Flash4Mb, targets::Chip};
use rocket::{response::content, State};

#[macro_use]
extern crate rocket;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// chip name
    #[clap(short, long)]
    chip: Chip,

    /// path to bootloader
    #[clap(short, long)]
    bootloader: Option<PathBuf>,

    /// path to partition table csv
    #[clap(short, long)]
    partition_table: Option<PathBuf>,

    elf: PathBuf,
}

#[get("/bootloader.bin")]
fn bootloader(data: &State<PartsData>) -> Vec<u8> {
    data.bootloader.clone()
}

#[get("/partitions.bin")]
fn partitions(data: &State<PartsData>) -> Vec<u8> {
    data.partitions.clone()
}

#[get("/firmware.bin")]
fn firmware(data: &State<PartsData>) -> Vec<u8> {
    data.firmware.clone()
}

#[get("/")]
fn index() -> content::RawHtml<&'static str> {
    content::RawHtml(
        "
        <html>
        <head>
            <style>
                body {
                    background-color: #cdcdcd;
                    font-family: Arial, Helvetica, sans-serif;
                    color: #343434;
                }
                
                h1 {
                    margin-left: 40px;
                }
                </style>        
        </head>
        <body>
            <center>
                <h1>ESP Web Flasher</h1>

                <div id=\"main\" style=\"display: none;\">

                    <br>
                    <script type=\"module\" src=\"https://unpkg.com/esp-web-tools@8.0.2/dist/web/install-button.js?module\">
                    </script>
                    <esp-web-install-button id=\"installButton\" manifest=\"manifest.json\"></esp-web-install-button>
                    <br>
                    <br>
                    <br>
                    <span><i>NOTE: Make sure to close anything using your device's com port (e.g. serial monitor)</i></span>
                </div>
                <div id=\"notSupported\" style=\"display: none;\">
                    Your browser does not support the Web Serial API. Try Chrome
                </div>
            </center>

            <script>
                if (navigator.serial) {
                    document.getElementById(\"notSupported\").style.display = 'none';
                    document.getElementById(\"main\").style.display = 'block';
                } else {
                    document.getElementById(\"notSupported\").style.display = 'block';
                    document.getElementById(\"main\").style.display = 'none';
                }
            </script>

        </body>
        </html>
        ",
    )
}

#[get("/manifest.json")]
fn manifest() -> content::RawJson<&'static str> {
    content::RawJson(
        r#"
        {
            "name": "ESP Application",
            "new_install_prompt_erase": true,
            "builds": [
                {
                "chipFamily": "ESP32",
                "parts": [
                    {
                    "path": "bootloader.bin",
                    "offset": 4096
                    },
                    {
                    "path": "partitions.bin",
                    "offset": 32768
                    },
                    {
                    "path": "firmware.bin",
                    "offset": 65536
                    }
                ]
                },
                {
                "chipFamily": "ESP32-C3",
                "parts": [
                    {
                    "path": "bootloader.bin",
                    "offset": 0
                    },
                    {
                    "path": "partitions.bin",
                    "offset": 32768
                    },
                    {
                    "path": "firmware.bin",
                    "offset": 65536
                    }
                ]
                },
                {
                    "chipFamily": "ESP32-C2",
                    "parts": [
                        {
                        "path": "bootloader.bin",
                        "offset": 0
                        },
                        {
                        "path": "partitions.bin",
                        "offset": 32768
                        },
                        {
                        "path": "firmware.bin",
                        "offset": 65536
                        }
                    ]
                    },
                {
                "chipFamily": "ESP32-S2",
                "parts": [
                    {
                    "path": "bootloader.bin",
                    "offset": 4096
                    },
                    {
                    "path": "partitions.bin",
                    "offset": 32768
                    },
                    {
                    "path": "firmware.bin",
                    "offset": 65536
                    }
                ]
                },
                {
                "chipFamily": "ESP32-S3",
                "parts": [
                    {
                    "path": "bootloader.bin",
                    "offset": 0
                    },
                    {
                    "path": "partitions.bin",
                    "offset": 32768
                    },
                    {
                    "path": "firmware.bin",
                    "offset": 65536
                    }
                ]
                }
            ]
        }
        "#,
    )
}

struct PartsData {
    _chip: String,
    bootloader: Vec<u8>,
    partitions: Vec<u8>,
    firmware: Vec<u8>,
}

fn prepare() -> Result<PartsData> {
    let opts = Args::parse();

    let elf = std::fs::read(opts.elf)?;
    let elf = xmas_elf::ElfFile::new(&elf).expect("Invalid elf file");

    let p = if let Some(p) = &opts.partition_table {
        Some(PartitionTable::try_from_bytes(std::fs::read(p)?)?)
    } else {
        None
    };

    let b = if let Some(p) = &opts.bootloader {
        Some(std::fs::read(p)?)
    } else {
        None
    };

    let firmware = ElfFirmwareImage::new(elf);

    let chip = opts.chip;
    let chip_name = match chip {
        Chip::Esp32 => "ESP32",
        Chip::Esp32c3 => "ESP32-C3",
        Chip::Esp32s2 => "ESP32-S2",
        Chip::Esp32s3 => "ESP32-S3",
        Chip::Esp8266 => "ESP8266",
        Chip::Esp32c2 => "ESP32-C2",
    };

    let image = chip.into_target().get_flash_image(
        &firmware,
        b,
        p,
        None,
        None,
        None,
        Some(Flash4Mb),
        None,
    )?;
    let parts: Vec<_> = image.flash_segments().collect();
    let bootloader = &parts[0];
    let partitions = &parts[1];
    let app = &parts[2];

    Ok(PartsData {
        _chip: chip_name.to_string(),
        bootloader: bootloader.data.to_vec(),
        partitions: partitions.data.to_vec(),
        firmware: app.data.to_vec(),
    })
}

fn main() -> Result<()> {
    let data = prepare()?;

    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(1000));
        opener::open_browser("http://127.0.0.1:8000/").ok();
    });

    async_main(async move {
        let _res = rocket::build()
            .mount(
                "/",
                routes![index, manifest, bootloader, partitions, firmware],
            )
            .manage(data)
            .launch()
            .await
            .expect("Problem launching server");
    });

    Ok(())
}
