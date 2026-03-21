use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub enum Medium {

}

#[derive(Debug, PartialEq, Clone)]
pub struct MediumInterface {
    pub inside: Option<Arc<Medium>>,
    pub outside: Option<Arc<Medium>>
}

impl MediumInterface {
    pub fn new() -> Self {
        Self {
            inside: None,
            outside: None
        }
    }
}

#[derive(Debug, Clone)]
pub struct HenyeyGreenstein {
    
}