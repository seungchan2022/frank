//! MVP15 M1 진단 바이너리 모듈 루트.
//!
//! 책임: 활성 태그 × 5변형 × 2엔진 측정 → `progress/mvp15/M1_diagnosis_data.md` 생성.
//! 운영 서버 코드와 분리. 공용 builder/factory(`SearchFallbackChain`, `feed_cache` 등) 미경유.

pub mod env_loader;
pub mod mapping_resolver;
pub mod markdown_formatter;
pub mod retry_classifier;
pub mod runner;
pub mod variant_generator;
