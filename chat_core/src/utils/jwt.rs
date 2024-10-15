use jwt_simple::prelude::*;

use crate::User;
const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISSUER: &str = "chat_server";
const JWT_AUDIENCE: &str = "chat_web";

pub struct EncodingKey(Ed25519KeyPair);

#[allow(unused)]
pub struct DecodingKey(Ed25519PublicKey);
impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    // pub fn sign(user: User, key: &EncodingKey) -> Result<String, AppError> {
    //     let mut claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
    //     let claims = claims.with_issuer(JWT_ISSUER)
    //         .with_audience(JWT_AUDIENCE);
    //     Ok(key.sign(claims)?)
    // }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, jwt_simple::Error> {
        let user: User = user.into();
        let claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISSUER).with_audience(JWT_AUDIENCE);
        self.0.sign(claims)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }
    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, jwt_simple::Error> {
        let mut opts = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISSUER])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUDIENCE])),
            ..Default::default()
        };
        let claims = self.0.verify_token::<User>(token, Some(opts))?;
        Ok(claims.custom)
    }
}
// impl Deref for EncodingKey {
//     type Target = Ed25519KeyPair;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl Deref for DecodingKey {
//     type Target = Ed25519PublicKey;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn jwt_sign_verify_should_work() -> anyhow::Result<()> {
        // let pem = fs::read_to_string(&config,jwt_pr)
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");
        let ek = EncodingKey::load(encoding_pem)?;
        let dk = DecodingKey::load(decoding_pem)?;
        let user = User::new(1, "zhang", "qazwsx2228@163.com");
        let token = ek.sign(user.clone())?;
        let user2 = dk.verify(&token)?;
        println!("{token:?}");
        println!("user = {:?}", user);
        println!("user2 = {:?}", user2);
        assert_eq!(user, user2);
        Ok(())
    }
}
