use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pago {
    pub id: u32,
    pub cliente: String,
    pub precio_hotel: u32,
    pub precio_aerolinea: u32,
}
