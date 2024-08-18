use crate::hand::finger::Finger;
use crate::n_gram::effort_type::EffortType;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct EffortStats {
    inner: HashMap<EffortType, Vec<f64>>,

    pub char_effort: HashMap<char, f64>,
    pub usage: HashMap<Finger, f64>,
    pub finger_speeds: HashMap<Finger, f64>,
}

impl EffortStats {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn usage_score(&self) -> f64 {
        return self.usage.values().sum();
    }

    pub fn finger_speed_score(&self) -> f64 {
        return self.usage.values().sum();
    }

    pub fn char_effort_score(&self) -> f64 {
        return self.usage.values().sum();
    }

    pub fn char_efforts() -> HashMap<char, f64> {
        for i in 0..30 {
            res.effort[i] = char_effort(data, layout, i);
        }

        res.effort_total = res.effort.iter().sum();
    }

    pub fn usages() -> HashMap<Finger, f64> {
        for column in 0..8 {
            res.usage[column] = column_usage(data, layout, weights, column);
        }

        res.usage_total = res.usage.iter().sum();
    }

    pub fn speeds() -> HashMap<Finger, f64> {
        for column in 0..8 {
            res.finger_speeds[column] = column_finger_speed(data, layout, column)
        }

        res.finger_speed_total = res.finger_speeds.iter().sum();
    }
}
