use crate::cparam::CParameter;

#[derive(Clone, Debug)]
pub struct CFunction {
    pub return_type: String,
    pub name: String,
    pub parameters: Vec<CParameter>,
}

impl CFunction {
    pub fn new() -> Self {
        Self {
            return_type: String::new(),
            name: String::new(),
            parameters: Vec::new(),
        }
    }
}
