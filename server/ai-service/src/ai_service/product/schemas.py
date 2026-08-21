"""DTOs for the contract between ai-service and product-service.

The wire format follows the public endpoints described in
`server/ai-service/DESIGN.md` (`GET /products`, `GET /products/{id}`,
`GET /products/filter-options`). Until product-service ships these
endpoints, the client is exercised against stubs.
"""

from uuid import UUID

from pydantic import BaseModel, ConfigDict, Field


class ProductSummary(BaseModel):
    model_config = ConfigDict(extra="ignore")

    id: UUID
    name: str
    brand: str | None = None
    category: str | None = None
    price: int | None = None
    currency: str | None = None
    tags: list[str] = Field(default_factory=list)
    image_count: int = 0


class ProductDetails(BaseModel):
    model_config = ConfigDict(extra="ignore")

    id: UUID
    name: str
    brand: str | None = None
    category: str | None = None
    price: int | None = None
    currency: str | None = None
    description: str | None = None
    tags: list[str] = Field(default_factory=list)
    size: str | None = None
    condition: str | None = None
    image_count: int = 0


class FilterOptions(BaseModel):
    model_config = ConfigDict(extra="ignore")

    brands: list[str] = Field(default_factory=list)
    categories: list[str] = Field(default_factory=list)
    tags: list[str] = Field(default_factory=list)
    sizes: list[str] = Field(default_factory=list)
    conditions: list[str] = Field(default_factory=list)


class SearchFilters(BaseModel):
    model_config = ConfigDict(extra="ignore")

    category: str | None = None
    brand: str | None = None
    tags: list[str] = Field(default_factory=list)
    price_min: int | None = None
    price_max: int | None = None
    size: str | None = None
    condition: str | None = None

    def to_query(self) -> dict[str, str]:
        params: dict[str, str] = {}
        if self.category:
            params["category"] = self.category
        if self.brand:
            params["brand"] = self.brand
        if self.tags:
            params["tags"] = ",".join(self.tags)
        if self.price_min is not None:
            params["price_min"] = str(self.price_min)
        if self.price_max is not None:
            params["price_max"] = str(self.price_max)
        if self.size:
            params["size"] = self.size
        if self.condition:
            params["condition"] = self.condition
        return params


class ProductListResponse(BaseModel):
    model_config = ConfigDict(extra="ignore")

    items: list[ProductSummary] = Field(default_factory=list)


class ScoreBreakdown(BaseModel):
    model_config = ConfigDict(extra="ignore")

    text: float = 0.0
    image: float = 0.0
    best_image_idx: int | None = None


class SemanticSearchItem(BaseModel):
    model_config = ConfigDict(extra="ignore")

    id: UUID
    score: float
    score_breakdown: ScoreBreakdown = Field(default_factory=ScoreBreakdown)
    name: str
    brand: str | None = None
    category: str | None = None
    price: int | None = None
    highlight_fields: list[str] = Field(default_factory=list)


class SemanticSearchResponse(BaseModel):
    model_config = ConfigDict(extra="ignore")

    items: list[SemanticSearchItem] = Field(default_factory=list)


class SemanticSearchFilters(BaseModel):
    """Filters passed to product-service's /internal/semantic-search.

    Same shape as `SearchFilters` but serialised as a nested JSON object
    instead of query params, matching the internal endpoint contract.
    """

    model_config = ConfigDict(extra="ignore")

    category: str | None = None
    brand: str | None = None
    tags: list[str] = Field(default_factory=list)
    size: str | None = None
    condition: str | None = None
    price_min: int | None = None
    price_max: int | None = None
