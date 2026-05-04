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

/// ArcFace probe vs owner reference frames — max cosine when both are L2-normalized.
pub fn max_cosine_vs_samples(probe: &[f32], samples: &[Vec<f32>]) -> Result<f32, String> {
    if samples.is_empty() {
        return Err("No embedding samples for owner".into());
    }
    let mut best: Option<f32> = None;
    for s in samples {
        let c = cosine_similarity(probe, s)?;
        best = Some(best.map_or(c, |b| b.max(c)));
    }
    best.ok_or_else(|| "No embedding samples for owner".into())
}

fn l2_normalize_vec(mut v: Vec<f32>) -> Vec<f32> {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 1e-6 {
        for x in &mut v {
            *x /= n;
        }
    }
    v
}

/// Mean of L2-normalized samples, then L2-normalize (enrollment fusion).
pub fn mean_embedding(samples: &[Vec<f32>]) -> Result<Vec<f32>, String> {
    if samples.is_empty() {
        return Err("No embeddings to average".into());
    }
    let dim = samples[0].len();
    if dim == 0 {
        return Err("Empty embedding vector".into());
    }
    for s in samples {
        if s.len() != dim {
            return Err("Inconsistent embedding dimension in enrollment batch".into());
        }
    }
    let mut sum = vec![0.0f32; dim];
    for s in samples {
        for i in 0..dim {
            sum[i] += s[i];
        }
    }
    let n = samples.len() as f32;
    for x in &mut sum {
        *x /= n;
    }
    Ok(l2_normalize_vec(sum))
}

/// Pairwise cosine statistics across enrollment samples → `μ - 3σ`, clamped to sane cosine bounds.
pub fn calibrate_personal_threshold(samples: &[Vec<f32>]) -> Result<f32, String> {
    if samples.len() < 2 {
        return Err("Need at least 2 enrollment embeddings for calibration".into());
    }
    let mut cosines = Vec::new();
    for i in 0..samples.len() {
        for j in (i + 1)..samples.len() {
            cosines.push(cosine_similarity(&samples[i], &samples[j])?);
        }
    }
    let n = cosines.len() as f32;
    let mean = cosines.iter().copied().sum::<f32>() / n;
    let var = cosines.iter().map(|c| (c - mean).powi(2)).sum::<f32>() / n;
    let sigma = var.sqrt();
    let raw = mean - 3.0 * sigma;
    let floor = 0.35_f32;
    let ceiling = 0.85_f32;
    Ok(raw.clamp(floor, ceiling))
}

/// Cosine threshold for owner match: personal calibration when present, else global default.
pub fn owner_cosine_threshold(
    profile: &crate::cv::types::OwnerProfile,
    global_fallback: f32,
) -> f32 {
    profile.personal_threshold.unwrap_or(global_fallback)
}
