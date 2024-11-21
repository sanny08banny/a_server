CREATE TABLE IF NOT EXISTS Users (
	user_id TEXT UNIQUE NOT NULL,
	email TEXT UNIQUE NOT NULL,
	password TEXT NOT NULL,

	-- User Information
	is_admin BOOLEAN NOT NULL,
	is_driver BOOLEAN NOT NULL,
	user_name TEXT NOT NULL,
	phone_number TEXT,

	-- Platform Information
	tokens INTEGER NOT NULL DEFAULT 0,
	notification_token TEXT NOT NULL,

	-- Constraints
	PRIMARY KEY (user_id)
);

CREATE TABLE IF NOT EXISTS Car (
	car_id TEXT UNIQUE NOT NULL,
	owner_id TEXT NOT NULL,

	-- Car Information
	model TEXT NOT NULL,
	description TEXT NOT NULL,
	car_images TEXT NOT NULL,

	-- Platform Information
	available BOOLEAN NOT NULL,
	booking_tokens INTEGER NOT NULL DEFAULT 0,
	location TEXT NOT NULL,
	daily_amount REAL NOT NULL,
	daily_downpayment_amt REAL NOT NULL,

	-- Constraints
	PRIMARY KEY (car_id, owner_id),
	FOREIGN KEY (owner_id) REFERENCES Users(user_id)
);

CREATE TABLE IF NOT EXISTS Taxis (
	taxi_id TEXT UNIQUE NOT NULL,
	driver_id TEXT NOT NULL,

	-- Vehicle Information
	image_paths TEXT NOT NULL,
	plate_number TEXT NOT NULL,
	color TEXT NOT NULL,
	model TEXT NOT NULL,
	category TEXT NOT NULL,
	manufacturer TEXT NOT NULL,
	capacity INTEGER NOT NULL DEFAULT 3,

	-- Platform Information
	verified BOOLEAN,

	-- Constraints
	PRIMARY KEY (taxi_id, driver_id),
	FOREIGN KEY (driver_id) REFERENCES Users(user_id)
);

CREATE TABLE IF NOT EXISTS TaxiVerifications (
	driver_id TEXT NOT NULL,

	-- Details
	inspection_report TEXT NOT NULL,
	insurance TEXT NOT NULL,
	driving_license TEXT NOT NULL,
	psv_license TEXT NOT NULL,
	national_id TEXT NOT NULL,

	-- Constraints
	PRIMARY KEY (driver_id),
	FOREIGN KEY (driver_id) REFERENCES Users(user_id)
);
