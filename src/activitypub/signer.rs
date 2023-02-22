pub trait Signer {
    fn get_key_id(&self) -> String;

    /// Sign some data with the signer keypair
    fn sign(&self, to_sign: &str) -> Result<Vec<u8>, String>;
}
