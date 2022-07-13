use log::info;
use openssl::{
    bn, ec,
    ec::{EcGroup, EcKey, EcPoint, EcPointRef},
    ecdsa::EcdsaSig,
    error::ErrorStack,
    nid::Nid,
    pkey::{Private, Public},
};
use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

const CURVE: Nid = Nid::X9_62_PRIME256V1;

// higher level abstraction for the parts of openssl that will be used in this project
pub struct KeyPair {
    pub private_key: PrivateKey,
}

impl KeyPair {
    pub fn generate() -> Self {
        let group = Self::get_group();
        let key = EcKey::generate(&group).unwrap();

        Self {
            private_key: PrivateKey { key },
        }
    }

    pub fn public_key_from_hex(str: &String) -> Result<EcPoint, ErrorStack> {
        let mut ctx = bn::BigNumContext::new().unwrap();
        EcPoint::from_bytes(
            &Self::get_group(),
            hex::decode(str.as_bytes()).unwrap().as_slice(),
            &mut ctx,
        )
    }

    pub fn public_key_to_hex(public_key: &EcPoint) -> String {
        let mut ctx = bn::BigNumContext::new().unwrap();
        hex::encode(
            public_key
                .to_bytes(
                    &Self::get_group(),
                    ec::PointConversionForm::UNCOMPRESSED,
                    &mut ctx,
                )
                .unwrap(),
        )
    }

    pub fn get_group() -> EcGroup {
        EcGroup::from_curve_name(CURVE).unwrap()
    }
}

pub struct Signature {
    pub signature: EcdsaSig,
}

impl Signature {
    pub fn from_string(str: &String) -> Result<Self, Box<dyn Error>> {
        let signature = EcdsaSig::from_der(hex::decode(str)?.as_slice())?;
        Ok(Self { signature })
    }

    pub fn verify(&self, data: &[u8], public_key: EcPoint) -> Result<bool, ErrorStack> {
        let key = EcKey::from_public_key(&KeyPair::get_group(), &public_key)?;

        self.signature.verify(data, &key)
    }

    pub fn from_sign(data: &[u8], private_key: &PrivateKey) -> Result<Self, ErrorStack> {
        let sig = EcdsaSig::sign(data, &private_key.key)?;

        Ok(Self { signature: sig })
    }

    pub fn to_encoded(&self) -> String {
        hex::encode(self.signature.to_der().unwrap())
    }
}

pub struct PrivateKey {
    key: EcKey<Private>,
}

impl PrivateKey {
    pub fn to_public_key(&self) -> EcPoint {
        self.key
            .public_key()
            .to_owned(&KeyPair::get_group())
            .unwrap()
    }

    pub fn to_pem(&self) -> Result<Vec<u8>, ErrorStack> {
        self.key.private_key_to_pem()
    }

    pub fn from_pem(pem: &[u8]) -> Self {
        Self {
            key: EcKey::private_key_from_pem(pem).unwrap(),
        }
    }

    pub fn read_file_pem(path: &Path) -> io::Result<Self> {
        let pem = fs::read_to_string(path)?;
        Ok(Self::from_pem(pem.as_bytes()))
    }

    pub fn write_file_pem(&self, path: &Path) -> io::Result<()> {
        let pem = self.to_pem().unwrap();

        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        let mut f = File::create(path)?;
        info!(
            "writing private key to path: {}",
            path.as_os_str().to_str().unwrap()
        );

        return f.write_all(&pem);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_encode_decode() {
        let pair = KeyPair::generate();

        let encoded_pub_key = KeyPair::public_key_to_hex(&pair.private_key.to_public_key());
        let decoded_pub_key = KeyPair::public_key_from_hex(&encoded_pub_key).unwrap();
    }
}
