use std::sync::Arc;

use crate::core::{Ray, sampler::Sampler, spectrum::Spectrum};

#[derive(Debug, PartialEq, Clone)]
pub enum Medium {

}

impl Medium {
    pub fn tr(&self, _ray: &Ray, _sampler: &mut Sampler) -> Spectrum {
        match self {
            _ => panic!("Medium::Tr")
        }
    }
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