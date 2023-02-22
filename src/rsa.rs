use openssl::rsa::Rsa;

pub fn der_to_pem(der: &Vec<u8>) -> String {
    let public_key = Rsa::public_key_from_der(der).unwrap();
    let public_key = public_key.public_key_to_pem().unwrap();
    String::from_utf8(public_key).unwrap()
}
