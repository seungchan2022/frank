// ApiClient 진입점 — 환경 변수로 Mock/Real 스위치.
//
// 사용법:
//   import { apiClient } from '$lib/api';
//   const articles = await apiClient.fetchArticles({ limit: 10 });
//
// Mock 활성화: VITE_USE_MOCK_API=true (web/.env.local 또는 환경변수)

import type { ApiClient } from './client';
import { mockApiClient } from './mockClient';
import { realApiClient } from './realClient';

const useMock = import.meta.env.VITE_USE_MOCK_API === 'true';

export const apiClient: ApiClient = useMock ? mockApiClient : realApiClient;

export type { ApiClient } from './client';
export type { Article, FeedItem, Profile, ProfilePatch, Tag, FetchArticlesOptions } from './types';
