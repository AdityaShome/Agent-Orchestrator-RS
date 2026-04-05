use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Plan {
    pub steps: Vec<String>,
}
