#!/usr/bin/env python3
import sys
import re
import unicodedata
from pathlib import Path
import argparse
import shutil
import markdown
import os

def read_text(p: Path) -> str:
    return p.read_text(encoding="utf-8")

def write_text(p: Path, s: str) -> None:
    p.write_text(s, encoding="utf-8")

def extract_title(md_text: str, fallback: str) -> str:
    for line in md_text.splitlines():
        m = re.match(r"^\s*#\s+(.+?)\s*$", line)
        if m:
            return m.group(1).strip()
    return fallback

def convert_markdown(md_text: str) -> str:
    exts = ["fenced_code", "codehilite", "tables", "attr_list", "md_in_html", "toc"]
    md = markdown.Markdown(
        extensions=exts,
        extension_configs={
            "codehilite": {"guess_lang": True, "noclasses": False, "pygments_style": "default"},
            "toc": {"permalink": False, "anchorlink": True, "title": "Table of contents"},
        },
        output_format="html5",
    )
    return md.convert(md_text)

def slugify(text: str) -> str:
    text = unicodedata.normalize("NFKD", text)
    text = text.encode("ascii", "ignore").decode("ascii")
    text = re.sub(r"[^a-zA-Z0-9]+", "-", text).strip("-").lower()
    return text or "page"

def render_with_template(template: str, title: str, body_html: str) -> str:
    return (
        template
        .replace("{{ title }}", title)
        .replace("{{ content }}", body_html)
    )

def find_markdown_files(dirs: list[Path]) -> list[Path]:
    files: list[Path] = []
    for d in dirs:
        files.extend(sorted(d.glob("*.md")))
    return files

def rewrite_image_srcs(html: str, md_dir: Path, out_dir: Path) -> str:
    # Repoint <img src="..."> to paths relative to dist/, targeting the original files under md_dir
    def repl(m: re.Match) -> str:
        prefix, src, suffix = m.group(1), m.group(2), m.group(3)
        if src.startswith(("http://", "https://", "data:")):
            return m.group(0)
        target = (md_dir / src).resolve()
        assert(target.exists() and target.is_file())
        rel = os.path.relpath(target, out_dir)
        return f'{prefix}{rel}{suffix}'

    pattern = re.compile(r'(<img\b[^>]*?\bsrc=["\'])([^"\']+)(["\'])', flags=re.I)
    return pattern.sub(repl, html)

def main():
    parser = argparse.ArgumentParser(
        description="Build HTML pages from folders of Markdown into dist/ and create an index."
    )
    parser.add_argument("folders", nargs="+", type=Path, help="Folders containing .md files (non-recursive)")
    args = parser.parse_args()

    template_path = Path("template.html")
    if not template_path.exists():
        print(f"Error: template not found: {template_path}", file=sys.stderr)
        sys.exit(1)

    md_files = find_markdown_files(args.folders)
    if not md_files:
        print("No .md files found in provided folders.", file=sys.stderr)
        sys.exit(1)

    template = read_text(template_path)
    out_dir = Path("dist")
    out_dir.mkdir(parents=True, exist_ok=True)

    style_src = Path("style.css")
    if style_src.exists():
        shutil.copy2(style_src, out_dir / "style.css")

    index_entries: list[tuple[str, str]] = []

    for md_path in md_files:
        md_text = read_text(md_path)
        title = extract_title(md_text, fallback=md_path.stem)
        slug = slugify(title)

        body_html = convert_markdown(md_text)
        body_html = rewrite_image_srcs(body_html, md_path.parent, out_dir)
        final_html = render_with_template(template, title=title, body_html=body_html)

        out_file = out_dir / f"{slug}.html"
        write_text(out_file, final_html)
        index_entries.append((title, out_file.name))
        print(f"Converted {md_path} to {out_file}")

    parts = []
    for title, href in index_entries:
        parts.append(f'<b><a href="{href}">{title}</a></b><br><br>')
    index_html = render_with_template(template, title="Index", body_html="\n".join(parts))
    write_text(out_dir / "index.html", index_html)
    print(f"Wrote index to {out_dir / 'index.html'}")

if __name__ == "__main__":
    main()
