"""Check local inline Markdown links; external URLs and heading fragments are excluded."""
import re
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]

def check() -> list[str]:
    failures: list[str] = []
    documents = sorted(ROOT.rglob("*.md"))
    for document in documents:
        if any(part in {"target", ".venv", ".git", "node_modules", "test-results", "playwright-report"} for part in document.relative_to(ROOT).parts):
            continue
        text = re.sub(r"```[^\n]*\n.*?```", "", document.read_text(), flags=re.S)
        for href in re.findall(r"!?\[[^\]]*\]\(([^)]+)\)", text):
            parsed = urlsplit(href)
            if parsed.scheme or href.startswith("#"):
                continue
            target = (document.parent / unquote(parsed.path)).resolve()
            if not target.is_relative_to(ROOT) or not target.exists():
                failures.append(f"{document.relative_to(ROOT)}: {href}")
    return failures

if __name__ == "__main__":
    errors = check()
    if errors:
        raise SystemExit("Broken local links:\n" + "\n".join(errors))
    print("Local inline Markdown links passed (external URLs/fragments not checked).")
