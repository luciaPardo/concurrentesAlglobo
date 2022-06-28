use serde::Deserialize;
use serde::Serialize;
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Serialize)]
pub struct AlgloboTransaction {
    pub id: u32,
    pub client: String,
    pub hotel_price: u32,
    pub airline_price: u32,
}
