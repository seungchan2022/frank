//! 5변형 쿼리 생성기.
//!
//! SSOT: `progress/mvp15/M1_search_diagnosis.md` "변형 셋(5개)".
//! 1) 운영 baseline `"{tag} latest news"`
//! 2) 단순 `"{tag}"`
//! 3) 시간 `"{tag} 2026"`
//! 4) 관점 `"{tag} announcement"`
//! 5) 영어 번역 `"{eng} latest news"` (한국어 약점 가설 검증용)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantKind {
    Baseline,
    Simple,
    Year2026,
    Announcement,
    EnglishBaseline,
}

impl VariantKind {
    pub fn as_label(self) -> &'static str {
        match self {
            VariantKind::Baseline => "baseline",
            VariantKind::Simple => "simple",
            VariantKind::Year2026 => "year2026",
            VariantKind::Announcement => "announcement",
            VariantKind::EnglishBaseline => "english_baseline",
        }
    }

    pub fn all() -> [VariantKind; 5] {
        [
            VariantKind::Baseline,
            VariantKind::Simple,
            VariantKind::Year2026,
            VariantKind::Announcement,
            VariantKind::EnglishBaseline,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    pub kind: VariantKind,
    pub query: String,
}

/// 한국어 태그 + 영어 번역으로부터 5개 변형을 생성.
pub fn generate(korean_tag: &str, english_translation: &str) -> Vec<Variant> {
    VariantKind::all()
        .into_iter()
        .map(|kind| {
            let query = match kind {
                VariantKind::Baseline => format!("{korean_tag} latest news"),
                VariantKind::Simple => korean_tag.to_string(),
                VariantKind::Year2026 => format!("{korean_tag} 2026"),
                VariantKind::Announcement => format!("{korean_tag} announcement"),
                VariantKind::EnglishBaseline => format!("{english_translation} latest news"),
            };
            Variant { kind, query }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-02: 각 태그 × 5변형 정확
    #[test]
    fn five_variants_for_aiml() {
        let v = generate("AI/ML", "AI/ML");
        assert_eq!(v.len(), 5);
        assert_eq!(v[0].kind, VariantKind::Baseline);
        assert_eq!(v[0].query, "AI/ML latest news");
        assert_eq!(v[1].query, "AI/ML");
        assert_eq!(v[2].query, "AI/ML 2026");
        assert_eq!(v[3].query, "AI/ML announcement");
        assert_eq!(v[4].query, "AI/ML latest news");
    }

    #[test]
    fn english_baseline_uses_translation() {
        let v = generate("모바일 개발", "Mobile Development");
        assert_eq!(v[4].kind, VariantKind::EnglishBaseline);
        assert_eq!(v[4].query, "Mobile Development latest news");
        // simple variant remains korean
        assert_eq!(v[1].query, "모바일 개발");
    }

    #[test]
    fn variant_kinds_are_distinct_labels() {
        let labels: Vec<&str> = VariantKind::all().iter().map(|k| k.as_label()).collect();
        assert_eq!(
            labels,
            vec![
                "baseline",
                "simple",
                "year2026",
                "announcement",
                "english_baseline"
            ]
        );
    }
}
