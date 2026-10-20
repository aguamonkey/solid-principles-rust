"""Run a disposable journal for browser tests; stop only the owned server."""
import signal
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def stop(_signum, _frame):
    raise SystemExit(0)

signal.signal(signal.SIGTERM, stop)
with tempfile.TemporaryDirectory(prefix="rust-console-browser-") as directory:
    child = subprocess.Popen([
        str(ROOT / "target/debug/release-console"), "--port", "5191",
        "--database", str(Path(directory) / "test.sqlite3"), "--markdown",
    ])
    try:
        raise SystemExit(child.wait())
    finally:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
