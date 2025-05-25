# SightSteer

## Create Virtual Environment for Remote Control
```
cd SightSteer\remote_control
python -m venv venv
.\venv\Scripts\activate # Windows
source venv/bin/activate # macOS/Linux
pip install -r requirements.txt
```

## Create ESP32 Project and Flash
### 1. Install Toolchain (One-Time Setup)

Open PowerShell and run:

```powershell
cargo install espup
espup install
```

After installation, restart your terminal and run:
```powershell
. $env:USERPROFILE\export-esp.ps1
```

### 2. Install Required Tools
```powershell
cargo install cargo-generate
cargo install cargo-espflash
cargo install ldproxy
```

### Create New Project (if you want a different one)
```powershell
cargo generate --git https://github.com/esp-rs/esp-idf-template cargo
```

Choose:
- esp32 for the MCU
- std = true

### 4. Move to Short Path
There can be an error when flashing with Windows when file path is too long

### 5. Flash the Board
```powershell
cargo espflash flash --release --port COM3 --no-stub
```

- `--no-stub` avoids certain issue: https://github.com/esp-rs/esp-flasher-stub/issues/63

### 6. Monitor Serial Output
```powershell
cargo espflash monitor --port COM3 --baud 74880 --no-stub
```

- Might need to change baud rate depending on ESP32 crystal frequency

### 7. Press RST Button
Should see logs and print statements after pressing button

## Flash ESP32 Board and Monitor Serial Output
```
cargo espflash flash --release --port COM3 --no-stub
cargo espflash monitor --port COM3 --baud 115200 --no-stub
```