pub struct R {
    user_id: String,
    amount: f32,
}

impl R {
    pub fn new(user_id: String) -> R {
        R {
            user_id,
            amount: 0.0,
        }
    }
}
