use crate::chip8::Instruction;

fn n_set_bits(num_bits: u8) -> u16 {
    assert!(num_bits < 16);
    (1 << num_bits) - 1
}

fn get_nibbles(instruction: u16, index: u8, num: u8) -> u16 {
    assert!(num <= 4); // Only 4 nibbles in u16
    assert!(index + num <= 4); // indexes from 0-3
    (instruction >> ((4 - index - num) * 4)) & n_set_bits(num * 4)
}

fn get_nibble(instruction: u16, index: u8) -> u8 {
    assert!(index <= 3);
    get_nibbles(instruction, index, 1) as u8
}

pub fn decode(instruction: u16) -> Option<Instruction> {
    match get_nibble(instruction, 0) {
        0x0 => match get_nibbles(instruction, 1, 3) {
            0x0e0 => Some(Instruction::ClearScreen),
            _ => None,
        },
        0x1 => {
            let dest = get_nibbles(instruction, 1, 3);
            Some(Instruction::Jump { dest })
        }
        0x2 => None,
        0x3 => None,
        0x4 => None,
        0x5 => None,
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
        0x8 => None,
        0x9 => None,
        0xa => {
            let value = get_nibbles(instruction, 1, 3);
            Some(Instruction::SetIndexRegister { value })
        }
        0xb => None,
        0xc => None,
        0xd => {
            let x_r = get_nibble(instruction, 1);
            let y_r = get_nibble(instruction, 2);
            let height = get_nibble(instruction, 3);
            Some(Instruction::Draw { x_r, y_r, height })
        }
        0xe => None,
        0xf => None,
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
    }
}