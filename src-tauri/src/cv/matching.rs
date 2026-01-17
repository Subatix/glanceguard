pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32, String> {
    if a.len() != b.len() {
        return Err("Embedding length mismatch".to_string());
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for (av, bv) in a.iter().zip(b.iter()) {
        dot += av * bv;
        norm_a += av * av;
        norm_b += bv * bv;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return Err("Zero-length embedding norm".to_string());
    }

    Ok(dot / (norm_a.sqrt() * norm_b.sqrt()))
}
