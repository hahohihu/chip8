pub type U4 = u8;
pub type U12 = u16;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    ClearScreen,
    Jump { dest: U12 },
    SetRegister { register: U4, value: u8 },
    AddToRegister { register: U4, value: u8 },
    SetIndexRegister { value: U12 },
    Draw { x_r: U4, y_r: U4, height: U4 },
}
