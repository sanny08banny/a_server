use base64::{
	alphabet,
	engine::{self, general_purpose},
};

pub const CUSTOM_ENGINE: engine::GeneralPurpose = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
