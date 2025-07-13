Name:           mame-frontend
Version:        0.1.1
Release:        1%{?dist}
Summary:        Modern GUI frontend for MAME arcade emulator

License:        MIT
URL:            https://github.com/yourusername/mame-frontend
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
MAME Frontend is a modern, fast, and user-friendly graphical interface
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
install -D -m 755 target/release/mame-frontend %{buildroot}%{_bindir}/mame-frontend

# Install desktop file
install -D -m 644 mame-frontend.desktop %{buildroot}%{_datadir}/applications/mame-frontend.desktop

# Install icons
install -D -m 644 assets/icons/16x16/mame-frontend-icon.png %{buildroot}%{_datadir}/icons/hicolor/16x16/apps/mame-frontend.png
install -D -m 644 assets/icons/32x32/mame-frontend-icon.png %{buildroot}%{_datadir}/icons/hicolor/32x32/apps/mame-frontend.png
install -D -m 644 assets/icons/48x48/mame-frontend-icon.png %{buildroot}%{_datadir}/icons/hicolor/48x48/apps/mame-frontend.png
install -D -m 644 assets/icons/64x64/mame-frontend-icon.png %{buildroot}%{_datadir}/icons/hicolor/64x64/apps/mame-frontend.png
install -D -m 644 assets/icons/128x128/mame-frontend-icon.png %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/mame-frontend.png
install -D -m 644 assets/icons/256x256/mame-frontend-icon.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/mame-frontend.png
install -D -m 644 assets/icons/scalable/mame-frontend-icon.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/mame-frontend.svg

# Install man page
install -D -m 644 mame-frontend.1 %{buildroot}%{_mandir}/man1/mame-frontend.1

%files
%license LICENSE
%doc README.md
%{_bindir}/mame-frontend
%{_datadir}/applications/mame-frontend.desktop
%{_datadir}/icons/hicolor/16x16/apps/mame-frontend.png
%{_datadir}/icons/hicolor/32x32/apps/mame-frontend.png
%{_datadir}/icons/hicolor/48x48/apps/mame-frontend.png
%{_datadir}/icons/hicolor/64x64/apps/mame-frontend.png
%{_datadir}/icons/hicolor/128x128/apps/mame-frontend.png
%{_datadir}/icons/hicolor/256x256/apps/mame-frontend.png
%{_datadir}/icons/hicolor/scalable/apps/mame-frontend.svg
%{_mandir}/man1/mame-frontend.1

%changelog
* $(date '+%a %b %d %Y') MAME Frontend Team <mame-frontend@example.com> - 0.1.1-1
- Initial release
- Modern GUI frontend for MAME arcade emulator
- Features include fast game scanning, artwork display, multiple themes,
  ROM status detection, game history tracking, and responsive UI 