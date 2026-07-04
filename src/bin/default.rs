use chacha20poly1305::{
    ChaCha20Poly1305,
    Key,
    Nonce,
    KeyInit,
    aead::Aead,
};
use x25519_dalek::{StaticSecret, PublicKey, SharedSecret};
use hkdf::Hkdf;
use sha2::Sha256;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use rand_core::{OsRng, RngCore};

use std::path::Path;
use std::fs;
use std::env;
use std::io::{self, Read};


// ----------------------------
// 鍵生成
// ----------------------------
fn generate_keys(secret: &mut String, public: &mut String) {
    let my_secret = StaticSecret::new(OsRng);
    let my_public = PublicKey::from(&my_secret);

    *secret = STANDARD.encode(my_secret.to_bytes());
    *public = STANDARD.encode(my_public.as_bytes());
}

// ----------------------------
// 秘密鍵の保存・読み込み
// ----------------------------
fn save_secret(path: &Path, secret: &StaticSecret) -> std::io::Result<()> {
    let encoded = STANDARD.encode(secret.to_bytes());
    fs::write(path, encoded)?;
    Ok(())
}

fn load_secret(path: &Path) -> StaticSecret {
    let encoded = fs::read_to_string(path).expect("failed to read secret key file");
    let bytes = STANDARD.decode(encoded).expect("failed to decode secret key");

    let arr: [u8; 32] = bytes.try_into().expect("secret key must be 32 bytes");
    StaticSecret::from(arr)
}

// ----------------------------
// 公開鍵の保存・読み込み
// ----------------------------
fn save_public(path: &Path, pubkey: &PublicKey) -> std::io::Result<()> {
    let encoded = STANDARD.encode(pubkey.as_bytes());
    fs::write(path, encoded)?;
    Ok(())
}

fn load_public(path: &Path) -> PublicKey {
    let encoded = fs::read_to_string(path).expect("failed to read public key file");
    let bytes = STANDARD.decode(encoded).expect("failed to decode public key");

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
    let ciphertext = STANDARD.decode(ciphertext_b64).expect("failed to decode ciphertext");

    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext_bytes = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .expect("decryption failed");

    String::from_utf8(plaintext_bytes).expect("plaintext is not valid UTF-8")
}

// ----------------------------
// 相手公開鍵の登録
// ----------------------------
fn register_their_public(their_public_path: &Path) {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read their public key from stdin");

    let input = input.trim();
    let bytes = STANDARD.decode(input).expect("failed to decode their public key");

    let arr: [u8; 32] = bytes.try_into().expect("public key must be 32 bytes");
    let pubkey = PublicKey::from(arr);

    save_public(their_public_path, &pubkey).expect("failed to save their public key");
}

fn compare_pub_keys(key1:&PublicKey, key2:&PublicKey) -> bool {
    if key1 == key2 {
        true
    } else {
        false
    }
}

// ----------------------------
// main
// ----------------------------
fn main() {
    let args: Vec<String> = env::args().collect();

    let my_secret_path = Path::new("sec.key");
    let my_public_path = Path::new("pub.key");
    let their_public_path = Path::new("their_pub.key");

    match args.len() {
        5 => {
            let code = &args[1];
            let mode = &args[2];

            if mode == "true" {
                let result = encrypt_message(my_secret_path, my_public_path,  their_public_path, code);
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

        7 => {
            let key_flag = &args[5];
            let register_flag = &args[6];

            if key_flag == "true" {
                let mut my_secret_b64 = String::new();
                let mut my_public_b64 = String::new();
                generate_keys(&mut my_secret_b64, &mut my_public_b64);

                let secret_bytes = STANDARD.decode(&my_secret_b64).unwrap();
                let public_bytes = STANDARD.decode(&my_public_b64).unwrap();

                let sec_arr: [u8; 32] = secret_bytes.try_into().unwrap();
                let pub_arr: [u8; 32] = public_bytes.try_into().unwrap();

                let my_secret = StaticSecret::from(sec_arr);
                let my_public = PublicKey::from(pub_arr);

                save_secret(my_secret_path, &my_secret).expect("failed to save secret key");
                save_public(my_public_path, &my_public).expect("failed to save public key");

                print!("\n{}\n", my_public_b64);
            } else if register_flag == "true" {
                register_their_public(their_public_path);
                println!("registered their public key");
            } else {
                eprintln!("no operation specified (key/register flags are false)");
                std::process::exit(1);
            }
        }

        _ => {
            eprintln!("invalid arguments for plugin");
            std::process::exit(1);
        }
    }
}
