use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce, aead::Aead};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey, SharedSecret, StaticSecret};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use rand_core::{OsRng, RngCore};

use std::env;
use std::fs;
use std::path::Path;
//use std::io::{self, Read};



fn load_secret(path: &Path) -> StaticSecret {
    let encoded = fs::read_to_string(path).expect("failed to read secret key file");
    let bytes = STANDARD
        .decode(encoded)
        .expect("failed to decode secret key");

    let arr: [u8; 32] = bytes.try_into().expect("secret key must be 32 bytes");
    StaticSecret::from(arr)
}

fn load_public(path: &Path) -> PublicKey {
    let encoded = fs::read_to_string(path).expect("failed to read public key file");
    let bytes = STANDARD
        .decode(encoded)
        .expect("failed to decode public key");

    let arr: [u8; 32] = bytes.try_into().expect("public key must be 32 bytes");
    PublicKey::from(arr)
}

// ----------------------------
// 共有鍵生成（X25519 + HKDF）
// ----------------------------
fn derive_chacha_key(my_secret: &StaticSecret, their_public: &PublicKey) -> Key {
    let shared: SharedSecret = my_secret.diffie_hellman(their_public);
    let shared_bytes = shared.as_bytes();

    let hk = Hkdf::<Sha256>::new(None, shared_bytes);
    let mut key_bytes = [0u8; 32];
    hk.expand(b"msg-encryption", &mut key_bytes)
        .expect("HKDF expand failed");

    Key::from_slice(&key_bytes).clone()
}

// ----------------------------
// 暗号化
// ----------------------------
fn encrypt_message(
    my_secret_path: &Path,
    my_pub_path: &Path,
    their_public_path: &Path,
    plaintext: &str,
) -> String {
    let my_secret = load_secret(my_secret_path);
    let my_pub = load_public(my_pub_path);
    let their_public = load_public(their_public_path);

    if compare_pub_keys(&my_pub, &their_public) {
        eprintln!("Error: your own public key is registered as the partner's key.");
        std::process::exit(1);
    }

    let key = derive_chacha_key(&my_secret, &their_public);
    let cipher = ChaCha20Poly1305::new(&key);

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("encryption failed");

    let nonce_b64 = STANDARD.encode(nonce_bytes);
    let ciphertext_b64 = STANDARD.encode(ciphertext);

    format!("{}:{}", nonce_b64, ciphertext_b64)
}

// ----------------------------
// 復号処理
// ----------------------------
fn decrypt_message(
    my_secret_path: &Path,
    their_public_path: &Path,
    nonce_b64: &str,
    ciphertext_b64: &str,
) -> String {
    let my_secret = load_secret(my_secret_path);
    let their_public = load_public(their_public_path);

    let key = derive_chacha_key(&my_secret, &their_public);
    let cipher = ChaCha20Poly1305::new(&key);

    let nonce_bytes = STANDARD.decode(nonce_b64).expect("failed to decode nonce");
    let ciphertext = STANDARD
        .decode(ciphertext_b64)
        .expect("failed to decode ciphertext");

    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .expect("decryption failed");

    String::from_utf8(plaintext_bytes).expect("plaintext is not valid UTF-8")
}

fn compare_pub_keys(key1: &PublicKey, key2: &PublicKey) -> bool {
    if key1 == key2 { true } else { false }
}

// ----------------------------
// main
// ----------------------------
fn main() {
    let args: Vec<String> = env::args().collect();

    let my_secret_path = Path::new("keys/sec.key");
    let my_public_path = Path::new("keys/pub.key");
    let their_public_path = Path::new("keys/their_pub.key");

    match args.len() {
        5 => {
            let code = &args[1];
            let mode = &args[2];

            if mode == "true" {
                let result =
                    encrypt_message(my_secret_path, my_public_path, their_public_path, code);
                print!("{}", result);
            } else {
                let parts: Vec<&str> = code.split(':').collect();
                if parts.len() != 2 {
                    eprintln!("invalid ciphertext format. expected nonce_b64:ciphertext_b64");
                    std::process::exit(1);
                }
                let nonce_b64 = parts[0];
                let ciphertext_b64 = parts[1];
                let result =
                    decrypt_message(my_secret_path, their_public_path, nonce_b64, ciphertext_b64);
                print!("{}", result);
            }
        }

        _ => {
            eprintln!("invalid arguments for plugin");
            std::process::exit(1);
        }
    }
}
