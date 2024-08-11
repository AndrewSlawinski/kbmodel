use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(from = "Vec<f64>")]
pub struct Bias {
    pub pinky: f64,
    pub ring: f64,
    pub middle: f64,
    pub index: f64,
}

impl From<Vec<f64>> for Bias {
    fn from(value: Vec<f64>) -> Self {
        Bias::adjust_with_quotient(&mut value.clone());

        return Self {
            pinky: value[0],
            ring: value[1],
            middle: value[2],
            index: value[3],
        };
    }
}

impl Bias {
    fn adjust_with_quotient(floats: &Vec<f64>) -> Vec<f64> {
        let sum: f64 = floats.iter().sum();

        let mut v = Vec::new();

        floats.iter().for_each(|x| {
            v.push(x / sum);
        });

        return v;
    }
}

impl Default for Bias {
    fn default() -> Self {
        let floats = Bias::adjust_with_quotient(&vec![9.0, 16.0, 19.5, 18.0]);

        return Self {
            pinky: floats[0],
            ring: floats[1],
            middle: floats[2],
            index: floats[3],
        };
    }
}
