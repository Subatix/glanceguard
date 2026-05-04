use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::cv::types::{BoundingBox, FaceDetection};

const IOU_MATCH_THRESHOLD: f32 = 0.3;
const TRACK_MAX_AGE: Duration = Duration::from_millis(1000);
const LABEL_HISTORY: usize = 8;
const OBSERVER_AGREE_NEED: usize = 5;
const EMA_ALPHA: f32 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StableFaceLabel {
    Owner,
    Observer,
    Uncertain,
}

struct Track {
    id: u64,
    bbox: BoundingBox,
    last_seen: Instant,
    ema_similarity: Option<f32>,
    label_history: VecDeque<bool>,
}

impl Track {
    fn new(id: u64, bbox: BoundingBox, now: Instant) -> Self {
        Self {
            id,
            bbox,
            last_seen: now,
            ema_similarity: None,
            label_history: VecDeque::with_capacity(LABEL_HISTORY),
        }
    }

    fn push_raw_label(&mut self, is_observer: bool) {
        if self.label_history.len() == LABEL_HISTORY {
            self.label_history.pop_front();
        }
        self.label_history.push_back(is_observer);
    }

    fn stable_label(&self) -> StableFaceLabel {
        if self.label_history.len() < LABEL_HISTORY {
            return StableFaceLabel::Uncertain;
        }
        let obs = self.label_history.iter().filter(|&&x| x).count();
        let own = LABEL_HISTORY - obs;
        if obs >= OBSERVER_AGREE_NEED {
            StableFaceLabel::Observer
        } else if own >= OBSERVER_AGREE_NEED {
            StableFaceLabel::Owner
        } else {
            StableFaceLabel::Uncertain
        }
    }

    fn update_similarity_ema(&mut self, similarity: f32) {
        self.ema_similarity = Some(match self.ema_similarity {
            None => similarity,
            Some(prev) => EMA_ALPHA * similarity + (1.0 - EMA_ALPHA) * prev,
        });
    }
}

pub struct FaceTracker {
    tracks: Vec<Track>,
    next_id: u64,
}

impl FaceTracker {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            next_id: 0,
        }
    }

    /// Returns per-detection: (track_id, stable_label, similarity used for UI / scoring).
    pub fn update(
        &mut self,
        now: Instant,
        detections: &[FaceDetection],
        similarities: &[Option<f32>],
        owner_cosine_threshold: f32,
    ) -> Vec<Option<TrackOutput>> {
        self.prune_stale(now);

        let n = detections.len();
        let mut outputs: Vec<Option<TrackOutput>> = vec![None; n];
        if n == 0 {
            return outputs;
        }

        let mut det_used = vec![false; n];
        let mut track_used = vec![false; self.tracks.len()];

        let mut pairs: Vec<(usize, usize, f32)> = Vec::new();
        for (ti, track) in self.tracks.iter().enumerate() {
            for (di, det) in detections.iter().enumerate() {
                let i = iou(&track.bbox, &det.bbox);
                if i >= IOU_MATCH_THRESHOLD {
                    pairs.push((di, ti, i));
                }
            }
        }
        pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        for (di, ti, _) in pairs {
            if det_used[di] || track_used[ti] {
                continue;
            }
            det_used[di] = true;
            track_used[ti] = true;

            let track = &mut self.tracks[ti];
            track.bbox = detections[di].bbox.clone();
            track.last_seen = now;

            let sim_opt = similarities.get(di).copied().flatten();
            if let Some(sim) = sim_opt {
                track.update_similarity_ema(sim);
                let is_observer = sim < owner_cosine_threshold;
                track.push_raw_label(is_observer);
            }

            let stable = track.stable_label();
            outputs[di] = Some(TrackOutput {
                track_id: track.id,
                stable_label: stable,
                ema_similarity: track.ema_similarity,
                similarity_this_frame: sim_opt,
            });
        }

        for di in 0..n {
            if det_used[di] {
                continue;
            }
            let id = self.next_id;
            self.next_id += 1;
            let bbox = detections[di].bbox.clone();
            let mut track = Track::new(id, bbox, now);
            let sim_opt = similarities.get(di).copied().flatten();
            if let Some(sim) = sim_opt {
                track.update_similarity_ema(sim);
                let is_observer = sim < owner_cosine_threshold;
                track.push_raw_label(is_observer);
            }
            self.tracks.push(track);
            let tr = self.tracks.last().expect("just pushed");
            outputs[di] = Some(TrackOutput {
                track_id: id,
                stable_label: tr.stable_label(),
                ema_similarity: tr.ema_similarity,
                similarity_this_frame: sim_opt,
            });
        }

        outputs
    }

    fn prune_stale(&mut self, now: Instant) {
        self.tracks
            .retain(|t| now.duration_since(t.last_seen) <= TRACK_MAX_AGE);
    }
}

impl Default for FaceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TrackOutput {
    #[allow(dead_code)]
    pub track_id: u64,
    pub stable_label: StableFaceLabel,
    pub ema_similarity: Option<f32>,
    pub similarity_this_frame: Option<f32>,
}

fn iou(a: &BoundingBox, b: &BoundingBox) -> f32 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.width).min(b.x + b.width);
    let y2 = (a.y + a.height).min(b.y + b.height);
    let inter = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
    let union = a.width * a.height + b.width * b.height - inter;
    if union <= 0.0 {
        0.0
    } else {
        inter / union
    }
}
