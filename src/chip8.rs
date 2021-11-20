use std::io::Read;
use crate::bits::{U4, U12};
use crate::decode::decode;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    ClearScreen,
    Return,
    Jump { dest: U12 },
    CallSubroutine { dest: U12},
    // SkipEQ { register: U4, value: u8 },
    SkipNEQ { register: U4, value: u8 },
    // SkipEQR { register1: U4, register2: U4 },
    SetRegister { register: U4, value: u8 },
    AddToRegister { register: U4, value: u8 },
    SetIndexRegister { value: U12 },
    Draw { x_r: U4, y_r: U4, height: U4 },
    SkipPressed { key: U4 },
    SkipNotPressed { key: U4 },
    AddToIndex { register: U4 },
}

pub enum Cycle {
    RedrawRequested,
    Complete
}

pub const INIT_INDEX: usize = 0x200;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
type Screen = [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT];
const BLANK_SCREEN: Screen = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8 {
    pub registers: [u8; 16],
    pub memory: [u8; 4096],
    pub pc: usize,
    pub index_register: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: Screen,
    pub stack: Vec<usize>
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            pc: INIT_INDEX,
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: BLANK_SCREEN,
            stack: Vec::new()
        };
        chip8.memory[0..FONT.len()].copy_from_slice(&FONT);
        chip8
    }

    pub fn get_instruction(&self) -> u16 {
        self.memory[self.pc + 1] as u16 | (self.memory[self.pc] as u16) << 8
    }

    pub fn pc_inbounds(&self) -> bool {
        self.pc >= INIT_INDEX && self.pc < 4095
    }

    pub fn read_program(&mut self, read: impl std::io::Read) -> Result<usize, std::io::Error> {
        let mut slice = &mut self.memory[INIT_INDEX .. ];
        let mut take = read.take(slice.len() as u64);
        take.read(&mut slice)
    }

    
    pub fn print_program(&self) {
        let mut zero_counter = 0;
        for i in (INIT_INDEX..4095).step_by(2) {
            let val1 = self.memory[i];
            let val2 = self.memory[i+1];
            if val1 == 0 {
                zero_counter += 1;
            }
            if zero_counter > 8 {
                break;
            }
            println!("{:01x}{:01x}", val1, val2);
        }
    }

    pub fn render(&self) {
        clear_terminal();
        render_screen(&self.display);
    }

    pub fn execute(&mut self, instruction: Instruction, key_pressed: Option<u8>) -> Cycle {
        match instruction {
            Instruction::ClearScreen => {
                self.display = BLANK_SCREEN;
                return Cycle::RedrawRequested;
            },
            Instruction::Return => {
                self.pc = self.stack.pop().expect("Program tried to return but stack was empty.");
            },
            Instruction::Jump { dest } => {
                self.pc = dest as usize;
            },
            Instruction::CallSubroutine { dest} => {
                self.stack.push(self.pc);
                self.pc = dest as usize;
            },
            Instruction::SkipNEQ { register, value} => {
                if self.registers[register as usize] != value {
                    self.pc += 2;
                }
            },
            Instruction::SetRegister { register, value } => {
                self.registers[register as usize] = value;
            },
            Instruction::AddToRegister { register, value } => {
                self.registers[register as usize] += value; // TODO: not clear if this should set carry flag
            },
            Instruction::SetIndexRegister { value } => {
                self.index_register = value;
            },
            Instruction::Draw { x_r, y_r, height } => { // TODO: problem is probably here
                let x = self.registers[x_r as usize] % SCREEN_WIDTH as u8;
                let y = self.registers[y_r as usize] % SCREEN_HEIGHT as u8;
                for row_index in 0..height {
                    let mem_location = self.index_register + row_index as u16;
                    let sprite_row = self.memory[mem_location as usize];
                    for bit_pos in 0..8 {
                        if ((1_u8 << bit_pos) & sprite_row) != 0 {
                            let pix_x = x + 7 - bit_pos;
                            let pix_y = y + row_index;
                            if pix_x >= x && pix_y >= y {
                                self.display[pix_y as usize][pix_x as usize] ^= true;
                            }
                        }
                    }
                }
                return Cycle::RedrawRequested;
            },
            Instruction::SkipPressed { key } => {
                if let Some(k) = key_pressed {
                    if k == key {
                        self.pc += 2;
                    }
                }
            },
            Instruction::SkipNotPressed { key } => {
                if let Some(k) = key_pressed {
                    if k != key {
                        self.pc += 2;
                    }
                } else {
                    self.pc += 2;
                }
            },
            Instruction::AddToIndex { register } => {
                self.index_register += self.registers[register as usize] as u16;
            }
        }
        Cycle::Complete
    }

    pub fn cycle(&mut self, key_pressed: Option<u8>) -> Cycle {
        if !self.pc_inbounds() {
            panic!("PC reached bad value: {}", self.pc);
        }
        let raw_instruction: u16 = self.get_instruction();
        log::debug!("Received raw instruction: {:#04x}", raw_instruction);  
        self.pc += 2;
        if let Some(instruction) = decode(raw_instruction) {
            log::debug!("Received instruction {:?}", instruction);
            return self.execute(instruction, key_pressed);
        } else {
            panic!("Reached unimplemented or invalid instruction: {:#04x}", raw_instruction);
        }
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (y, row) in self.display.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                let i = x * 4 + y * SCREEN_WIDTH * 4;
                frame[i] = if *pixel { u8::MAX } else { 0 };
            }
        }
    }
}

fn clear_terminal() {
    print!("{}[2J", 27 as char);
}

fn render_screen(display: &Screen) {
    for row in display {
        for pixel in row {
            print!("{}", if *pixel { 'Q' } else { ' ' });
        }
        println!("");
    }
}

#[cfg(test)]
mod tests {
    fn init() {
        env_logger::builder().is_test(true).try_init().unwrap();
    }

    use super::{Chip8, Instruction};
    #[test]
    fn draw_tests() {
        init();
        let mut chip8 = Chip8::new();
        chip8.execute(Instruction::Draw { x_r: 0, y_r: 0, height: 5 });
        assert!(chip8.display[0][0]);
        assert!(chip8.display[1][0]);
        assert!(chip8.display[0][1]);
        chip8.execute(Instruction::Draw { x_r: 0, y_r: 0, height: 5 });
        assert!(!chip8.display[0][0]);
        assert!(!chip8.display[1][0]);
        assert!(!chip8.display[0][1]);
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn basic_instructions(v_u12 in 0..(1 << 12))
        {
            let mut chip8 = Chip8::new();
            chip8.execute(Instruction::SetIndexRegister { value: v_u12 as u16 });
            assert_eq!(chip8.index_register as i32, v_u12);
            chip8.execute(Instruction::Jump { dest: v_u12 as u16 });
            assert_eq!(chip8.pc as i32, v_u12);
        }

        #[test]
        fn draw_doesnt_crash(
            a in 0..(1 << 4),
            b in 0..(1 << 4),
            c in 0..(1 << 4),
        ) {
            let mut chip8 = Chip8::new();
            chip8.execute(Instruction::Draw {x_r: a as u8, y_r: b as u8, height:c as u8});
        }
    }
}