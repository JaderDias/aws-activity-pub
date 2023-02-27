pub trait Verifier {
    /// Verify if the signature is valid
    /// # Errors
    ///
    /// Will return `Err` if is unable to perform the verification.
    fn verify(&self, data: &str, signature: &[u8]) -> Result<bool, String>;
}
