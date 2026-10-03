#!/usr/bin/env bash
# Builds Formiga Hill as a universal macOS app that Formiga Desktop can find: the bundle id and
# the travel version it reads are the ones formiga-travel's discovery module names.
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_dir="$(cd "$script_dir/.." && pwd)"
dist_dir="$repo_dir/dist"
app_dir="$dist_dir/Formiga Hill.app"
version="${FORMIGA_HILL_VERSION:-$(cargo metadata --no-deps --format-version 1 --manifest-path "$repo_dir/Cargo.toml" | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == "formiga-hill"))')}"
version="${version#v}"
build_number="${FORMIGA_HILL_BUILD_NUMBER:-1}"
archive="$dist_dir/Formiga-Hill-$version-macOS-universal.zip"
disk_image="$dist_dir/Formiga-Hill-$version-macOS-universal.dmg"
dmg_staging="$dist_dir/Formiga-Hill-dmg"

cd "$repo_dir"
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release -p formiga-hill --target aarch64-apple-darwin
cargo build --release -p formiga-hill --target x86_64-apple-darwin

rm -rf "$app_dir" "$dmg_staging"
mkdir -p "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources"
cp "$repo_dir/packaging/macos/Info.plist" "$app_dir/Contents/Info.plist"
lipo -create \
    "$repo_dir/target/aarch64-apple-darwin/release/formiga-hill" \
    "$repo_dir/target/x86_64-apple-darwin/release/formiga-hill" \
    -output "$app_dir/Contents/MacOS/Formiga Hill"
chmod 755 "$app_dir/Contents/MacOS/Formiga Hill"

# The travel version comes from the binary itself, so the bundle can never claim another.
travel_version="$("$app_dir/Contents/MacOS/Formiga Hill" --travel-version)"
plist="$app_dir/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $version" "$plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $build_number" "$plist"
/usr/libexec/PlistBuddy -c "Set :FormigaTravelVersion $travel_version" "$plist"

if [[ -n "${FORMIGA_CODESIGN_IDENTITY:-}" ]]; then
    codesign --force --deep --options runtime --timestamp \
        --sign "$FORMIGA_CODESIGN_IDENTITY" "$app_dir"
else
    codesign --force --deep --sign - "$app_dir"
fi

ditto -c -k --sequesterRsrc --keepParent "$app_dir" "$archive"
mkdir -p "$dmg_staging"
ditto "$app_dir" "$dmg_staging/Formiga Hill.app"
ln -s /Applications "$dmg_staging/Applications"
cp "$repo_dir/packaging/macos/README.txt" "$dmg_staging/Read Me.txt"
hdiutil create -volname "Formiga Hill" -srcfolder "$dmg_staging" -ov -format UDZO "$disk_image"

if [[ -n "${FORMIGA_NOTARY_PROFILE:-}" ]]; then
    xcrun notarytool submit "$disk_image" --keychain-profile "$FORMIGA_NOTARY_PROFILE" --wait
    xcrun stapler staple "$disk_image"
fi

shasum -a 256 "$archive" > "$archive.sha256"
shasum -a 256 "$disk_image" > "$disk_image.sha256"

echo "Packaged $disk_image and $archive (travel version $travel_version)"
