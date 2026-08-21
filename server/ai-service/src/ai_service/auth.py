from uuid import UUID

from fastapi import Header, HTTPException, status
from pydantic import BaseModel


class User(BaseModel):
    id: UUID
    role: str


async def current_user(
    x_user_id: str | None = Header(default=None, alias="X-User-Id"),
    x_user_role: str | None = Header(default=None, alias="X-User-Role"),
) -> User:
    if not x_user_id or not x_user_role:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="missing X-User-Id or X-User-Role",
        )
    try:
        user_id = UUID(x_user_id)
    except ValueError as exc:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="invalid X-User-Id",
        ) from exc
    return User(id=user_id, role=x_user_role)
