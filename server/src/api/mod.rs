pub mod health;
pub mod tags;

use crate::domain::ports::DbPort;

#[derive(Debug, Clone)]
pub struct AppState<D: DbPort> {
    pub db: D,
}
