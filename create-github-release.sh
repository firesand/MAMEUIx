#!/bin/bash
set -euo pipefail

TAG="v0.1.5"
TITLE="MAMEUIx v0.1.5"
NOTES=".github/release-notes/v0.1.5.md"
ARCH="$(uname -m)"
VERSION="${TAG#v}"
APPIMAGE="MAMEUIx-${VERSION}-${ARCH}.AppImage"

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

if gh release view "${TAG}" >/dev/null 2>&1; then
    echo "Release ${TAG} already exists."
    if [[ -f "${APPIMAGE}" ]]; then
        echo "Uploading AppImage asset: ${APPIMAGE}"
        gh release upload "${TAG}" "${APPIMAGE}" --clobber
    else
        echo "No AppImage at ${APPIMAGE}. Build first: ./build-appimage.sh"
    fi
    echo "Edit notes with: gh release edit ${TAG} --notes-file ${NOTES}"
    exit 0
fi

RELEASE_ARGS=(--title "${TITLE}" --notes-file "${NOTES}")
if [[ -f "${APPIMAGE}" ]]; then
    gh release create "${TAG}" "${APPIMAGE}" "${RELEASE_ARGS[@]}"
else
    echo "Note: ${APPIMAGE} not found; creating release without AppImage asset."
    echo "Build later with ./build-appimage.sh and: gh release upload ${TAG} ${APPIMAGE}"
    gh release create "${TAG}" "${RELEASE_ARGS[@]}"
fi

echo "Release published: https://github.com/firesand/MAMEUIx/releases/tag/${TAG}"
