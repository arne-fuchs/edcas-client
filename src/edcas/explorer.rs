use std::sync::Arc;

use crate::edcas::explorer::system::System;
use crate::edcas::settings::Settings;

pub mod belt_cluster;
pub mod body;
pub mod planet;
pub(crate) mod ring;
pub mod star;
pub mod system;

pub struct Explorer {
    pub systems: Vec<System>,
    pub index: usize,
    pub body_list_index: Option<usize>,
    pub settings: Arc<Settings>,
}
