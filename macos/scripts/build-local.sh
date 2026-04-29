#!/usr/bin/env bash
# Local macOS release build: archive → export → notarize app → staple app
# → DMG → codesign DMG → notarize DMG → staple DMG → appcast.
#
# Reads configuration from macos/.env. Uses the Developer ID cert already in
# the user's Keychain — does NOT import .p12.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MACOS_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

ENV_FILE="$MACOS_DIR/.env"
if [ ! -f "$ENV_FILE" ]; then
  echo "ERROR: $ENV_FILE not found. Copy .env.example to .env and fill it in." >&2
  exit 1
fi
set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

: "${SPARKLE_BIN:=$HOME/Tools/sparkle-2.9.1/bin}"
SIGN_UPDATE="$SPARKLE_BIN/sign_update"
if [ ! -x "$SIGN_UPDATE" ]; then
  echo "ERROR: sign_update not found at $SIGN_UPDATE" >&2
  echo "  Set SPARKLE_BIN in the environment to the directory containing sign_update." >&2
  exit 1
fi

# DMGMaker — pinned to a specific commit so upstream changes can't break builds.
: "${DMGMAKER_DIR:=$HOME/Tools/DMGMaker}"
: "${DMGMAKER_REPO:=https://github.com/saihgupr/DMGMaker.git}"
: "${DMGMAKER_COMMIT:=9e17aec84483938d154ce159ded8ce641379d5aa}"

setup_dmgmaker() {
  if [ ! -d "$DMGMAKER_DIR/.git" ]; then
    echo "==> Cloning DMGMaker into $DMGMAKER_DIR"
    git clone "$DMGMAKER_REPO" "$DMGMAKER_DIR"
  fi
  local current
  current=$(git -C "$DMGMAKER_DIR" rev-parse HEAD 2>/dev/null || echo "")
  if [ "$current" != "$DMGMAKER_COMMIT" ]; then
    echo "==> Pinning DMGMaker to $DMGMAKER_COMMIT"
    git -C "$DMGMAKER_DIR" fetch --depth 1 origin "$DMGMAKER_COMMIT" 2>/dev/null \
      || git -C "$DMGMAKER_DIR" fetch origin
    git -C "$DMGMAKER_DIR" checkout --detach "$DMGMAKER_COMMIT"
  fi
}

required=(APPLE_TEAM_ID DEVELOPER_ID_APPLICATION APPLE_ID APPLE_APP_PASSWORD
          SPARKLE_ED25519_PUBLIC_KEY SPARKLE_DOWNLOAD_URL_PREFIX
          VERSION_NUMBER BUILD_NUMBER)
for var in "${required[@]}"; do
  if [ -z "${!var:-}" ]; then
    echo "ERROR: $var not set in $ENV_FILE" >&2
    exit 1
  fi
done

INFO_PLIST="$MACOS_DIR/SEE/Info.plist"
PLIST_KEY=$(/usr/libexec/PlistBuddy -c "Print :SUPublicEDKey" "$INFO_PLIST")
if [ "$PLIST_KEY" != "$SPARKLE_ED25519_PUBLIC_KEY" ]; then
  echo "ERROR: Info.plist SUPublicEDKey ($PLIST_KEY) does not match .env SPARKLE_ED25519_PUBLIC_KEY." >&2
  exit 1
fi

if ! security find-identity -v -p codesigning | grep -q "$DEVELOPER_ID_APPLICATION"; then
  echo "ERROR: Codesigning identity not found in any Keychain:" >&2
  echo "  $DEVELOPER_ID_APPLICATION" >&2
  exit 1
fi

PROJECT="$MACOS_DIR/SEE.xcodeproj"
SCHEME="SEE"
BUILD_DIR="$MACOS_DIR/build"
ARCHIVE_PATH="$BUILD_DIR/SEE.xcarchive"
EXPORT_DIR="$BUILD_DIR/export"
APP_PATH="$EXPORT_DIR/SEE.app"
APP_ZIP="$BUILD_DIR/SEE.app.zip"
DMG_NAME="SEE-${VERSION_NUMBER}.dmg"
DMG_PATH="$BUILD_DIR/$DMG_NAME"
APPCAST_PATH="$BUILD_DIR/appcast.xml"
EXPORT_OPTIONS="$BUILD_DIR/ExportOptions.plist"

mkdir -p "$BUILD_DIR"
rm -rf "$ARCHIVE_PATH" "$EXPORT_DIR" "$APP_ZIP" "$DMG_PATH" "$APPCAST_PATH" "$BUILD_DIR/dmg-staging"

echo "==> Resolving SPM dependencies"
xcodebuild -resolvePackageDependencies \
  -project "$PROJECT" -scheme "$SCHEME" >/dev/null

echo "==> Archiving (Release, version $VERSION_NUMBER build $BUILD_NUMBER)"
xcodebuild archive \
  -project "$PROJECT" \
  -scheme "$SCHEME" \
  -configuration Release \
  -destination "generic/platform=macOS" \
  -archivePath "$ARCHIVE_PATH" \
  MARKETING_VERSION="$VERSION_NUMBER" \
  CURRENT_PROJECT_VERSION="$BUILD_NUMBER" \
  CODE_SIGN_STYLE=Manual \
  CODE_SIGN_IDENTITY="Developer ID Application" \
  DEVELOPMENT_TEAM="$APPLE_TEAM_ID" \
  OTHER_CODE_SIGN_FLAGS="--timestamp --options runtime" \
  ONLY_ACTIVE_ARCH=NO

echo "==> Exporting archive (developer-id)"
cat > "$EXPORT_OPTIONS" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>method</key>
  <string>developer-id</string>
  <key>signingStyle</key>
  <string>manual</string>
  <key>teamID</key>
  <string>$APPLE_TEAM_ID</string>
  <key>signingCertificate</key>
  <string>Developer ID Application</string>
</dict>
</plist>
PLIST

xcodebuild -exportArchive \
  -archivePath "$ARCHIVE_PATH" \
  -exportOptionsPlist "$EXPORT_OPTIONS" \
  -exportPath "$EXPORT_DIR"

echo "==> Notarizing .app (round 1)"
ditto -c -k --keepParent "$APP_PATH" "$APP_ZIP"
xcrun notarytool submit "$APP_ZIP" \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_APP_PASSWORD" \
  --team-id "$APPLE_TEAM_ID" \
  --wait --timeout 30m

echo "==> Stapling .app"
xcrun stapler staple "$APP_PATH"
xcrun stapler validate "$APP_PATH"

echo "==> Creating DMG with DMGMaker (stapled .app)"
setup_dmgmaker
( cd "$DMGMAKER_DIR" && swift run -c release "DMG Maker" \
    --app "$APP_PATH" --name "S.EE" )
# DMGMaker writes <name>.dmg next to the input .app and gives no -o flag.
mv "$EXPORT_DIR/S.EE.dmg" "$DMG_PATH"

echo "==> Codesigning DMG"
codesign --sign "$DEVELOPER_ID_APPLICATION" --timestamp "$DMG_PATH"

echo "==> Notarizing DMG (round 2)"
xcrun notarytool submit "$DMG_PATH" \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_APP_PASSWORD" \
  --team-id "$APPLE_TEAM_ID" \
  --wait --timeout 30m

echo "==> Stapling DMG"
xcrun stapler staple "$DMG_PATH"
xcrun stapler validate "$DMG_PATH"

echo "==> Final Gatekeeper assessment"
spctl -a -vv -t open --context context:primary-signature "$DMG_PATH" 2>&1 || true
spctl -a -vv -t exec "$APP_PATH" 2>&1 || true

echo "==> Generating Sparkle appcast"
SIGN_OUTPUT=$("$SIGN_UPDATE" "$DMG_PATH")
SIGNATURE=$(printf '%s' "$SIGN_OUTPUT" | sed -n 's/.*sparkle:edSignature="\([^"]*\)".*/\1/p')
if [ -z "$SIGNATURE" ]; then
  echo "ERROR: failed to extract sparkle:edSignature from sign_update output:" >&2
  echo "$SIGN_OUTPUT" >&2
  exit 1
fi
DMG_SIZE=$(stat -f%z "$DMG_PATH")
DOWNLOAD_URL="${SPARKLE_DOWNLOAD_URL_PREFIX}${DMG_NAME}"
PUB_DATE=$(date -R)

cat > "$APPCAST_PATH" <<APPCAST
<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>S.EE</title>
    <link>${SPARKLE_DOWNLOAD_URL_PREFIX}appcast.xml</link>
    <description>S.EE App Updates</description>
    <language>en</language>
    <item>
      <title>Version ${VERSION_NUMBER}</title>
      <sparkle:version>${BUILD_NUMBER}</sparkle:version>
      <sparkle:shortVersionString>${VERSION_NUMBER}</sparkle:shortVersionString>
      <sparkle:minimumSystemVersion>14.0</sparkle:minimumSystemVersion>
      <pubDate>${PUB_DATE}</pubDate>
      <enclosure url="${DOWNLOAD_URL}"
        sparkle:edSignature="${SIGNATURE}"
        length="${DMG_SIZE}"
        type="application/octet-stream" />
    </item>
  </channel>
</rss>
APPCAST

shasum -a 256 "$DMG_PATH" > "${DMG_PATH}.sha256"

echo
echo "Done."
echo "  DMG:     $DMG_PATH"
echo "  Appcast: $APPCAST_PATH"
echo "  SHA256:  ${DMG_PATH}.sha256"
