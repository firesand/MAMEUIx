Name:           mameuix
Version:        0.1.6
Release:        1%{?dist}
Summary:        Modern GUI frontend for MAME arcade emulator

License:        MIT AND OFL-1.1
URL:            https://github.com/firesand/MAMEUIx
Source0:        %{name}-%{version}.tar.gz
BuildArch:      x86_64

BuildRequires:  rust >= 1.85
BuildRequires:  cargo >= 1.85
BuildRequires:  gcc-c++
BuildRequires:  pkgconfig
BuildRequires:  libxcb-devel
BuildRequires:  libxkbcommon-devel
BuildRequires:  libX11-devel
BuildRequires:  libXcursor-devel
BuildRequires:  libXi-devel
BuildRequires:  libXinerama-devel
BuildRequires:  libXrandr-devel
BuildRequires:  cmake

Requires:       mame >= 0.200
Requires:       libX11
Requires:       libxcb
Requires:       libXcursor
Requires:       libXi
Requires:       libXinerama
Requires:       libXrandr
Requires:       libxkbcommon
Requires:       wayland-libs

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
cargo build --release --locked

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

# Install man page
install -D -m 644 debian/mameuix.1 %{buildroot}%{_mandir}/man1/mameuix.1

%files
%license LICENSE assets/fonts/public_sans/OFL.txt
%doc README.md CHANGELOG.md
%doc docs/INSTALL.md docs/MAME_FOLDER_STRUCTURE.md docs/BGFX_GLSL_INTEGRATION.md
%{_bindir}/mameuix
%{_datadir}/applications/mameuix.desktop
%{_datadir}/icons/hicolor/16x16/apps/mameuix.png
%{_datadir}/icons/hicolor/32x32/apps/mameuix.png
%{_datadir}/icons/hicolor/48x48/apps/mameuix.png
%{_datadir}/icons/hicolor/64x64/apps/mameuix.png
%{_datadir}/icons/hicolor/128x128/apps/mameuix.png
%{_datadir}/icons/hicolor/256x256/apps/mameuix.png
%{_datadir}/icons/hicolor/scalable/apps/mameuix.svg
%{_mandir}/man1/mameuix.1

%changelog
* Sat Jul 11 2026 edo hikmahtiar <edohikmahtiar@me.com> - 0.1.6-1
- Add the opt-in redesigned UI and Software Lists preview
- Refresh documentation, AppImage release metadata, and FreeBSD readiness

* Sun Jul 05 2026 edo hikmahtiar <edohikmahtiar@me.com> - 0.1.5-1
- Stability, UI consistency, parser, and packaging metadata updates

* Sun Aug 03 2025 MAMEUIx Team <edohikmahtiar@me.com> - 0.1.4-1
- CLRMamePro Lite ROM verification
- Performance improvements and shader system enhancements

* Sun Jul 13 2025 MAMEUIx Team <edohikmahtiar@me.com> - 0.1.1-1
- Initial release
- Modern GUI frontend for MAME arcade emulator
- Features include fast game scanning, artwork display, multiple themes,
  ROM status detection, game history tracking, and responsive UI 
