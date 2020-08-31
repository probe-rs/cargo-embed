use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Register {
    pub address: u32,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registers {
    pub registers: Vec<Register>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Update {
    Registers(Registers),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    UpdateInterval(usize),
    Watch(Vec<u32>),
    SetRegister(Register),
}
