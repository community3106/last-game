use last_game::nostr::crypto::{nip04_decrypt, nip04_encrypt, nip44_decrypt, nip44_encrypt, KeyPair};

#[test]
fn test_nip04_encryption_roundtrip() {
    let admin = KeyPair::default_admin();
    let user = KeyPair::demo_vip_user();

    let secret_message = "WORLD_1_4_BOWSER_CASTLE_VIP_ACCESS_TOKEN";

    // Admin encrypts to User
    let encrypted = nip04_encrypt(secret_message, &admin.secret_key_hex, &user.public_key_hex)
        .expect("NIP-04 encryption should succeed");
    
    assert!(encrypted.contains("?iv="), "NIP-04 format must contain ?iv=");

    // User decrypts with their secret key
    let decrypted = nip04_decrypt(&encrypted, &user.secret_key_hex, &admin.public_key_hex)
        .expect("NIP-04 decryption should succeed");

    assert_eq!(decrypted, secret_message);
}

#[test]
fn test_nip44_encryption_roundtrip() {
    let admin = KeyPair::default_admin();
    let user = KeyPair::demo_vip_user();

    let secret_json = r#"{"vip_stage": "1-4", "bowser_defeated": false}"#;

    // Admin encrypts to User with ChaCha20-Poly1305 (NIP-44 v2)
    let encrypted_b64 = nip44_encrypt(secret_json, &admin.secret_key_hex, &user.public_key_hex)
        .expect("NIP-44 encryption should succeed");

    // User decrypts
    let decrypted = nip44_decrypt(&encrypted_b64, &user.secret_key_hex, &admin.public_key_hex)
        .expect("NIP-44 decryption should succeed");

    assert_eq!(decrypted, secret_json);
}
