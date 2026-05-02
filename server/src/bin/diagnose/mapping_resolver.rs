//! 한국어 태그 → 영어 직역 매핑.
//!
//! SSOT: `progress/mvp15/M1_search_diagnosis.md` "영어 번역 매핑" 표.
//! 12개 활성 태그를 사전 등록. 신규 태그가 들어오면 `resolve` 가 `Err(MappingError::Missing)` 반환.

use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MappingError {
    #[error("영어 매핑 누락 태그: {0}")]
    Missing(String),
}

/// 한국어 태그 이름 → 영어 번역. SSOT 표 그대로.
static MAPPING: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let pairs: &[(&str, &str)] = &[
        ("AI/ML", "AI/ML"),
        ("모바일 개발", "Mobile Development"),
        ("UX/디자인", "UX/Design"),
        ("데이터 사이언스", "Data Science"),
        ("보안", "Security"),
        ("블록체인", "Blockchain"),
        ("스타트업", "Startup"),
        ("오픈소스", "Open Source"),
        ("웹 개발", "Web Development"),
        ("클라우드/인프라", "Cloud/Infrastructure"),
        ("투자/VC", "Investment/VC"),
        ("프로덕트", "Product"),
    ];
    pairs.iter().copied().collect()
});

/// 한국어 태그 이름 → 영어 직역.
pub fn resolve(korean: &str) -> Result<&'static str, MappingError> {
    MAPPING
        .get(korean)
        .copied()
        .ok_or_else(|| MappingError::Missing(korean.to_string()))
}

/// 등록된 한국어 → 영어 매핑 전체. data.md "매핑 표" 출력용.
pub fn all_pairs() -> Vec<(&'static str, &'static str)> {
    let mut v: Vec<_> = MAPPING.iter().map(|(k, v)| (*k, *v)).collect();
    v.sort_by_key(|(k, _)| *k);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-01 단위: 12개 태그 매핑 존재
    #[test]
    fn all_twelve_korean_tags_resolve() {
        let cases = [
            ("AI/ML", "AI/ML"),
            ("모바일 개발", "Mobile Development"),
            ("UX/디자인", "UX/Design"),
            ("데이터 사이언스", "Data Science"),
            ("보안", "Security"),
            ("블록체인", "Blockchain"),
            ("스타트업", "Startup"),
            ("오픈소스", "Open Source"),
            ("웹 개발", "Web Development"),
            ("클라우드/인프라", "Cloud/Infrastructure"),
            ("투자/VC", "Investment/VC"),
            ("프로덕트", "Product"),
        ];
        for (ko, en) in cases {
            assert_eq!(resolve(ko).unwrap(), en, "{ko} 매핑 실패");
        }
    }

    // E-02 누락 감지
    #[test]
    fn missing_tag_returns_error() {
        let err = resolve("미등록태그").unwrap_err();
        assert_eq!(err, MappingError::Missing("미등록태그".to_string()));
    }

    #[test]
    fn all_pairs_returns_twelve() {
        assert_eq!(all_pairs().len(), 12);
    }
}
