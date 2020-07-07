use crate::config::Parameters;
use crate::States;
use anyhow::anyhow;
use anyhow::Result;
use lazy_static::lazy_static;
use ring::aead::{Aad, Algorithm, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

static ALGORITHM: &Algorithm = &AES_256_GCM;

lazy_static! {
    static ref RANDOM: SystemRandom = SystemRandom::new();
}

impl Parameters {
    pub fn read_states(&self) -> Result<States> {
        let mut file = File::open(&self.state_file)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let opened = self.open(&mut bytes)?;
        Ok(serde_json::from_slice(opened)?)
    }

    pub fn write_states(&self, states: &States) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.state_file)?;

        let vec = serde_json::to_vec(states)?;
        let sealed = self.seal(vec)?;

        file.write_all(&sealed[..])?;
        file.sync_all()?;

        Ok(())
    }

    fn seal(&self, mut raw: Vec<u8>) -> Result<Vec<u8>> {
        let mut key = match self.state_key {
            Some(ref key) => key.clone(),
            None => return Ok(raw),
        };
        key.extend(&[' '; 32]);
        let key = &key.as_bytes()[0..32];
        let key = UnboundKey::new(ALGORITHM, key)?;

        let seal_key = LessSafeKey::new(key);

        let nonce = new_nonce();
        seal_key.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce),
            Aad::empty(),
            &mut raw,
        )?;

        let result = [&nonce[..], &raw[..]].concat();

        Ok(result)
    }

    fn open<'a>(&self, sealed: &'a mut [u8]) -> Result<&'a [u8]> {
        let mut key = match self.state_key {
            Some(ref key) => key.clone(),
            None => return Ok(sealed),
        };
        if sealed.len() < NONCE_LEN {
            return Err(anyhow!("Less than nonce length."));
        }
        key.extend(&[' '; 32]);
        let key = &key.as_bytes()[0..32];
        let key = UnboundKey::new(ALGORITHM, key)?;

        let open_key = LessSafeKey::new(key);

        let mut nonce = [0; NONCE_LEN];
        nonce.copy_from_slice(&sealed[0..NONCE_LEN]);
        let opened = open_key.open_in_place(
            Nonce::assume_unique_for_key(nonce),
            Aad::empty(),
            &mut sealed[NONCE_LEN..],
        )?;

        Ok(opened)
    }
}

fn new_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0; NONCE_LEN];
    RANDOM.fill(&mut nonce).unwrap();

    nonce
}
