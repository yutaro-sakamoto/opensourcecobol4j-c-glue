use crate::cparam::CParameterType;

#[derive(Clone, Debug)]
pub struct CFunction {
    pub return_type: String,
    pub name: String,
    pub parameter_types: Vec<CParameterType>,
}

impl CFunction {
    pub fn new() -> Self {
        Self {
            return_type: String::new(),
            name: String::new(),
            parameter_types: Vec::new(),
        }
    }
}
