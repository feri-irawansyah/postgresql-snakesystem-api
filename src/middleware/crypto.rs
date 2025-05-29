use aes::Aes256;
use ctr::cipher::{KeyIvInit, StreamCipher};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use sha2::{Digest, Sha256};

use crate::SECRETS; // Import SHA-256 untuk hashing

type Aes256Ctr = ctr::Ctr64BE<Aes256>; // AES-256 dengan Counter Mode (CTR)

/// 📌 **Fungsi untuk membuat IV dari hash plaintext**
fn generate_iv(plain_text: &str) -> [u8; 16] {
    let hash = Sha256::digest(plain_text.as_bytes()); // Hash teks asli
    let mut iv = [0u8; 16];
    iv.copy_from_slice(&hash[..16]); // Ambil 16 byte pertama sebagai IV
    iv
}

/// 🔐 Enkripsi teks dengan AES-256-CTR (IV tetap sama untuk teks yang sama)
pub fn encrypt_text(plain_text: String) -> String {
    let secrets = SECRETS.get().expect("SECRETS not initialized");
    // 🔑 Kunci rahasia (HARUS 32-byte untuk AES-256)
    let secret_key = secrets.get("CRYPTO_SECRET").expect("secret was not found");
    let key = secret_key.as_bytes();
    let iv = generate_iv(&plain_text); // Gunakan IV dari hash

    // Inisialisasi cipher dengan key & IV
    let mut cipher = Aes256Ctr::new(key.into(), &iv.into());

    // Enkripsi data
    let mut encrypted_data = plain_text.as_bytes().to_vec();
    cipher.apply_keystream(&mut encrypted_data);

    // Encode hanya Ciphertext ke Base64 URL-Safe (IV tidak dikirim)
    URL_SAFE.encode(encrypted_data)
}

// 🔓 Dekripsi teks dengan AES-256-CTR
pub fn decrypt_text(encrypted_text: &str, original_text: &str) -> String {
    let secrets = SECRETS.get().expect("SECRETS not initialized");
    let secret_key = secrets.get("CRYPTO_SECRET").expect("secret was not found");
    // 🔑 Kunci rahasia (HARUS 32-byte untuk AES-256)
    let key = secret_key.as_bytes();
    let iv = generate_iv(original_text); // Dapatkan IV dari teks asli

    // Decode Base64 URL-Safe ke Ciphertext
    let encrypted_data = URL_SAFE.decode(encrypted_text).expect("Invalid Base64");

    // Inisialisasi cipher dengan key & IV
    let mut cipher = Aes256Ctr::new(key.into(), &iv.into());

    // Dekripsi data
    let mut decrypted_data = encrypted_data.to_vec();
    cipher.apply_keystream(&mut decrypted_data);

    String::from_utf8(decrypted_data).expect("Invalid UTF-8")
}