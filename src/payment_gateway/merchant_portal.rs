#[derive(Clone)]
pub struct Merchant<'a> {
	pub pass_key: String,
	pub business_short_code: i32,
	pub basic_auth: &'a str,
}

impl Merchant<'_> {
	pub fn get_credentials() -> Self {
		Merchant {
			pass_key: "bfb279f9aa9bdbcf158e97dd71a467cd2e0c893059b10f78e6b72ada1ed2c919".to_owned(),
			business_short_code: 174379,
			basic_auth: "Basic InthISf0rMatsWqKjgafiyu3jbkauyi32jhbjtg89dakbjbahsd",
		}
	}
}
