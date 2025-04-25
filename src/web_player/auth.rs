use super::{Result, WebPlayerError, USER_AGENT_CONTENT};
use reqwest::header::{COOKIE, USER_AGENT};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};

const URL_AUTH: &str = "https://open.spotify.com/get_access_token";
const QUERY: [(&str, &str); 3] = [
    ("reason", "transport"),
    ("productType", "web_player"),
    ("totpVer", "5"),
];

// Ciphered TOTP secret for Spotify Web Player
// This cipher should be fetched from Spotify web page for each request.
// However, this cipher does not change frequently at the moment.
// In addition, to fetch cipher is difficult for Spotify's complicated algorithm.
// So, we headcode cipher until Spotify changes algorithm.
const SECRET_CIPHER: [u8; 17] = [
    12, 56, 76, 33, 88, 44, 88, 33, 78, 78, 11, 66, 22, 22, 55, 69, 54,
];

fn decrypt_secret(cipher: &[u8]) -> Vec<u8> {
    // Decrypt the secret using XOR operation
    // Example: cipher = [0, 100, 255]
    cipher
        // XOR each byte with its index
        // Example: [0, 100, 255] -> [9, 110, 244]
        //          val=0 ^ (i=0 % 33 + 9) -> 0 ^ 9 -> 9
        .iter()
        .enumerate()
        .map(|(i, val)| val ^ ((i as u8) % 33 + 9))
        // Concatenate the decrypted bytes into a string
        // Example: [9, 110, 244] -> "9110244"
        .map(|val| val.to_string())
        .collect::<String>()
        // Convert the string to bytes
        // Example: "9110244" -> [57, 49, 49, 48, 50, 52, 52]
        .into_bytes()
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub clientId: String,
    pub accessToken: String,
    pub accessTokenExpirationTimestampMs: i64,
    pub isAnonymous: bool,
}

#[derive(Debug)]
pub struct AuthClient {
    request: RequestBuilder,
    totp: TOTP,
}

impl AuthClient {
    pub fn new<S: AsRef<str>>(sp_dc: S) -> Self {
        let secret_bytes = decrypt_secret(&SECRET_CIPHER);
        let secret = Secret::Raw(secret_bytes).to_bytes().unwrap();
        let totp = TOTP::new_unchecked(Algorithm::SHA1, 6, 1, 30, secret);

        let cookie = format!("sp_dc={}", sp_dc.as_ref());
        let request = Client::new()
            .get(URL_AUTH)
            .query(&QUERY)
            .header(COOKIE, cookie)
            .header(USER_AGENT, USER_AGENT_CONTENT);
        Self { request, totp }
    }

    pub async fn auth(self) -> Result<AuthData> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let otp = self.totp.generate(ts);

        let totp_query = [("totp", otp), ("ts", ts.to_string())];
        let resp = self.request.query(&totp_query).send().await?;
        if !resp.status().is_success() {
            return Err(WebPlayerError::InvalidToken);
        }
        let res_json = resp.text().await?;
        let auth_res: AuthData = serde_json::from_str(res_json.as_str())
            .map_err(|_| WebPlayerError::JsonParseError(res_json))?;
        if auth_res.isAnonymous {
            return Err(WebPlayerError::InvalidToken);
        }
        Ok(auth_res)
    }
}
