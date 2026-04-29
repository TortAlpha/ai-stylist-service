import { CategoryFullResponse } from '../../core/models/category.model';

function normalizeCategoryToken(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, '');
}

export function isRedundantTypeRoot(
  category: CategoryFullResponse,
  typeKey: string,
): boolean {
  return (
    category.parent_id === null &&
    category.product_type === typeKey &&
    normalizeCategoryToken(category.name) === normalizeCategoryToken(typeKey)
  );
}

export function pathWithoutRedundantTypeRoot(path: string[], typeKey: string): string[] {
  return path.length > 0 && normalizeCategoryToken(path[0]) === normalizeCategoryToken(typeKey)
    ? path.slice(1)
    : path;
}
