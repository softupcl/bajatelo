#!/usr/bin/env bash
# Descarga los motores fijados (D-09 opción C) por plataforma:
#   scripts/fetch-engines.sh darwin-aarch64 | windows-x86_64 | linux-x86_64 | linux-aarch64
#
# Coloca yt-dlp/ffmpeg/ffprobe en src-tauri/engines/<plataforma>/. Integridad:
#   - yt-dlp: verificado contra el SHA2-256SUMS OFICIAL del release fijado.
#   - ffmpeg/ffprobe macOS: SHA256 publicado por el proveedor (osxexperts).
#   - win/linux (BtbN, etiqueta fechada): el SHA real calculado se registra.
# Idempotente: los binarios en caché con hash correcto no se re-descargan.
# Portabilidad: usa curl + python3 (zipfile/tarfile) presentes en los runners.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST_ROOT="${REPO_ROOT}/src-tauri/engines"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

YTDLP_VERSION="2026.08.19"
YTDLP_BASE="https://github.com/yt-dlp/yt-dlp/releases/download/${YTDLP_VERSION}"
# FFmpeg línea 9.0.2 — BtbN autobuild fechada (misma línea que Homebrew 9.0.2):
FFMPEG_WIN_URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-10-19-13-06/ffmpeg-n9.0.2-28-g669f5ab083-win64-gpl-9.0.zip"
FFMPEG_LINUX_URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-10-19-13-06/ffmpeg-n9.0.2-28-g669f5ab083-linux64-gpl-9.0.tar.xz"
# macOS arm64 estático (SHA256 del binario interno publicados por el proveedor):
FFMPEG_MAC_URL="https://www.osxexperts.net/ffmpeg9arm.zip"
FFPROBE_MAC_URL="https://www.osxexperts.net/ffprobe9arm.zip"
FFMPEG_MAC_SHA="591260c945d0eef150e3bf82b0ef988bd36a9cecc18ff05d6679617159f0a95e"
FFPROBE_MAC_SHA="e11c17e8200b3ee4c4c186d245e2b4053f01d56957336c1817fca0b997469106"

PLATFORM="${1:?plataforma requerida}"
DEST="${DEST_ROOT}/${PLATFORM}"
mkdir -p "${DEST}"

dl() { curl -L --fail --silent --show-error --retry 3 -o "$2" "$1"; }

sha256() {
	local py
	py="$(command -v python3 || command -v python)"
	"$py" -c 'import hashlib,sys; f=open(sys.argv[1],"rb"); h=hashlib.sha256(); h.update(f.read()); print(h.hexdigest())' "$1"
}

extract() {
	# extract <archivo> <destino> (zip / tar.xz — con python3|python)
	local py
	py="$(command -v python3 || command -v python)"
	"$py" - "$1" "$2" <<'PY'
import sys, zipfile, tarfile
src, dst = sys.argv[1], sys.argv[2]
if zipfile.is_zipfile(src):
    zipfile.ZipFile(src).extractall(dst)
elif tarfile.is_tarfile(src):
    tarfile.open(src).extractall(dst)
else:
    sys.exit(f"formato desconocido: {src}")
PY
}

local_ytdlp_expected() {
	local sums_file="$TMP_ROOT/SHA2-256SUMS"
	[ -f "$sums_file" ] || dl "${YTDLP_BASE}/SHA2-256SUMS" "$sums_file"
	grep -E "^([0-9a-f]{64})  ${1}\$" "$sums_file" | awk '{print $1}'
}

fetch_ytdlp() {
	# En Windows los binarios conservan su .exe (CreateProcess no ejecuta sin extensión).
	local asset="$1"
	local file_name="yt-dlp"
	[ "${asset}" = "yt-dlp.exe" ] && file_name="yt-dlp.exe"
	local file="${DEST}/${file_name}"
	local expected
	expected="$(local_ytdlp_expected "${asset}")"
	[ -n "${expected}" ] || { echo "asset ${asset} sin hash oficial" >&2; exit 1; }
	if [ -f "${file}" ] && [ "$(sha256 "${file}")" = "${expected}" ]; then
		echo "yt-dlp en caché (${YTDLP_VERSION}) ok"
		chmod +x "${file}"
		return
	fi
	# yt-dlp_macos se distribuye como zip con el binario dentro.
	case "${asset}" in
		*.zip)
			dl "${YTDLP_BASE}/${asset}" "${TMP_ROOT}/ytdlp.zip"
			extract "${TMP_ROOT}/ytdlp.zip" "${TMP_ROOT}/ytdlp-zip"
			mv "${TMP_ROOT}/ytdlp-zip/macos" "${file}"
			;;
		*) dl "${YTDLP_BASE}/${asset}" "${file}" ;;
	esac
	chmod +x "${file}"
	local real
	real="$(sha256 "${file}")"
	[ "${real}" = "${expected}" ] || { echo "SHA-256 NO coincide para ${asset}: ${real} != ${expected}" >&2; exit 1; }
	echo "yt-dlp ${YTDLP_VERSION} ok (${real})"
}

fetch_mac_static() {
	# fetch_mac_static <url> <sha-esperado> <nombre>
	local url="$1"
	local expected="$2"
	local name="$3"
	local file="${DEST}/${name}"
	if [ -f "${file}" ] && [ "$(sha256 "${file}")" = "${expected}" ]; then
		echo "${name} en caché ok"
		chmod +x "${file}"
		return
	fi
	dl "${url}" "${TMP_ROOT}/${name}.zip"
	extract "${TMP_ROOT}/${name}.zip" "${TMP_ROOT}/${name}-x"
	local bin
	bin="$(find "${TMP_ROOT}/${name}-x" -type f -name "${name}" ! -name '*.py' | head -1)"
	[ -n "${bin}" ] || bin="$(find "${TMP_ROOT}/${name}-x" -type f ! -name '*.*' | head -1)"
	[ -n "${bin}" ] || { echo "no se encontró ${name} en el zip" >&2; exit 1; }
	mv "${bin}" "${file}"
	chmod +x "${file}"
	local real
	real="$(sha256 "${file}")"
	[ "${real}" = "${expected}" ] || { echo "SHA-256 NO coincide para ${name}: ${real} != ${expected}" >&2; exit 1; }
	echo "${name} ok (${real})"
}

fetch_ffmpeg_bundle() {
	# fetch_ffmpeg_bundle <url> <zip|tar.xz> — extrae ffmpeg+ffprobe de BtbN.
	# NOTA Windows: los .exe se conservan con su extensión (los binarios nativos
	# de Windows deben llamarse *.exe para poder ejecutarse).
	local url="$1"
	local kind="$2"
	local archive="${TMP_ROOT}/ffmpeg-bundle.${kind}"
	local suffix=""
	[ "${kind}" = "zip" ] && suffix=".exe"
	if [ -f "${DEST}/ffmpeg${suffix}" ] && [ -f "${DEST}/ffprobe${suffix}" ]; then
		echo "ffmpeg/ffprobe ya existen (caché, sin re-descarga)"
		return
	fi
	dl "${url}" "${archive}"
	extract "${archive}" "${TMP_ROOT}/bundle-x"
	local root
	root="$(find "${TMP_ROOT}/bundle-x" -maxdepth 1 -type d -name 'ffmpeg-*' | head -1)"
	[ -n "${root}" ] || { echo "no se encontró la raíz ffmpeg-*" >&2; exit 1; }
	for tool in ffmpeg ffprobe; do
		local src_bin
		src_bin="$(find "${root}/bin" -maxdepth 2 -type f -iname "${tool}*" ! -iname "*shared*" | sort | head -1)"
		[ -n "${src_bin}" ] || { echo "no se encontró ${tool}" >&2; exit 1; }
		mv "${src_bin}" "${DEST}/${tool}${suffix}"
		echo "${tool}${suffix}: $(sha256 "${DEST}/${tool}${suffix}")"
	done
}

case "${PLATFORM}" in
	darwin-aarch64)
		fetch_ytdlp "yt-dlp_macos"
		fetch_mac_static "${FFMPEG_MAC_URL}" "${FFMPEG_MAC_SHA}" "ffmpeg"
		fetch_mac_static "${FFPROBE_MAC_URL}" "${FFPROBE_MAC_SHA}" "ffprobe"
		;;
	windows-x86_64)
		fetch_ytdlp "yt-dlp.exe"
		fetch_ffmpeg_bundle "${FFMPEG_WIN_URL}" "zip"
		;;
	linux-x86_64)
		fetch_ytdlp "yt-dlp_linux"
		fetch_ffmpeg_bundle "${FFMPEG_LINUX_URL}" "tar.xz"
		;;
	linux-aarch64)
		fetch_ytdlp "yt-dlp_linux_aarch64"
		# FFmpeg arm64 en Linux: se resuelve con el paquete del SO (degradación RF-08).
		;;
	*)
		echo "plataforma desconocida: ${PLATFORM}" >&2
		exit 1
		;;
esac

{
	echo "manifest=${PLATFORM}"
	echo "ytdlp=${YTDLP_VERSION}"
	echo "yt-dlp: https://github.com/yt-dlp/yt-dlp (Unlicense)"
	echo "ffmpeg: BtbN autobuild-2026-10-19-13-06 (win/linux, GPL) · osxexperts 9.0 (darwin, GPL; fuente: https://github.com/FFmpeg/FFmpeg)"
} > "${DEST}/MANIFEST.txt"

echo "engines listas en ${DEST}"
