#!/usr/bin/env python3
import sys
import re
import unicodedata
from pathlib import Path
import argparse
import shutil
import markdown
from datetime import datetime
import os

def parse_front_matter(md_text: str) -> tuple[dict[str, str], str]:
    lines = md_text.splitlines()
    meta: dict[str, str] = {}

    if lines and lines[0].strip() == "---":
        collected = []
        for i in range(1, len(lines)):
            if lines[i].strip() == "---":
                # done, return dict and rest
                for entry in collected:
                    if ":" in entry:
                        k, v = entry.split(":", 1)
                        meta[k.strip()] = v.strip()
                remaining = "\n".join(lines[i + 1 :])
                return meta, remaining
            collected.append(lines[i])
    # no front matter
    return {}, md_text

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
            "codehilite": {"guess_lang": False, "noclasses": False, "pygments_style": "default"},
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

def find_markdown_files(paths: list[Path]) -> list[Path]:
    files: list[Path] = []
    for p in paths:
        if p.is_file() and p.suffix.lower() == ".md":
            files.append(p)
        elif p.is_dir():
            files.extend(sorted(p.glob("*.md")))
    return files

def rewrite_image_srcs(html: str, md_dir: Path, out_dir: Path) -> str:
    # Repoint <img src="..."> to symlinks inside out_dir
    def repl(m: re.Match) -> str:
        prefix, src, suffix = m.group(1), m.group(2), m.group(3)
        if src.startswith(("http://", "https://", "data:")):
            return m.group(0)

        # Source file under md_dir
        target = (md_dir / src).resolve()
        assert target.exists() and target.is_file(), f"Image not found: {target}"

        # Place symlink inside out_dir with same relative path
        symlink_path = (out_dir / src).resolve()

        # Ensure subdirectories exist
        symlink_path.parent.mkdir(parents=True, exist_ok=True)

        # Remove stale symlink or file if present
        if symlink_path.exists() or symlink_path.is_symlink():
            symlink_path.unlink()

        # Create relative symlink (better portability)
        rel_target = os.path.relpath(target, symlink_path.parent)
        symlink_path.symlink_to(rel_target)

        # HTML should point to the symlink path relative to out_dir
        rel_html = os.path.relpath(symlink_path, out_dir)
        return f"{prefix}{rel_html}{suffix}"

    pattern = re.compile(r'(<img\b[^>]*?\bsrc=["\'])([^"\']+)(["\'])', flags=re.I)
    return pattern.sub(repl, html)

def main():
    parser = argparse.ArgumentParser(
        description="Build HTML pages from folders of Markdown into dist/ and create an index."
    )
    parser.add_argument("paths", nargs="+", type=Path, help="Folders or paths containing .md files (non-recursive)")
    args = parser.parse_args()

    template_path = Path("template.html")
    if not template_path.exists():
        print(f"Error: template not found: {template_path}", file=sys.stderr)
        sys.exit(1)

    md_files = find_markdown_files(args.paths)

    template = template_path.read_text(encoding="utf-8")
    dist_dir = Path("dist/")
    dist_dir.mkdir(parents=True, exist_ok=True)

    style_src = Path("style.css")
    if style_src.exists():
        shutil.copy2(style_src, dist_dir / "style.css")

    index_entries: list[tuple[str, str, str]] = []
    for md_path in md_files:
        md_text = md_path.read_text(encoding="utf-8")
        metadata, md_text = parse_front_matter(md_text)
        title = extract_title(md_text, fallback=md_path.stem)
        slug = slugify(title)

        out_dir = dist_dir / md_path.parent
        out_dir.mkdir(parents=True, exist_ok=True)

        body_html = convert_markdown(md_text)
        body_html = rewrite_image_srcs(body_html, md_path.parent, out_dir)
        final_html = render_with_template(template, title=title, body_html=body_html)

        out_file = out_dir / f"{slug}.html"
        out_file.write_text(final_html, encoding="utf-8")
        index_entries.append((title, out_file, metadata.get("date", None)))
        print(f"Converted {md_path} to {out_file}")

    parts = ['<div class="post-list">']
    for title, href, date in reversed(index_entries):
        parts.append(
            f'<div class="post-row">'
            f'  <span class="post-date">{date or ""}</span>'
            f'  <h1><a class="post-title" href="{href.relative_to(dist_dir)}">{title}</a></h1>'
            f'</div>'
        )
    parts.append('</div>')
    index_html = render_with_template(template, title="Index", body_html="\n".join(parts))
    index = dist_dir / "index.html"
    index.write_text(index_html, encoding="utf-8")
    print(f"Wrote index to {dist_dir / 'index.html'}")

if __name__ == "__main__":
    main()
