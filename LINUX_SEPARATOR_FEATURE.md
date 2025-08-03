# 🔧 Linux Separator Feature

## Overview

MAMEUIx mendukung separator `;` untuk kompatibilitas Linux dalam pengaturan ROM paths. Fitur ini memungkinkan pengguna untuk menambahkan multiple ROM directories sekaligus menggunakan format yang kompatibel dengan MAME Linux.

## 🎯 Fitur Utama

### 1. **Quick Add dengan Separator**
- **Input Format**: `/path1;/path2;/path3`
- **Separator**: Semicolon (`;`)
- **Fungsi**: Menambahkan multiple ROM directories sekaligus
- **Validation**: Otomatis validasi dan filter path kosong

### 2. **Display Format**
- **Current Paths**: Menampilkan semua paths dalam format separator
- **Copy Button**: Menyalin paths ke clipboard
- **Read-only**: Field display tidak bisa diedit untuk menghindari error

### 3. **ROM File Counting**
- **Per Directory**: Menampilkan jumlah ROM files di setiap directory
- **Total Count**: Menampilkan total ROM files di semua directories
- **Real-time**: Update otomatis saat directory berubah

## 🚀 Cara Penggunaan

### 1. **Quick Add Multiple Paths**
```
1. Buka Options → Directories
2. Pilih tab "MAME Paths"
3. Di section "ROM Directories"
4. Masukkan paths dengan separator: /home/user/mame/roms;/opt/mame/roms
5. Klik "Add Paths"
6. Paths akan otomatis dipisah dan ditambahkan
```

### 2. **Copy-Paste Paths**
```
1. Paths yang sudah ada akan ditampilkan dalam format separator
2. Klik "Copy" untuk menyalin ke clipboard
3. Paste di aplikasi lain atau gunakan untuk backup
```

### 3. **Monitor ROM Files**
```
1. Setiap directory akan menampilkan jumlah ROM files
2. Total ROM files ditampilkan di bagian bawah
3. Status ✓ hijau untuk directory yang valid
4. Status (not found) merah untuk directory yang tidak ada
```

## 📋 Contoh Penggunaan

### Input Examples:
```
# Single path
/home/user/mame/roms

# Multiple paths
/home/user/mame/roms;/opt/mame/roms;/media/external/roms

# Mixed paths with spaces
/home/user/mame/roms; /opt/mame/roms ; /media/external/roms
```

### Output Examples:
```
# Display format
/home/user/mame/roms;/opt/mame/roms;/media/external/roms

# Individual paths
✓ /home/user/mame/roms (150 ROM files)
✓ /opt/mame/roms (75 ROM files)
✓ /media/external/roms (200 ROM files)

# Total count
Total ROM files found: 425
```

## 🔧 Implementasi Teknis

### 1. **Parsing Logic**
```rust
let new_paths: Vec<PathBuf> = separator_input
    .split(';')
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .map(|s| PathBuf::from(s))
    .collect();
```

### 2. **ROM Counting**
```rust
let rom_count = entries
    .filter_map(|e| e.ok())
    .filter(|e| {
        e.path().extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("zip"))
            .unwrap_or(false)
    })
    .count();
```

### 3. **Clipboard Integration**
```rust
if let Ok(mut clipboard) = arboard::Clipboard::new() {
    if let Err(e) = clipboard.set_text(separator_string.clone()) {
        eprintln!("Failed to copy to clipboard: {}", e);
    }
}
```

## 🎨 UI Components

### 1. **Quick Add Section**
- **Label**: "Quick Add (Linux separator):"
- **Description**: "Enter multiple paths separated by semicolon (;) for Linux compatibility"
- **Input Field**: TextEdit dengan hint text
- **Add Button**: "Add Paths"

### 2. **Help Section**
- **Collapsible**: "ℹ️ Help - Linux Separator"
- **Content**: Penjelasan format dan contoh penggunaan

### 3. **Display Section**
- **Label**: "Current paths (semicolon-separated):"
- **Read-only Field**: Menampilkan paths dalam format separator
- **Copy Button**: Menyalin ke clipboard
- **Total Count**: Menampilkan total ROM files

### 4. **Individual Paths Section**
- **Label**: "Individual Paths:"
- **Path Fields**: TextEdit untuk setiap path
- **Status Indicators**: ✓ hijau atau (not found) merah
- **ROM Count**: Jumlah ROM files per directory
- **Browse Buttons**: Browse untuk setiap path
- **Remove Buttons**: Hapus path individual

## ✅ Keuntungan

### 1. **Kompatibilitas Linux**
- ✅ Format sesuai dengan MAME Linux
- ✅ Separator `;` yang standar
- ✅ Mudah diintegrasikan dengan sistem Linux

### 2. **User Experience**
- ✅ Batch input untuk multiple paths
- ✅ Real-time validation
- ✅ Visual feedback dengan status indicators
- ✅ ROM file counting untuk monitoring

### 3. **Productivity**
- ✅ Quick add multiple paths
- ✅ Copy-paste functionality
- ✅ Easy backup dan restore
- ✅ Clear visual organization

### 4. **Error Handling**
- ✅ Automatic trimming whitespace
- ✅ Filter empty paths
- ✅ Duplicate prevention
- ✅ Path validation

## 🔄 Workflow

### 1. **Setup ROM Directories**
```
1. Open MAMEUIx
2. Go to Options → Directories
3. Select "MAME Paths" tab
4. Use Quick Add for multiple paths
5. Verify individual paths
6. Check ROM file counts
7. Save configuration
```

### 2. **Maintenance**
```
1. Monitor ROM file counts
2. Use copy-paste for backup
3. Add new directories as needed
4. Remove invalid directories
5. Update paths when moving ROMs
```

## 🐛 Troubleshooting

### Common Issues:
1. **Path not found**: Periksa apakah path benar dan ada
2. **No ROM files**: Pastikan directory berisi file .zip
3. **Copy failed**: Periksa clipboard permissions
4. **Separator error**: Pastikan menggunakan `;` bukan `,`

### Solutions:
1. **Use Browse button**: Untuk memastikan path benar
2. **Check permissions**: Pastikan akses ke directory
3. **Manual input**: Jika Quick Add gagal, gunakan individual paths
4. **Restart app**: Jika clipboard tidak berfungsi

## 📈 Future Enhancements

### 1. **Advanced Features**
- 🔄 Drag & drop untuk paths
- 🔄 Import/export path lists
- 🔄 Path templates
- 🔄 Network path support

### 2. **UI Improvements**
- 🔄 Progress indicators untuk ROM counting
- 🔄 Search/filter paths
- 🔄 Path categories
- 🔄 Custom separators

### 3. **Integration Features**
- 🔄 Auto-detect ROM directories
- 🔄 Sync dengan MAME configuration
- 🔄 Backup/restore settings
- 🔄 Cloud storage integration 