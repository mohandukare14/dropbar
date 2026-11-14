#!/usr/bin/env bash
set -e

echo "==> Building DropBar APK for P4 (Android)..."

DROPBAR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MOBILE_DIR="$DROPBAR_DIR/dropbar-mobile"

if ! command -v flutter &> /dev/null; then
    echo "⚠️ Flutter SDK not found in PATH."
    echo ""
    echo "To build locally on Arch Linux, install Flutter:"
    echo "  sudo pacman -S jdk17-openjdk"
    echo "  yay -S flutter"
    echo ""
    echo "Or use GitHub Actions (pre-configured in .github/workflows/build-apk.yml)"
    echo "to automatically compile and download DropBar.apk in 2 minutes."
    exit 1
fi

cd "$MOBILE_DIR"
echo "  [1/3] Getting Flutter dependencies..."
flutter pub get

echo "  [2/3] Building Release APK..."
flutter build apk --release

OUTPUT_APK="$MOBILE_DIR/build/app/outputs/flutter-apk/app-release.apk"
if [ -f "$OUTPUT_APK" ]; then
    echo ""
    echo "✅ Success! DropBar APK ready at:"
    echo "   $OUTPUT_APK"
    echo ""
    echo "To install on your connected P4 phone:"
    echo "   adb install $OUTPUT_APK"
fi

