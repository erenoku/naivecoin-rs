use openssl::{
    bn, ec,
    ec::{EcGroup, EcKey, EcPoint, EcPointRef},
    ecdsa::EcdsaSig,
    error::ErrorStack,
    nid::Nid,
    pkey::{Private, Public},
};

const CURVE: Nid = Nid::X9_62_PRIME256V1;

// higher level abstraction for the parts of openssl that will be used in this project
pub struct KeyPair {
    pub private_key: PrivateKey,
    group: EcGroup,
}

impl KeyPair {
    pub fn generate() -> Self {
        let group = Self::get_group();
        let key = EcKey::generate(&group).unwrap();

        Self {
            private_key: PrivateKey { key },
            group: Self::get_group(),
        }
    }

    pub fn public_key_from_hex(str: &String) -> Result<EcPoint, ErrorStack> {
        let mut ctx = bn::BigNumContext::new().unwrap();
        EcPoint::from_bytes(&Self::get_group(), str.as_bytes(), &mut ctx)
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

    // pub fn public_key_from_private(private_key:  )

    pub fn get_group() -> EcGroup {
        EcGroup::from_curve_name(CURVE).unwrap()
    }
}

pub struct Signature {
    pub signature: EcdsaSig,
}

impl Signature {
    pub fn from_string(str: &String) -> Result<Self, ErrorStack> {
        let signature = EcdsaSig::from_der(hex::decode(str).unwrap().as_slice())?;
        Ok(Self { signature })
    }

    pub fn to_string(&self) -> Result<String, ErrorStack> {
        Ok(hex::encode(self.signature.to_der()?))
    }

    pub fn verify(&self, data: &[u8], public_key: EcPoint) -> Result<bool, ErrorStack> {
        let key = EcKey::from_public_key(&KeyPair::get_group(), &public_key)?;

        self.signature.verify(data, &key)
    }

    pub fn from_sign(data: &[u8], private_key: PrivateKey) -> Result<Self, ErrorStack> {
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
}
