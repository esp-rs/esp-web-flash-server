use ::rocket::async_main;
use anyhow::Result;
use std::{path::PathBuf, time::Duration};

use clap::Parser;
use espflash::{elf::FirmwareImageBuilder, Chip, FlashSize, PartitionTable};
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
    <script
        type=\"module\"
        src=\"https://unpkg.com/esp-web-tools@8.0.2/dist/web/install-button.js?module\">
    </script>
    <body>
    
    <esp-web-install-button
        manifest=\"/manifest.json\">
    </esp-web-install-button>

    </body>
    </html>
    ",
    )
}

#[get("/manifest.json")]
fn manifest(data: &State<PartsData>) -> content::RawJson<String> {
    content::RawJson(format!(
        r#"
{{
    "name": "Something",
    "version": "1",
    "home_assistant_domain": "esphome",
    "funding_url": "https://esphome.io/guides/supporters.html",
    "new_install_prompt_erase": false,
    "builds": [
      {{
        "chipFamily": "{}",
        "parts": [
          {{ "path": "bootloader.bin", "offset": 4096 }},
          {{ "path": "partitions.bin", "offset": 32768 }},
          {{ "path": "firmware.bin", "offset": 65536 }}
        ]
      }}
    ]
  }}
"#,
        data.chip
    ))
}

struct PartsData {
    chip: String,
    bootloader: Vec<u8>,
    partitions: Vec<u8>,
    firmware: Vec<u8>,
}

fn prepare() -> Result<PartsData> {
    let opts = Args::parse();

    let elf = std::fs::read(opts.elf)?;

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

    let firmware = FirmwareImageBuilder::new(&elf)
        .flash_size(Some(FlashSize::Flash4Mb)) // TODO make configurable
        .build()?;

    let chip = opts.chip;
    let chip_name = match chip {
        Chip::Esp32 => "ESP32",
        Chip::Esp32c3 => "ESP32-C3",
        Chip::Esp32s2 => "ESP32-S2",
        Chip::Esp32s3 => "ESP32-S3",
        Chip::Esp8266 => "ESP8266",
    };

    let image = chip.get_flash_image(&firmware, b, p, None, None)?;
    let parts: Vec<_> = image.flash_segments().collect();
    let bootloader = &parts[0];
    let partitions = &parts[1];
    let app = &parts[2];

    Ok(PartsData {
        chip: chip_name.to_string(),
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
