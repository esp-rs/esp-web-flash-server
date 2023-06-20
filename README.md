# web-flash

Starts a local server serving a web page to flash a given ELF file.

## Installation
```
cargo install espup
```
## Usage
```
Usage: web-flash [OPTIONS] --chip <CHIP> <ELF>

Arguments:
  <ELF>
          Path to the ELF file

Options:
  -c, --chip <CHIP>
          Chip name

          Possible values:
          - esp32:   ESP32
          - esp32c2: ESP32-C2, ESP8684
          - esp32c3: ESP32-C3, ESP8685
          - esp32c6: ESP32-C6
          - esp32s2: ESP32-S2
          - esp32s3: ESP32-S3
          - esp32h2: ESP32-H2
          - esp8266: ESP8266

  -b, --bootloader <BOOTLOADER>
          Path to bootloader

  -p, --partition-table <PARTITION_TABLE>
          Path to partition table CSV

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
