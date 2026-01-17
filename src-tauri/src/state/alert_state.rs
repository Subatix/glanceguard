use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorState {
    Idle,
    Monitoring,
    Alert,
    Cooldown,
}

#[derive(Debug, Clone)]
pub struct AlertState {
    above_since: Option<Instant>,
    cooldown_until: Option<Instant>,
    alert_until: Option<Instant>,
}

#[derive(Debug, Clone, Copy)]
pub struct AlertUpdate {
    pub state: MonitorState,
    pub triggered: bool,
}

impl AlertState {
    pub fn new() -> Self {
        Self {
            above_since: None,
            cooldown_until: None,
            alert_until: None,
        }
    }

    pub fn update(
        &mut self,
        score: Option<f32>,
        threshold: f32,
        now: Instant,
        cooldown: Duration,
        has_faces: bool,
    ) -> AlertUpdate {
        let mut state = if has_faces {
            MonitorState::Monitoring
        } else {
            MonitorState::Idle
        };
        let mut triggered = false;

        if let Some(until) = self.cooldown_until {
            if now < until {
                return AlertUpdate {
                    state: MonitorState::Cooldown,
                    triggered: false,
                };
            }
            self.cooldown_until = None;
        }

        if let Some(score) = score {
            if score >= threshold {
                if self.above_since.is_none() {
                    self.above_since = Some(now);
                }
                if now.duration_since(self.above_since.unwrap()) >= Duration::from_secs(2) {
                    self.alert_until = Some(now + Duration::from_secs(2));
                    self.cooldown_until = Some(now + cooldown);
                    self.above_since = None;
                    triggered = true;
                }
            } else {
                self.above_since = None;
            }
        } else {
            self.above_since = None;
        }

        if let Some(until) = self.alert_until {
            if now < until {
                state = MonitorState::Alert;
            } else if self.cooldown_until.is_some() {
                state = MonitorState::Cooldown;
            }
        } else if self.cooldown_until.is_some() {
            state = MonitorState::Cooldown;
        }

        AlertUpdate { state, triggered }
    }
}

#[cfg(test)]
mod tests {
    use super::{AlertState, MonitorState};
    use std::time::{Duration, Instant};

    #[test]
    fn triggers_after_two_seconds() {
        let mut state = AlertState::new();
        let start = Instant::now();

        let update = state.update(Some(0.9), 0.8, start, Duration::from_secs(30), true);
        assert!(!update.triggered);
        assert_eq!(update.state, MonitorState::Monitoring);

        let update = state.update(
            Some(0.9),
            0.8,
            start + Duration::from_secs(1),
            Duration::from_secs(30),
            true,
        );
        assert!(!update.triggered);

        let update = state.update(
            Some(0.9),
            0.8,
            start + Duration::from_secs(2),
            Duration::from_secs(30),
            true,
        );
        assert!(update.triggered);
        assert_eq!(update.state, MonitorState::Alert);
    }

    #[test]
    fn cooldown_blocks_new_alerts() {
        let mut state = AlertState::new();
        let start = Instant::now();

        let _ = state.update(
            Some(0.9),
            0.8,
            start,
            Duration::from_secs(10),
            true,
        );
        let update = state.update(
            Some(0.9),
            0.8,
            start + Duration::from_secs(2),
            Duration::from_secs(10),
            true,
        );
        assert!(update.triggered);

        let update = state.update(
            Some(0.9),
            0.8,
            start + Duration::from_secs(3),
            Duration::from_secs(10),
            true,
        );
        assert!(!update.triggered);
        assert_eq!(update.state, MonitorState::Cooldown);
    }

    #[test]
    fn returns_to_monitoring_after_cooldown() {
        let mut state = AlertState::new();
        let start = Instant::now();

        let _ = state.update(
            Some(0.9),
            0.8,
            start,
            Duration::from_secs(5),
            true,
        );
        let _ = state.update(
            Some(0.9),
            0.8,
            start + Duration::from_secs(2),
            Duration::from_secs(5),
            true,
        );

        let update = state.update(
            Some(0.1),
            0.8,
            start + Duration::from_secs(7),
            Duration::from_secs(5),
            true,
        );
        assert_eq!(update.state, MonitorState::Monitoring);
    }
}
