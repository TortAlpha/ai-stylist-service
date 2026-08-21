from uuid import uuid4

from fastapi.testclient import TestClient


def test_whoami_requires_user_headers(client: TestClient) -> None:
    resp = client.get("/api/stylist/whoami")
    assert resp.status_code == 401


def test_whoami_rejects_invalid_uuid(client: TestClient) -> None:
    resp = client.get(
        "/api/stylist/whoami",
        headers={"X-User-Id": "not-a-uuid", "X-User-Role": "customer"},
    )
    assert resp.status_code == 401


def test_whoami_ok(client: TestClient) -> None:
    user_id = str(uuid4())
    resp = client.get(
        "/api/stylist/whoami",
        headers={"X-User-Id": user_id, "X-User-Role": "customer"},
    )
    assert resp.status_code == 200
    assert resp.json() == {"user_id": user_id, "role": "customer"}
