Name:           mameuix
Version:        0.1.1
Release:        1%{?dist}
Summary:        Modern GUI frontend for MAME arcade emulator

License:        MIT
URL:            https://github.com/firesand/MAMEUIx
Source0:        %{name}-%{version}.tar.gz
BuildArch:      x86_64

BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  pkgconfig
BuildRequires:  openssl-devel
BuildRequires:  gtk3-devel
BuildRequires:  webkit2gtk3-devel

Requires:       mame >= 0.200

%description
MAMEUIx is a modern, fast, and user-friendly graphical interface
for the MAME arcade emulator. It provides an intuitive way to browse,
search, and launch arcade games with features like:

* Fast game scanning and filtering
* Artwork display and management
* Multiple theme support (10 themes)
* ROM status detection
* Game history tracking
* Responsive and modern UI
* Persistent column widths
* Enhanced preferences dialog

%prep
%autosetup

%build
cargo build --release

%install
# Install binary
install -D -m 755 target/release/mameuix %{buildroot}%{_bindir}/mameuix

# Install desktop file
install -D -m 644 mameuix.desktop %{buildroot}%{_datadir}/applications/mameuix.desktop

# Install icons
install -D -m 644 assets/icons/16x16/mameuix.png %{buildroot}%{_datadir}/icons/hicolor/16x16/apps/mameuix.png
install -D -m 644 assets/icons/32x32/mameuix.png %{buildroot}%{_datadir}/icons/hicolor/32x32/apps/mameuix.png
install -D -m 644 assets/icons/48x48/mameuix.png %{buildroot}%{_datadir}/icons/hicolor/48x48/apps/mameuix.png
install -D -m 644 assets/icons/64x64/mameuix.png %{buildroot}%{_datadir}/icons/hicolor/64x64/apps/mameuix.png
install -D -m 644 assets/icons/128x128/mameuix.png %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/mameuix.png
install -D -m 644 assets/icons/256x256/mameuix.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/mameuix.png
install -D -m 644 assets/icons/scalable/mameuix.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/mameuix.svg

# Install man page (commented out for now)
# install -D -m 644 mameuix.1 %{buildroot}%{_mandir}/man1/mameuix.1

%files
%license LICENSE
%doc README.md
%{_bindir}/mameuix
%{_datadir}/applications/mameuix.desktop
%{_datadir}/icons/hicolor/16x16/apps/mameuix.png
%{_datadir}/icons/hicolor/32x32/apps/mameuix.png
%{_datadir}/icons/hicolor/48x48/apps/mameuix.png
%{_datadir}/icons/hicolor/64x64/apps/mameuix.png
%{_datadir}/icons/hicolor/128x128/apps/mameuix.png
%{_datadir}/icons/hicolor/256x256/apps/mameuix.png
%{_datadir}/icons/hicolor/scalable/apps/mameuix.svg
# %{_mandir}/man1/mameuix.1

%changelog
* Sun Jul 13 2025 MAMEUIx Team <mameuix@example.com> - 0.1.1-1
- Initial release
- Modern GUI frontend for MAME arcade emulator
- Features include fast game scanning, artwork display, multiple themes,
  ROM status detection, game history tracking, and responsive UI 