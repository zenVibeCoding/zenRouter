from fastapi.testclient import TestClient

from zenrouter_adapter import CONTRACT_VERSION
from zenrouter_adapter.app import CONTRACT_HEADER, create_app

client = TestClient(create_app())


def test_healthz_reports_contract_version():
    resp = client.get("/healthz", headers={CONTRACT_HEADER: CONTRACT_VERSION})
    assert resp.status_code == 200
    body = resp.json()
    assert body["status"] == "ok"
    assert body["contract_version"] == CONTRACT_VERSION
    assert "contract_mismatch" not in body


def test_healthz_flags_contract_mismatch():
    resp = client.get("/healthz", headers={CONTRACT_HEADER: "999"})
    assert resp.json()["contract_mismatch"] is True


def test_attempts_not_implemented():
    assert client.post("/v1/attempts", json={}).status_code == 501


def test_docs_are_disabled():
    assert client.get("/docs").status_code == 404
    assert client.get("/openapi.json").status_code == 404


def test_contract_version_matches_rust():
    from pathlib import Path

    rust = Path(__file__).resolve().parents[3] / "crates/contracts/src/lib.rs"
    assert f'ADAPTER_CONTRACT_VERSION: &str = "{CONTRACT_VERSION}"' in rust.read_text()
