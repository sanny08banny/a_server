struct Review {
    id: u32,
    user_name: String,
    title: String,
    comment: String,
    rating: u8,
}

struct CarReview {
    owner_id: String,
    car_id: String,
    review: Vec<Review>,
}

struct DriverReview {
    driver_id: String,
    review: Vec<Review>,
}
