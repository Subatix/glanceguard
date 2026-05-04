#!/usr/bin/env bash
# Downloads InsightFace buffalo_s detector + MobileFaceNet embedder ONNX files
# into src-tauri/models/ (paths must match models/scrfd.json and models/arcface.json).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="${ROOT}/src-tauri/models"
ZIP_URL="${BUFFALO_S_ZIP_URL:-https://github.com/deepinsight/insightface/releases/download/v0.7/buffalo_s.zip}"
TMP="$(mktemp -d)"
cleanup() { rm -rf "${TMP}"; }
trap cleanup EXIT

curl -fsSL -o "${TMP}/buffalo_s.zip" "${ZIP_URL}"
unzip -o -q -j "${TMP}/buffalo_s.zip" det_500m.onnx w600k_mbf.onnx -d "${DEST}"
echo "Wrote ${DEST}/det_500m.onnx and ${DEST}/w600k_mbf.onnx"
