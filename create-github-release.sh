#!/bin/bash
set -euo pipefail

TAG="${1:-v0.1.6}"
if [[ ! "${TAG}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
    echo "Invalid release tag: ${TAG}" >&2
    echo "Usage: $0 [vMAJOR.MINOR.PATCH]" >&2
    exit 2
fi

TITLE="MAMEUIx ${TAG}"
NOTES=".github/release-notes/${TAG}.md"
ARCH="$(uname -m)"
VERSION="${TAG#v}"
APPIMAGE="MAMEUIx-${VERSION}-${ARCH}.AppImage"
CHECKSUM="${APPIMAGE}.sha256"

if ! command -v gh >/dev/null 2>&1; then
    echo "GitHub CLI (gh) is not installed."
    echo
    echo "Gentoo:  emerge -av github-cli"
    echo "Or create the release manually:"
    echo "  https://github.com/firesand/MAMEUIx/releases/new?tag=${TAG}"
    echo
    echo "Paste the contents of: ${NOTES}"
    exit 1
fi

if [[ ! -f "${NOTES}" ]]; then
    echo "Release notes not found: ${NOTES}" >&2
    exit 1
fi

if [[ ! -f "${APPIMAGE}" || ! -f "${CHECKSUM}" ]]; then
    echo "Refusing to publish an incomplete release." >&2
    echo "Required assets: ${APPIMAGE} and ${CHECKSUM}" >&2
    exit 1
fi

sha256sum -c "${CHECKSUM}"

if gh release view "${TAG}" >/dev/null 2>&1; then
    echo "Release ${TAG} already exists."
    echo "Refreshing release title and notes."
    gh release edit "${TAG}" --title "${TITLE}" --notes-file "${NOTES}"
    echo "Uploading AppImage assets: ${APPIMAGE}, ${CHECKSUM}"
    gh release upload "${TAG}" "${APPIMAGE}" "${CHECKSUM}" --clobber
    exit 0
fi

RELEASE_ARGS=(--verify-tag --title "${TITLE}" --notes-file "${NOTES}")
gh release create "${TAG}" "${APPIMAGE}" "${CHECKSUM}" "${RELEASE_ARGS[@]}"

echo "Release published: https://github.com/firesand/MAMEUIx/releases/tag/${TAG}"
