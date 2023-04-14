use openssl::rsa::Rsa;

/// # Panics
///
/// Will panic if can´t parse the der public key, or can´t convert it to pem or can´t convert it to a string.
#[must_use]
pub fn der_to_pem(der: &[u8]) -> String {
    let public_key = Rsa::public_key_from_der(der).unwrap();
    let public_key = public_key.public_key_to_pem().unwrap();
    String::from_utf8(public_key).unwrap()
}
