pub fn generate(key: String) -> (String, String) {
	/* BCrypt only allows for keys with a maximum of 72 bytes. */
	let pass = key.into_bytes().into_iter().take(72).collect::<Vec<_>>();
	let salt = (0..16)
		.into_iter()
		.map(|_| rand::random::<u8>())
		.collect::<Vec<_>>();
	
	let mut out = [0_u8; 24];
	bcrypt::bcrypt(bcrypt::DEFAULT_COST, &pass[..], &salt[..], &mut out);

	let pass = out.iter()
		.map(|val| format!("{:02x}", val))
		.collect::<String>();
	let salt = salt.iter()
		.map(|val| format!("{:02x}", val))
		.collect::<String>();
	
	(pass, salt)
}

pub enum VerifyError {
	InvalidHash,
	InvalidSalt
}

fn _from_hex(val: &str) -> Result<Vec<u8>, ()> {
	let chars = val.chars().collect::<Vec<_>>();
	
	chars[..]
		.chunks(2)
		.map(|chars| {
			let mut s = String::new();
			for c in chars { s.push(*c); }

			u8::from_str_radix(s.as_str(), 16)
				.map_err(|_| ())
		})
        .collect()
}

pub fn verify(key: String, hash: String, _salt: String) 
	-> Result<bool, VerifyError> {

	let pass = key.into_bytes().into_iter().take(72).collect::<Vec<_>>();
	Ok(
		bcrypt::verify(&pass[..], hash.as_str())
			.expect("ALRIGHT WE FUCKED UP WITH BCRYPT PLS HELP ;-;")
	)
}
