pub struct Car {
    car_images: Vec<String>,
    model: String,
    car_id: String,
    owner_id: String,
    location: String,
    downpayment_amt: f32,
    price: f32,
    description: String,
    pricing: Charges,
    available: bool,
    verified: bool,
}

enum Charges {
    Hourly,
    PerDay,
}
