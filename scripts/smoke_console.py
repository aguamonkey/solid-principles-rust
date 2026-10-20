"""Real HTTP + process restart checks, isolated from development data."""
import json
import socket
import subprocess
import tempfile
import time
from contextlib import contextmanager
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target/debug/release-console"

def request(base, path, body=None, key=None):
    headers = {"Content-Type": "application/json"}
    if key:
        headers["Idempotency-Key"] = key
    data = None if body is None else json.dumps(body).encode()
    try:
        with urlopen(Request(base + path, data=data, headers=headers), timeout=5) as response:
            content = response.read()
            try:
                content = json.loads(content)
            except ValueError:
                pass
            return response.status, content
    except HTTPError as error:
        return error.code, error.read()

@contextmanager
def server(port, storage):
    child = subprocess.Popen([str(BINARY), "--port", str(port), *storage], stdout=subprocess.DEVNULL)
    base = f"http://127.0.0.1:{port}"
    try:
        for _ in range(100):
            if child.poll() is not None:
                raise RuntimeError("Console exited before readiness")
            try:
                if request(base, "/api/ready")[0] == 200:
                    break
            except (URLError, OSError):
                pass
            time.sleep(0.05)
        else:
            raise RuntimeError("Console did not become ready")
        yield base
    finally:
        child.terminate()
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()

invalid = subprocess.run([str(BINARY), "--memory", "--database", "unused.sqlite3"], capture_output=True)
assert invalid.returncode != 0 and b"not both" in invalid.stderr

payload = {"identity": "smoke-build", "project": "HTTP smoke", "duration_ms": 420, "artifacts": ["smoke.bin"]}
with socket.socket() as probe:
    probe.bind(("127.0.0.1", 0))
    port = probe.getsockname()[1]
with tempfile.TemporaryDirectory(prefix="rust-console-smoke-") as directory:
    storage = ["--database", str(Path(directory) / "journal.sqlite3")]
    with server(port, storage) as base:
        assert request(base, "/")[0] == 200
        assert request(base, "/api/builds", payload, "key")[0] == 201
        assert request(base, "/api/builds", payload, "key")[0] == 200
        assert request(base, "/api/builds", {**payload, "duration_ms": 999}, "key")[0] == 409
    with server(port, storage) as base:
        assert request(base, "/api/builds/smoke-build")[1]["candidate"] == payload
        assert request(base, "/api/builds", payload, "key")[0] == 200
        assert request(base, "/api/releases/process", {})[1]["outcome"] == "confirmed"
    with server(port, storage) as base:
        assert request(base, "/api/releases/process", {})[1]["outcome"] == "empty"
        assert len(request(base, "/api/reports/json")[1]) == 1
    with server(port, ["--memory"]) as base:
        assert request(base, "/api/builds", payload, "key")[0] == 201
    with server(port, ["--memory"]) as base:
        assert request(base, "/api/builds")[1]["builds"] == []
print("HTTP + SQLite restart/replay/receipt persistence + ephemeral memory checks passed.")
