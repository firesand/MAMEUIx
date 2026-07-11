# MAME Folder and File Paths

MAMEUIx tidak mengharuskan semua file berada dalam satu folder. Pilih lokasi yang
sesuai dengan instalasi MAME Anda; path yang dikosongkan akan mengikuti default
MAME atau tidak digunakan oleh MAMEUIx.

## Tempat mengatur path

Ada dua tampilan konfigurasi:

- **Settings → Directories** pada UI redesign menyediakan pengaturan cepat untuk
  executable MAME, ROM, CHD, artwork, dan file data.
- **Options → Directories & Paths** menyediakan pengaturan lengkap dalam lima
  bagian. Gunakan dialog ini untuk Software Lists, support files, dan folder
  internal MAME.

Perubahan ROM pada halaman Settings memicu pemindaian ulang secara otomatis.
Pada dialog Directories & Paths, simpan perubahan dengan tombol **OK**.

## Settings → Directories

Halaman Directories pada UI redesign berisi:

- **MAME**: MAME executable, ROMs, dan CHDs.
- **Artwork**: Artwork root, Snapshots, Marquees, Title screens, Flyers,
  Cabinets, dan PCB.
- **Data Files**: `catver.ini`, `history.xml`, `mameinfo.dat`, `command.dat`,
  `hiscore.dat`, dan `gameinit.dat`.

Path CHD ikut diteruskan ke search path ROM MAME. `catver.ini` diperlukan jika
Anda ingin menampilkan kategori game; file data lainnya bersifat opsional.

## Options → Directories & Paths

### MAME Paths

- **MAME Executables**: satu atau beberapa executable MAME.
- **ROM Directories**: satu atau beberapa folder ROM, misalnya `roms/`.

Saat game dijalankan, semua ROM directory digabungkan menjadi nilai
`-rompath` untuk MAME.

### Software Lists

- **Hash XML Directory**: folder `hash/` yang berisi definisi seperti
  `a2600.xml`, `nes.xml`, atau `msx1_cart.xml`.
- **Software-list ROMs**: root set software-list terpisah, biasanya dengan
  subfolder bernama sesuai daftar, misalnya `a2600/` atau `nes/`.
- **Software Media Directory**: media lepas seperti cartridge, disk, atau
  cassette yang diteruskan melalui `-swpath`.

Hash XML digunakan untuk membangun tabel Software Lists. Root software-list
ikut ditambahkan ke `-rompath`, sedangkan Hash XML Directory diteruskan melalui
`-hashpath`. Kolom **Media path** pada preview Software Lists hanya memeriksa
keberadaan path secara best-effort; isi arsip dan CHD tidak diaudit, dan layout
merged set mungkin tidak terdeteksi secara lengkap.

### Support Files

- **Artwork**: artwork yang juga dapat diteruskan ke MAME melalui `-artpath`.
- **Snap**: screenshot game.
- **Cabinet**, **Title**, **Flyer**, dan **Marquees**: gambar untuk panel
  artwork MAMEUIx.
- **Cheats**: folder cheat; jika tersedia, MAME dijalankan dengan cheat aktif
  dan `-cheatpath`.
- **Icons**: ikon game.

Nama file gambar umumnya harus mengikuti short name ROM, misalnya
`pacman.png` untuk `pacman.zip`.

### INI & DAT Files

- **Catver INI**: `catver.ini` untuk kategori game.
- **History**: `history.xml`.
- **MAME Info DAT**: `mameinfo.dat`.
- **High Score DAT**: `hiscore.dat`.
- **Game Init DAT**: `gameinit.dat`.
- **Command DAT**: `command.dat`.

Field pada bagian ini memilih file, bukan folder.

### Internal Folders

Folder berikut dapat dikonfigurasi langsung dan akan menggantikan lokasi
default MAME saat game diluncurkan:

- **Configuration Files (cfg)** → `-cfg_directory`
- **NVRAM** → `-nvram_directory`
- **Input Configuration (input)** → `-input_directory`
- **Save States (state)** → `-state_directory`
- **Hard Disk Diffs (diff)** → `-diff_directory`
- **Comment Files (comment)** → `-comment_directory`

Jadi, `cfg/` dan `nvram/` bukan lagi folder yang hanya dapat memakai lokasi
default. Kosongkan field jika Anda memang ingin mengikuti default MAME. Jika
memilih lokasi sendiri, pastikan folder dapat ditulis oleh pengguna yang
menjalankan MAMEUIx.

## Contoh struktur

Struktur ini hanya contoh; setiap bagian dapat ditempatkan di lokasi berbeda.

```text
mame/
├── mame
├── roms/
├── chds/
├── hash/
├── software-roms/
│   ├── a2600/
│   └── nes/
├── software-media/
├── artwork/
├── snap/
├── cabinets/
├── titles/
├── flyers/
├── marquees/
├── pcb/
├── cheat/
├── icons/
├── cfg/
├── nvram/
├── input/
├── state/
├── diff/
├── comment/
├── catver.ini
├── history.xml
└── mameinfo.dat
```

## Pemeriksaan cepat

- Jika ROM tidak terdeteksi, periksa nama set dan semua **ROM Directories**.
- Jika CHD tidak ditemukan saat game berjalan, periksa path **CHDs** pada
  Settings atau tambahkan parent folder yang sesuai ke ROM path.
- Jika kategori kosong, pilih `catver.ini` yang valid.
- Jika artwork tidak muncul, cocokkan nama file dengan short name ROM dan
  periksa field artwork yang digunakan.
- Jika konfigurasi atau save data tidak tersimpan, pastikan folder Internal
  Folders yang dipilih ada dan dapat ditulis.

Referensi struktur dan opsi path MAME tersedia di
[dokumentasi resmi MAME](https://docs.mamedev.org/initialsetup/directorystructure.html).
