# 📁 MAME Folder Structure Guide

## Overview
MAMEUIx menggunakan struktur folder MAME standar untuk mengorganisir file-file yang diperlukan oleh emulator MAME. Struktur ini dikonfigurasi melalui dialog **Options → Directories** dalam aplikasi.

## 🗂️ Struktur Folder MAME Standar

### 1. **ROM Directories** (Tab: MAME Paths)
```
roms/
├── pacman.zip          # Game ROM files
├── donkey.zip
├── galaga.zip
└── ...
```

**Konfigurasi di MAMEUIx:**
- **Tab**: "MAME Paths" → "ROM Directories"
- **Fungsi**: Folder yang berisi file ROM game dalam format .zip
- **Contoh Path**: `/home/user/mame/roms/`

#### 🔧 **Linux Separator Support**
MAMEUIx mendukung separator `;` untuk kompatibilitas Linux:

**Quick Add dengan Separator:**
- **Input**: `/path1;/path2;/path3`
- **Fungsi**: Menambahkan multiple ROM directories sekaligus
- **Format**: Path dipisahkan dengan semicolon (`;`)

**Contoh Penggunaan:**
```
/home/user/mame/roms;/opt/mame/roms;/media/external/roms
```

**Fitur Copy-Paste:**
- **Display**: Menampilkan current paths dalam format separator
- **Copy Button**: Menyalin paths ke clipboard untuk penggunaan di tempat lain
- **Format Output**: `/path1;/path2;/path3`

**Keuntungan:**
- ✅ **Kompatibilitas Linux**: Sesuai dengan format MAME Linux
- ✅ **Batch Input**: Menambahkan multiple paths sekaligus
- ✅ **Easy Copy-Paste**: Mudah menyalin ke aplikasi lain
- ✅ **Validation**: Otomatis validasi path yang ada

### 2. **Sample Directories** (Tab: MAME Paths)
```
samples/
├── pacman/             # Sound samples per game
│   ├── sample1.wav
│   └── sample2.wav
├── donkey/
└── ...
```

**Konfigurasi di MAMEUIx:**
- **Tab**: "MAME Paths" → "Sample Directories"
- **Fungsi**: File audio sample untuk game yang memerlukan sound samples
- **Contoh Path**: `/home/user/mame/samples/`

### 3. **MAME Support Files** (Tab: MAME Support Files)

#### Artwork Directory
```
artwork/
├── pacman.png          # Game artwork
├── donkey.png
└── ...
```

#### Screenshots Directory
```
snap/
├── pacman.png          # Game screenshots
├── donkey.png
└── ...
```

#### Cabinet Artwork
```
cabinets/
├── pacman.png          # Cabinet artwork
├── donkey.png
└── ...
```

#### Title Screens
```
titles/
├── pacman.png          # Title screen images
├── donkey.png
└── ...
```

#### Promotional Flyers
```
flyers/
├── pacman.png          # Promotional flyers
├── donkey.png
└── ...
```

#### Marquee Artwork
```
marquees/
├── pacman.png          # Marquee artwork
├── donkey.png
└── ...
```

#### Cheat Files
```
cheat/
├── cheat.dat           # Cheat codes database
└── ...
```

#### Game Icons
```
icons/
├── pacman.png          # Game icons
├── donkey.png
└── ...
```

### 4. **History, INI's and DAT's Files** (Tab: History, INI's and DAT's Files)

#### Category Support
```
catver.ini              # Game categories database
```

#### Game History
```
history.xml             # Game history information
```

#### MAME Info Database
```
mameinfo.dat            # Detailed game information
```

#### High Score Database
```
hiscore.dat             # High score information
```

#### Game Initialization
```
gameinit.dat            # Game initialization data
```

#### Command Database
```
command.dat             # Game command information
```

## 🔧 Konfigurasi di MAMEUIx

### Dialog Directories
Untuk mengkonfigurasi struktur folder MAME:

1. **Buka aplikasi MAMEUIx**
2. **Klik Options → Directories**
3. **Pilih tab yang sesuai:**

#### Tab 1: "MAME Paths"
- **MAME Executables**: Path ke executable MAME
- **ROM Directories**: Folder yang berisi file ROM (.zip)
- **Sample Directories**: Folder yang berisi sound samples

#### Tab 2: "MAME Support Files"
- **Artwork**: Game artwork files
- **Snap**: Game screenshots
- **Cabinet**: Cabinet artwork
- **Title**: Title screens
- **Flyer**: Promotional flyers
- **Marquee**: Marquee artwork
- **Cheats**: Cheat files
- **Icons**: Game icon files

#### Tab 3: "History, INI's and DAT's Files"
- **Catver INI**: Game category information (catver.ini)
- **History**: Game history information (history.xml)
- **MAME Info DAT**: Detailed game information (mameinfo.dat)
- **High Score DAT**: High score information (hiscore.dat)
- **Game Init DAT**: Game initialization data (gameinit.dat)
- **Command DAT**: Game command information (command.dat)

## 📂 Folder yang Tidak Dikonfigurasi Langsung

### cfg/ Directory
```
cfg/
├── default.cfg         # Default MAME configuration
├── pacman.cfg          # Game-specific configuration
├── donkey.cfg
└── ...
```

**Status**: Folder `cfg/` tidak dikonfigurasi langsung di MAMEUIx karena:
- MAME secara otomatis membuat folder ini di lokasi default
- Biasanya berada di `~/.mame/cfg/` (Linux) atau `%USERPROFILE%\mame\cfg\` (Windows)
- Berisi konfigurasi per-game yang dibuat otomatis saat game pertama kali dimainkan

### nvram/ Directory
```
nvram/
├── pacman/             # NVRAM data per game
│   ├── pacman.nv
│   └── ...
├── donkey/
└── ...
```

**Status**: Folder `nvram/` juga tidak dikonfigurasi langsung karena:
- Dibuat otomatis oleh MAME saat game menyimpan data
- Biasanya berada di `~/.mame/nvram/` (Linux) atau `%USERPROFILE%\mame\nvram\` (Windows)
- Berisi data yang disimpan game (high scores, settings, dll.)

### hi/ Directory
```
hi/
├── pacman.hi           # High score files
├── donkey.hi
└── ...
```

**Status**: Folder `hi/` untuk high score files juga dikelola otomatis oleh MAME.

## 🚀 Cara MAMEUIx Menggunakan Struktur Ini

### 1. **ROM Scanning**
```rust
// MAMEUIx scan ROM directories untuk menemukan game yang tersedia
pub rom_paths: Vec<PathBuf>,     // ROM directories
pub sample_paths: Vec<PathBuf>,  // Sample directories
```

### 2. **Artwork Loading**
```rust
// MAMEUIx load artwork dari berbagai folder
pub artwork_path: Option<PathBuf>,   // Artwork directory
pub snap_path: Option<PathBuf>,      // Screenshots directory
pub cabinet_path: Option<PathBuf>,   // Cabinet artwork directory
pub title_path: Option<PathBuf>,     // Title screens directory
pub flyer_path: Option<PathBuf>,     // Promotional flyers directory
pub marquee_path: Option<PathBuf>,   // Marquee artwork directory
```

### 3. **Game Launching**
```rust
// Saat launch game, MAMEUIx pass ROM paths ke MAME
cmd.arg("-rompath").arg(&rom_paths);
```

### 4. **Category Support**
```rust
// MAMEUIx load categories dari catver.ini
pub catver_ini_path: Option<PathBuf>,    // catver.ini file path
```

## 📋 Rekomendasi Struktur Folder

### Struktur Lengkap yang Direkomendasikan:
```
mame/
├── mame                 # MAME executable
├── roms/                # ROM files
│   ├── pacman.zip
│   ├── donkey.zip
│   └── ...
├── samples/             # Sound samples
│   ├── pacman/
│   ├── donkey/
│   └── ...
├── artwork/             # Game artwork
│   ├── pacman.png
│   ├── donkey.png
│   └── ...
├── snap/                # Screenshots
├── cabinets/            # Cabinet artwork
├── titles/              # Title screens
├── flyers/              # Promotional flyers
├── marquees/            # Marquee artwork
├── cheat/               # Cheat files
│   └── cheat.dat
├── icons/               # Game icons
├── catver.ini           # Category database
├── history.xml          # Game history
├── mameinfo.dat         # Game information
├── hiscore.dat          # High score database
├── gameinit.dat         # Game initialization
├── command.dat          # Command database
├── cfg/                 # Auto-generated configs
├── nvram/               # Auto-generated NVRAM
└── hi/                  # Auto-generated high scores
```

## 🔍 Troubleshooting

### Masalah Umum:

1. **ROM tidak ditemukan**
   - Periksa path ROM directories di Options → Directories
   - Pastikan file ROM dalam format .zip
   - Periksa nama file ROM sesuai dengan yang diharapkan MAME

2. **Artwork tidak muncul**
   - Periksa path artwork di tab "MAME Support Files"
   - Pastikan nama file artwork sesuai dengan nama ROM
   - Periksa format file (PNG, JPG, dll.)

3. **Categories tidak muncul**
   - Pastikan catver.ini dikonfigurasi di tab "History, INI's and DAT's Files"
   - Download catver.ini dari sumber MAME community
   - Restart aplikasi setelah mengkonfigurasi catver.ini

4. **Game tidak bisa dijalankan**
   - Periksa path MAME executable
   - Pastikan MAME executable memiliki permission execute
   - Periksa ROM paths di konfigurasi

## 📚 Referensi

- [MAME Documentation](https://docs.mamedev.org/)
- [MAME Directory Structure](https://docs.mamedev.org/initialsetup/directorystructure.html)
- [CatVer.ini Download](https://www.progettosnaps.net/catver/)
- [MAME Community Resources](https://www.mamedev.org/resources/) 