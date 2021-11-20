use crate::chip8::Instruction;
use crate::bits::{get_nibble, get_nibbles};

pub fn decode(instruction: u16) -> Option<Instruction> {
    match get_nibble(instruction, 0) {
        0x0 => match get_nibbles(instruction, 1, 3) {
            0x0e0 => Some(Instruction::ClearScreen),
            0x0ee => Some(Instruction::Return),
            _ => None,
        },
        0x1 => {
            Some(Instruction::Jump { 
                dest: get_nibbles(instruction, 1, 3) 
            })
        }
        0x2 => {
            Some(Instruction::CallSubroutine { 
                dest: get_nibbles(instruction, 1, 3) 
            })
        },
        0x3 => Some(Instruction::SkipEQ {
            register: get_nibble(instruction, 1),
            value: get_nibbles(instruction, 2, 2) as u8
        }),
        0x4 => Some(Instruction::SkipNEQ {
            register: get_nibble(instruction, 1),
            value: get_nibbles(instruction, 2, 2) as u8
        }),
        0x5 => Some(Instruction::SkipEQR {
            register1: get_nibble(instruction, 1),
            register2: get_nibble(instruction, 2),
        }),
        0x6 => {
            let register = get_nibble(instruction, 1);
            let value = get_nibbles(instruction, 2, 2) as u8;
            Some(Instruction::SetRegister { register, value })
        }
        0x7 => {
            let register = get_nibble(instruction, 1);
            let value = get_nibbles(instruction, 2, 2) as u8;
            Some(Instruction::AddToRegister { register, value })
        }
        0x8 => match get_nibble(instruction, 3) {
            0 => Some(Instruction::MovRegister { 
                register1: get_nibble(instruction, 1), 
                register2: get_nibble(instruction, 2)
            }),
            _ => None
        },
        0x9 => Some(Instruction::SkipNEQR {
            register1: get_nibble(instruction, 1),
            register2: get_nibble(instruction, 2),
        }),
        0xa => {
            let value = get_nibbles(instruction, 1, 3);
            Some(Instruction::SetIndexRegister { value })
        }
        0xb => None,
        0xc => Some(Instruction::Random { 
            register: get_nibble(instruction, 1),
            value: get_nibbles(instruction, 2, 2) as u8
        }),
        0xd => {
            let x_r = get_nibble(instruction, 1);
            let y_r = get_nibble(instruction, 2);
            let height = get_nibble(instruction, 3);
            Some(Instruction::Draw { x_r, y_r, height })
        },
        0xe => match get_nibbles(instruction, 2, 2) {
            0x9e => Some(Instruction::SkipPressed { key: get_nibble(instruction, 1) }),
            0xa1 => Some(Instruction::SkipNotPressed { key: get_nibble(instruction, 1) }),
            _ => None
        },
        0xf => {
            let nib = get_nibble(instruction, 1);
            match get_nibbles(instruction, 2, 2) {
                0x07 => Some(Instruction::GetDelayTimer { register: nib }),
                0x15 => Some(Instruction::SetDelayTimer { register: nib }),
                0x1e => Some(Instruction::AddToIndex { register: nib }), // TODO: set overflow
                _ => None
            }
        },
        _ => panic!("Impossible instruction"),
    }
}

#[cfg(test)]
mod tests {
    use crate::chip8::Instruction;
    use super::decode;
    #[test]
    fn working_instructions() {
        assert_eq!(decode(0xa2e0).unwrap(), Instruction::SetIndexRegister { value: 0x2e0 });
        assert_eq!(decode(0xdeaf).unwrap(), Instruction::Draw {x_r: 0xe, y_r: 0xa, height: 0xf });
        assert_eq!(decode(0x7abc).unwrap(), Instruction::AddToRegister { register: 0xa, value: 0xbc });
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn never_panics(instruction in 0..std::u16::MAX)
        {
            decode(instruction);
        }
    }
}