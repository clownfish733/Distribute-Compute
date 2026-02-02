use crate::core::error::P2PError;

use std::sync::Arc;

pub struct Network{
    
}

impl Network {
    pub async fn start(self: &mut Arc<Network>) -> Result<(), P2PError>{
        todo!()
    }
}

pub trait NetworkHander{
    
}