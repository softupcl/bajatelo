#!/usr/bin/env python3
"""Genera THIRD-PARTY-NOTICES.md a partir de los lockfiles reales.

Fuente de verdad: `src-tauri/Cargo.lock` (crates embebidos en el binario) y
`package.json` → "dependencies" (JS de runtime embebido en el bundle).
Las licencias se leen del registro local del toolchain (CARGO_HOME/registry)
y de node_modules, sin descargar nada.

Uso: python3 scripts/third-party-notices.py
"""

import json
import os
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CARGO_HOME = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
OUT = REPO / "THIRD-PARTY-NOTICES.md"

# El propio proyecto no se lista.
CARGO_SKIP = {
    "bajatelo",
}


def parse_cargo_lock() -> list[dict]:
    """Parser mínimo de Cargo.lock (TOML): solo los bloques [[package]]."""
    pkgs: list[dict] = []
    current: dict = {}
    in_registry_blocks = False
    for line in (REPO / "src-tauri" / "Cargo.lock").read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped == "[[package]]":
            if current:
                pkgs.append(current)
            current = {}
            in_registry_blocks = True
        elif in_registry_blocks:
            if stripped.startswith("[") and stripped.endswith("]"):
                in_registry_blocks = False  # fin de [[package]]
            elif "=" in stripped:
                key, _, value = stripped.partition("=")
                key = key.strip()
                value = value.strip().strip('"').strip("'")
                if key == "name":
                    current["name"] = value
                elif key == "version":
                    # la versión puede tener sufijo de rama (p. ej. "1.0#ref")
                    current["version"] = value.split("#")[0].strip()
                elif key == "source":
                    current["source"] = value
    if current:
        pkgs.append(current)
    return pkgs


def cargo_license(name: str, version: str) -> str:
    for registry in (CARGO_HOME / "registry").glob("src/*/"):
        candidate = registry / f"{name}-{version}" / "Cargo.toml"
        if candidate.is_file():
            try:
                for line in candidate.read_text(encoding="utf-8", errors="replace").splitlines():
                    if line.strip().startswith("license ") or line.strip().startswith("license="):
                        value = line.split("=", 1)[1].strip().strip('"').strip("'")
                        return value if value else "desconocida"
            except OSError:
                pass
    return "desconocida (sin fuente en el registro local)"


def main() -> int:
    lines = [
        "# Bajatelo — Avisos de terceros (THIRD-PARTY-NOTICES)",
        "",
        "> Generado por `scripts/third-party-notices.py` desde `src-tauri/Cargo.lock` y"
        " `package.json`. No se redistribuye yt-dlp ni FFmpeg (decisión D-03 del"
        " proyecto: se instalan por el usuario y solo se localizan en el PATH).",
        "",
        "## Crates Rust presentes en el binario (`Cargo.lock`)",
        "",
    ]

    pkgs = parse_cargo_lock()
    rows = []
    for pkg in pkgs:
        name = pkg.get("name", "")
        if name in CARGO_SKIP:
            continue
        version = pkg.get("version", "")
        source = pkg.get("source", "")
        lic = cargo_license(name, version) if source.startswith("registry+") else "local"
        rows.append((name, version, lic))
    rows.sort(key=lambda r: r[0].lower())
    lines += [
        "| crate | versión | licencia declarada |",
        "|---|---|---|",
    ]
    lines += [f"| {n} | {v} | {l} |" for n, v, l in rows]

    lines += [
        "",
        "Los archivos de licencia y copyright completos de cada crate acompañan a su",
        "código fuente en el registro del toolchain con el que se compiló el binario",
        "(CARGO_HOME/registry). Las licencias MIT/Apache-2.0/ISC exigen conservar",
        "los avisos correspondientes de cada componente embebido; este fichero",
        "cumple esa obligación redistribuida por la app.",
        "",
        "## JavaScript de runtime embebido en el bundle (`package.json`)",
        "",
        "| paquete | licencia declarada |",
        "|---|---|",
    ]

    for dep in sorted(json.loads((REPO / "package.json").read_text())["dependencies"]):
        pkg_file = REPO / "node_modules" / dep / "package.json"
        try:
            data = json.loads(pkg_file.read_text(encoding="utf-8"))
        except OSError:
            lines.append(f"| {dep} | — (instalar `npm install` para leerla) |")
            continue
        lic = data.get("license", "desconocida")
        version = data.get("version", "?")
        lines.append(f"| {dep}@{version} | {lic} |")

    lines += [
        "",
        "## No redistribuidos (instalados por el usuario; la app solo los detecta)",
        "",
        "| herramienta | licencia | obligación del clon |",
        "|---|---|---|",
        "| yt-dlp | Unlicense (dominio público) | ninguna (no se redistribuye) |",
        "| FFmpeg/FFprobe | LGPL/GPL según el build del usuario | ninguna (no se redistribuye) |",
        "| SQLite (rusqlite `bundled`) | SQLite Public Domain alternativa | conserva el aviso de copyright en el código fuente del proyecto |",
        "",
    ]
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"escrito {OUT} ({len(rows)} crates)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
