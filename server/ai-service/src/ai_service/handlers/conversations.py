from uuid import UUID

from fastapi import APIRouter, Depends, Response, status
from pydantic import BaseModel

from ..auth import User, current_user
from ..deps import get_store
from ..domain.conversation import ConversationStore

router = APIRouter(prefix="/api/stylist/conversations", tags=["stylist"])

_user_dep = Depends(current_user)
_store_dep = Depends(get_store)


class CreateResponse(BaseModel):
    conversation_id: UUID


@router.post("", response_model=CreateResponse, status_code=status.HTTP_201_CREATED)
async def create_conversation(
    user: User = _user_dep,
    store: ConversationStore = _store_dep,
) -> CreateResponse:
    conv = await store.create(user.id)
    return CreateResponse(conversation_id=conv.id)


@router.delete("/{conversation_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_conversation(
    conversation_id: UUID,
    user: User = _user_dep,
    store: ConversationStore = _store_dep,
) -> Response:
    await store.delete(conversation_id, user.id)
    return Response(status_code=status.HTTP_204_NO_CONTENT)
