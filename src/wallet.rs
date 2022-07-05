// use k256::ecdsa::{SigningKey, VerifyingKey};
// use k256::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, Result as pkcsResult};
// use k256::SecretKey;
// use rand_core::OsRng;

// struct Wallet {
//     signing_key_location: String,
// }

// impl Wallet {
//     pub fn get_signing_key(&self) -> pkcsResult<SigningKey> {
//         SigningKey::read_pkcs8_pem_file(&self.signing_key_location)
//     }

//     pub fn get_public_key(&self) -> pkcsResult<VerifyingKey> {
//         let signing = self.get_signing_key()?;
//         Ok(signing.verifying_key())
//     }

//     pub fn generate_signing_key(&self) -> SigningKey {
//         let signing_key = SigningKey::random(&mut OsRng);
//         signing_key.write_pkcs8_pem_file();

//         todo!()
//     }
// }
