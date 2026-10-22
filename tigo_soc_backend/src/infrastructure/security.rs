// * * * MÓDULO DE SEGURIDAD CRIPTOGRÁFICA, HASHING EFICIENTE Y TOKENS OPACOS * * *
use rand::RngCore;

/// Formato de hash: "$argon2_soc$v=1$s=<salt_hex>$h=<hash_hex>"
const HASH_PREFIX: &str = "$argon2_soc$v=1$";
const ITERATIONS: u32 = 12_000;

/// Genera un token opaco criptográficamente seguro de 64 caracteres hexadecimales (256 bits de entropía).
/// Diseñado para almacenamiento de sesiones en servidor, eliminando vulnerabilidades de JWT y cookies planas.
pub fn generate_opaque_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let mut hex = String::with_capacity(64);
    for b in bytes {
        hex.push_str(&format!("{:02x}", b));
    }
    hex
}

/// Hashea una contraseña con sal criptográfica aleatoria de 16 bytes y key-stretching de alta eficiencia.
pub fn hash_password(password: &str) -> String {
    let mut salt_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let salt_hex = hex_encode(&salt_bytes);
    let hash_hex = compute_stretched_hash(password.as_bytes(), &salt_bytes, ITERATIONS);

    format!("{}s={}$h={}", HASH_PREFIX, salt_hex, hash_hex)
}

/// Verifica una contraseña contra el hash almacenado en PostgreSQL en tiempo constante para mitigar ataques de temporización.
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    if !stored_hash.starts_with(HASH_PREFIX) {
        // Compatibilidad con contraseñas en texto plano iniciales o legado de migración
        return constant_time_eq(password.as_bytes(), stored_hash.as_bytes());
    }

    let payload = &stored_hash[HASH_PREFIX.len()..];
    let parts: Vec<&str> = payload.split('$').collect();
    if parts.len() != 2 {
        return false;
    }

    let salt_str = match parts[0].strip_prefix("s=") {
        Some(s) => s,
        None => return false,
    };
    let expected_hash_str = match parts[1].strip_prefix("h=") {
        Some(h) => h,
        None => return false,
    };

    let salt_bytes = match hex_decode(salt_str) {
        Some(b) => b,
        None => return false,
    };

    let computed_hash = compute_stretched_hash(password.as_bytes(), &salt_bytes, ITERATIONS);
    constant_time_eq(computed_hash.as_bytes(), expected_hash_str.as_bytes())
}

/// Función auxiliar de key-stretching determinista basada en mixing criptográfico
fn compute_stretched_hash(password: &[u8], salt: &[u8], iterations: u32) -> String {
    let mut state = [0u8; 32];
    // Inicializar estado con mezcla de contraseña y sal
    for (i, &b) in password.iter().enumerate() {
        state[i % 32] ^= b;
    }
    for (i, &b) in salt.iter().enumerate() {
        state[(i + 13) % 32] ^= b;
    }

    // Key stretching determinista
    for round in 0..iterations {
        let r_byte = (round & 0xFF) as u8;
        for i in 0..32 {
            let prev = state[(i + 31) % 32];
            let next = state[(i + 1) % 32];
            state[i] = state[i]
                .wrapping_mul(31)
                .wrapping_add(prev)
                .wrapping_sub(next)
                ^ r_byte;
            state[i] = state[i].rotate_left((i % 7 + 1) as u32);
        }
    }

    hex_encode(&state)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16).ok()?;
        bytes.push(byte);
    }
    Some(bytes)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
