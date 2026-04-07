// DEPRECATED: 이 파일은 backwards-compat shim이다.
// 새 코드는 `$lib/api`에서 `apiClient`를 import할 것.
//
// M1.5에서 ApiClient 인터페이스로 추상화 + Mock/Real 분리됐다.
// M2(웹 전환) 완료 후 콜사이트들이 모두 `apiClient.foo()` 형태로 마이그레이션되면 본 파일 제거.

import { apiClient } from '$lib/api';
import { supabase } from '$lib/supabase';

export const fetchTags = () => apiClient.fetchTags();
export const fetchMyTagIds = () => apiClient.fetchMyTagIds();
export const saveMyTags = (tagIds: string[]) => apiClient.saveMyTags(tagIds);
export const updateMyTags = (tagIds: string[]) => apiClient.updateMyTags(tagIds);
export const fetchProfile = () => apiClient.fetchProfile();
export const fetchArticles = (offset = 0, limit = 10, tagId?: string) =>
	apiClient.fetchArticles({ offset, limit, tagId });
export const fetchArticleById = (id: string) => apiClient.fetchArticleById(id);
export const collectArticles = () => apiClient.collectArticles();
export const summarizeArticles = () => apiClient.summarizeArticles();

// getAuthHeaders는 Rust 서버 프록시용으로 일부 라우트에서 직접 사용 중
export async function getAuthHeaders(): Promise<Record<string, string>> {
	const {
		data: { session }
	} = await supabase.auth.getSession();
	if (!session) throw new Error('Not authenticated');
	return {
		Authorization: `Bearer ${session.access_token}`,
		'Content-Type': 'application/json'
	};
}
