#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
sdk=${ANDROID_SDK_ROOT:-${ANDROID_HOME:-}}
if [[ -z "$sdk" ]]; then
    printf 'Set ANDROID_HOME to an Android SDK with platform 36 and build-tools 36.0.0.\n' >&2
    exit 1
fi
ndk=${ANDROID_NDK_HOME:-"$sdk/ndk/28.2.13676358"}
case "$(uname -s)" in
    Linux) host=linux-x86_64 ;;
    Darwin) host=darwin-x86_64 ;;
    *) printf 'Run this script on Linux or macOS.\n' >&2; exit 1 ;;
esac
llvm="$ndk/toolchains/llvm/prebuilt/$host/bin"
tools="$sdk/build-tools/36.0.0"
platform="$sdk/platforms/android-36/android.jar"
out="$PWD/build/android"
assemblies="$out/native"
version=$(sed -n 's/.*android:versionName="\([^"]*\)".*/\1/p' packaging/AndroidManifest.xml)
apk="$out/Waldwinkel-$version.apk"
key="${WALDWINKEL_KEYSTORE:-$out/waldwinkel-test.jks}"
read -r -a architectures <<< "${WALDWINKEL_ABIS:-arm64-v8a x86_64}"

mkdir -p "$out" "$assemblies" "$out/classes"
for abi in "${architectures[@]}"; do
    case "$abi" in
        arm64-v8a) rust_target=aarch64-linux-android ;;
        x86_64) rust_target=x86_64-linux-android ;;
        *) printf 'Unsupported ABI: %s\n' "$abi" >&2; exit 1 ;;
    esac
    env_name=${rust_target//-/_}
    linker="$llvm/${rust_target}26-clang"
    env "CARGO_TARGET_${env_name^^}_LINKER=$linker" "CC_${env_name}=$linker" "AR_${env_name}=$llvm/llvm-ar" \
        cargo build --release --locked --lib --target "$rust_target"
    library="${CARGO_TARGET_DIR:-target}/$rust_target/release/libwaldwinkel.so"
    mkdir -p "$assemblies/$abi"
    "$llvm/llvm-strip" --strip-unneeded -o "$assemblies/$abi/libwaldwinkel.so" "$library"
done

"$tools/aapt2" compile --dir packaging/res -o "$out/resources.zip"
"$tools/aapt2" link --manifest packaging/AndroidManifest.xml -I "$platform" -A assets \
    -o "$out/linked.apk" "$out/resources.zip"
java -m jdk.compiler/com.sun.tools.javac.Main -source 8 -target 8 -Xlint:-options -bootclasspath "$platform" \
    -d "$out/classes" packaging/java/game/waldwinkel/GameActivity.java
"$tools/d8" --min-api 26 --lib "$platform" --output "$out" "$out/classes/game/waldwinkel/GameActivity.class"

python3 - "$out" "${architectures[@]}" <<'PY'
import sys
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZIP_STORED, ZipFile

directory = Path(sys.argv[1])
with ZipFile(directory / "linked.apk", "a", compression=ZIP_STORED) as apk:
    apk.write(directory / "classes.dex", "classes.dex")
    for abi in sys.argv[2:]:
        apk.write(directory / "native" / abi / "libwaldwinkel.so",
                  f"lib/{abi}/libwaldwinkel.so", compress_type=ZIP_DEFLATED, compresslevel=6)
PY
"$tools/zipalign" -f -P 16 4 "$out/linked.apk" "$out/aligned.apk"
if [[ ! -f "$key" ]]; then
    keytool -genkeypair -keystore "$key" -storetype PKCS12 -alias waldwinkel-test \
        -storepass android -keypass android -keyalg RSA -keysize 2048 -validity 10000 \
        -dname 'CN=Waldwinkel Test, O=Waldwinkel'
fi
"$tools/apksigner" sign --ks "$key" --ks-key-alias waldwinkel-test --ks-pass pass:android \
    --v4-signing-enabled false --out "$apk" "$out/aligned.apk"
"$tools/apksigner" verify --verbose "$apk"
"$tools/zipalign" -c -P 16 4 "$apk"
printf 'APK: %s\n' "$apk"
