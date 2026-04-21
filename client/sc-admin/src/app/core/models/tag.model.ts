export interface StyleTag {
  id: number;
  name: string;
}

export interface VibeTag {
  id: number;
  name: string;
}

export interface Season {
  id: number;
  name: string;
}

export interface CreateTagRequest {
  name: string;
}

export interface SetTagIdsRequest {
  ids: number[];
}
