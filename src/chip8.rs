use std::io::Read;
use std::num::Wrapping;
use std::time::Duration;
use std::time::Instant;
use crate::bits::{U4, U12};
use crate::decode::decode;
use rand::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    ClearScreen,
    Return,
    Jump { dest: U12 },
    CallSubroutine { dest: U12},
    SkipEQ { register: U4, value: u8 },
    SkipNEQ { register: U4, value: u8 },
    SkipEQR { register1: U4, register2: U4 },
    SkipNEQR { register1: U4, register2: U4 },
    SetRegister { register: U4, value: u8 },
    AddToRegister { register: U4, value: u8 },
    SetIndexRegister { value: U12 },
    MovRegister { register1: U4, register2: U4 },
    BinaryOr { register1: U4, register2: U4 },
    BinaryAnd { register1: U4, register2: U4 },
    BinaryXor { register1: U4, register2: U4 },
    Add { register1: U4, register2: U4 },
    SubtractForward { register1: U4, register2: U4 },
    SubtractBackward { register1: U4, register2: U4 },
    ShiftRight { register1: U4, register2: U4 },
    ShiftLeft { register1: U4, register2: U4 },
    Random { register: U4, value: u8 },
    Draw { x_r: U4, y_r: U4, height: U4 },
    SkipPressed { key: U4 },
    SkipNotPressed { key: U4 },
    GetDelayTimer { register: U4 },
    GetKey { register: U4 },
    FontChar { register: U4 },
    SetDelayTimer { register: U4 },
    SetSoundTimer { register: U4 },
    AddToIndex { register: U4 },
    RegToDecimal { register: U4 },
    StoreMemory { register: U4 },
    LoadMemory { register: U4 },
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
    pub registers: [Wrapping<u8>; 16],
    pub memory: [u8; 4096],
    pub pc: usize,
    pub index_register: Wrapping<u16>,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display: Screen,
    pub stack: Vec<usize>,
    last_clock: Instant,
    rng: StdRng
}

impl Chip8 {
    pub fn new(start: Instant) -> Self {
        let mut chip8 = Chip8 {
            registers: [Wrapping(0); 16],
            memory: [0; 4096],
            pc: INIT_INDEX,
            index_register: Wrapping(0),
            delay_timer: 0,
            sound_timer: 0,
            display: BLANK_SCREEN,
            stack: Vec::new(),
            last_clock: start,
            rng: StdRng::seed_from_u64(0)
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

    pub fn should_beep(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn read_program(&mut self, read: impl std::io::Read) -> Result<usize, std::io::Error> {
        let mut slice = &mut self.memory[INIT_INDEX .. ];
        let mut take = read.take(slice.len() as u64);
        take.read(&mut slice)
    }

    pub fn print_state(&self) {
        log::debug!("====Display=============================");
        print_screen(&self.display);
        log::debug!("====Registers===========================");
        for (i, reg) in self.registers.map(|v| v.0).iter().enumerate() {
            log::debug!("Register {} = {}", i, reg);
        }
        log::debug!("Index = {}", self.index_register.0);
        log::debug!("Delay = {}", self.delay_timer);
        log::debug!("Sound = {}", self.sound_timer);
        log::debug!("====Stack===============================");
        log::debug!("Stack = {:?}", self.stack);
    }
    
    pub fn print_program(&self) {
        log::debug!("====Program=============================");
        for i in (INIT_INDEX..4095).step_by(2) {
            let val1 = self.memory[i];
            let val2 = self.memory[i+1];
            if val1 == 0 && val2 == 0 {
                break;
            }
            let raw = self.memory[i + 1] as u16 | (self.memory[i] as u16) << 8;
            let index = if self.pc == i {
                format!("[{}]", i)
            } else {
                format!("{}", i)
            };
            log::debug!("{}: {:#04x} => {:?}", index, raw, decode(raw));
        }
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
            Instruction::SkipEQ { register, value} => {
                if self.registers[register as usize].0 == value {
                    self.pc += 2;
                }
            },
            Instruction::SkipNEQ { register, value} => {
                if self.registers[register as usize].0 != value {
                    self.pc += 2;
                }
            },
            Instruction::SkipEQR { register1, register2} => {
                if self.registers[register1 as usize] == self.registers[register2 as usize] {
                    self.pc += 2;
                }
            },
            Instruction::SkipNEQR { register1, register2} => {
                if self.registers[register1 as usize] != self.registers[register2 as usize] {
                    self.pc += 2;
                }
            },
            Instruction::SetRegister { register, value } => {
                self.registers[register as usize].0 = value;
            },
            Instruction::AddToRegister { register, value } => {
                self.registers[register as usize] += Wrapping(value);
            },
            Instruction::MovRegister { register1, register2 } => {
                self.registers[register1 as usize] = self.registers[register2 as usize];
            },
            Instruction::BinaryOr { register1, register2 } => {
                self.registers[register1 as usize] |= self.registers[register2 as usize];
            },
            Instruction::BinaryAnd { register1, register2 } => {
                self.registers[register1 as usize] &= self.registers[register2 as usize];
            },
            Instruction::BinaryXor { register1, register2 } => {
                self.registers[register1 as usize] ^= self.registers[register2 as usize];
            },
            Instruction::Add { register1, register2 } => {
                let saved_val = self.registers[register1 as usize];
                self.registers[register1 as usize] += self.registers[register2 as usize];
                // handle overflow
                self.registers[0xf] = Wrapping(if 
                    self.registers[register1 as usize] < saved_val
                    || self.registers[register1 as usize] < self.registers[register1 as usize] 
                    { 1 } else { 0 }
                )
            },
            Instruction::SubtractForward { register1, register2 } => {
                self.registers[register1 as usize] -= self.registers[register2 as usize];
                self.registers[0xf] = Wrapping(
                    if self.registers[register1 as usize] < self.registers[register2 as usize] 
                    { 0 } else { 1 }
                )
            },
            Instruction::SubtractBackward { register1, register2 } => {
                self.registers[register1 as usize] = self.registers[register2 as usize] - self.registers[register1 as usize];
                self.registers[0xf] = Wrapping(
                    if self.registers[register1 as usize] > self.registers[register2 as usize] 
                    { 0 } else { 1 }
                )
            },
            Instruction::ShiftRight { register1, register2: _ } => {
                self.registers[register1 as usize] >>= 1; // different between chip and super-chip
            },
            Instruction::ShiftLeft { register1, register2: _ } => {
                self.registers[register1 as usize] <<= 1; // different between chip and super-chip
            },
            Instruction::SetIndexRegister { value } => {
                self.index_register = Wrapping(value);
            },
            Instruction::Random { register, value } => {
                let num: u8 = self.rng.gen();
                self.registers[register as usize].0 = num & value;
            },
            Instruction::Draw { x_r, y_r, height } => { // TODO: problem is probably here
                let x = self.registers[x_r as usize].0 % SCREEN_WIDTH as u8;
                let y = self.registers[y_r as usize].0 % SCREEN_HEIGHT as u8;
                for row_index in 0..height {
                    let mem_location = self.index_register.0 + row_index as u16;
                    let sprite_row = self.memory[mem_location as usize];
                    for bit_pos in 0..8 {
                        if ((1_u8 << bit_pos) & sprite_row) != 0 {
                            let pix_x = x + 7 - bit_pos;
                            let pix_y = y + row_index;
                            if pix_x < SCREEN_WIDTH as u8 && pix_y < SCREEN_HEIGHT as u8 {
                                self.display[pix_y as usize][pix_x as usize] ^= true;
                            }
                        }
                    }
                }
                return Cycle::RedrawRequested;
            },
            Instruction::SkipPressed { key } => {
                if let Some(k) = key_pressed {
                    if self.registers[key as usize].0 == k {
                        self.pc += 2;
                    }
                }
            },
            Instruction::SkipNotPressed { key } => {
                if let Some(k) = key_pressed {
                    if self.registers[key as usize].0 != k {
                        self.pc += 2;
                    }
                } else {
                    self.pc += 2;
                }
            },
            Instruction::GetDelayTimer { register } => {
                self.registers[register as usize] = Wrapping(self.delay_timer);
            },
            Instruction::GetKey { register } => {
                if let Some(key) = key_pressed {
                    self.registers[register as usize] = Wrapping(key);
                } else {
                    self.pc -= 2;
                }
            },
            Instruction::FontChar { register } => {
                self.index_register = Wrapping((self.registers[register as usize].0 as u16 & 0xf) * 5)
            },
            Instruction::SetDelayTimer { register } => {
                self.delay_timer = self.registers[register as usize].0;
            },
            Instruction::SetSoundTimer { register } => {
                self.sound_timer = self.registers[register as usize].0;
            },
            Instruction::AddToIndex { register } => {
                let saved_val = self.index_register;
                self.index_register += Wrapping(self.registers[register as usize].0 as u16);
                self.registers[0xf] = Wrapping(if self.index_register < saved_val { 1 } else { 0 })
            },
            Instruction::RegToDecimal { register } => {
                let mut val = self.registers[register as usize].0;
                for i in (0..3).rev() {
                    self.memory[self.index_register.0 as usize + i] = val % 10;
                    val /= 10;
                }
            },
            Instruction::StoreMemory { register } => {
                for i in 0..=register as usize {
                    self.memory[self.index_register.0 as usize + i] = 
                        self.registers[i].0;
                }
                // self.index_register += Wrapping(register as u16 + 1); // TODO; this is original behavior, not modern
            },
            Instruction::LoadMemory { register } => {
                for i in 0..=register as usize {
                    self.registers[i].0 = 
                        self.memory[self.index_register.0 as usize + i];
                }
                // self.index_register += Wrapping(register as u16 + 1); // TODO; this is original behavior, not modern
            }
        }
        Cycle::Complete
    }

    pub fn cycle(&mut self, key_pressed: Option<u8>, now: Instant) -> Cycle {
        if !self.pc_inbounds() {
            panic!("PC reached bad value: {}", self.pc);
        }
        key_pressed.map(|k| log::debug!("Key pressed {}", k));
        let time_60hz = Duration::from_secs_f32(1.0) / 60;
        if now.duration_since(self.last_clock) >= time_60hz {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1; // TODO: beep
            }
            self.last_clock = now;
        }
        let raw_instruction: u16 = self.get_instruction();
        self.pc += 2;
        if let Some(instruction) = decode(raw_instruction) {
            return self.execute(instruction, key_pressed);
        } else {
            panic!("Reached unimplemented or invalid instruction: {:#04x} at PC {}", raw_instruction, self.pc);
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

fn print_screen(display: &Screen) {
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
        env_logger::builder()
            .is_test(true)
            .parse_env(env_logger::Env::default()
                .filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"))
            .try_init()
            .unwrap();
    }

    use std::num::Wrapping;
    use std::time::{Duration, Instant};

    use super::{Chip8, Instruction, print_screen};
    #[test]
    fn draw_tests() {
        init();
        let mut chip8 = Chip8::new(Instant::now());
        chip8.execute(Instruction::Draw { x_r: 0, y_r: 0, height: 5 }, None);
        assert!(chip8.display[0][0]);
        assert!(chip8.display[1][0]);
        assert!(chip8.display[0][1]);
        chip8.execute(Instruction::Draw { x_r: 0, y_r: 0, height: 5 }, None);
        assert!(!chip8.display[0][0]);
        assert!(!chip8.display[1][0]);
        assert!(!chip8.display[0][1]);
    }

    #[test]
    fn num_tests() {
        init();
        let mut chip8 = Chip8::new(Instant::now());
        chip8.execute(Instruction::SetRegister { register: 0, value: 123 }, None);
        chip8.execute(Instruction::SetIndexRegister { value: 0x400 }, None);
        chip8.execute(Instruction::RegToDecimal { register: 0 }, None);
        assert_eq!(chip8.memory[0x400], 1);
        assert_eq!(chip8.memory[0x401], 2);
        assert_eq!(chip8.memory[0x402], 3);
        chip8.execute(Instruction::SetRegister { register: 0, value: 10 }, None);
        chip8.execute(Instruction::SetIndexRegister { value: 0x400 }, None);
        chip8.execute(Instruction::RegToDecimal { register: 0 }, None);
        assert_eq!(chip8.memory[0x400], 0);
        assert_eq!(chip8.memory[0x401], 1);
        assert_eq!(chip8.memory[0x402], 0);
    }

    #[test]
    fn rom_test() {
        let mut chip8 = Chip8::new(Instant::now());
        chip8.read_program(std::fs::File::open("/home/rachel/Downloads/Chip-8 Pack/Chip-8 Games/15 Puzzle [Roger Ivie].ch8").unwrap()).unwrap();
        let mut now = Instant::now();
        for i in 0..10000 {
            now += Duration::from_secs(1);
            chip8.cycle(Some(4), now);
            print_screen(&chip8.display);
        }
    }

    use proptest::prelude::*;
    use rand::prelude::*;
    proptest! {
        #[test]
        fn instruction_tests(
            a in 0..u8::MAX,
            b in 0..u8::MAX,
            r1 in 0..15 as u8,
            r2 in 0..15 as u8
        ) {
            let mut chip8 = Chip8::new(Instant::now());
            chip8.execute(Instruction::SetRegister { register: r1, value: a as u8 }, None);
            assert_eq!(chip8.registers[r1 as usize].0, a as u8);
            chip8.execute(Instruction::SetRegister { register: r2, value: b as u8 }, None);
            assert_eq!(chip8.registers[r2 as usize].0, b as u8);
            chip8.execute(Instruction::MovRegister { register1: r1, register2: r2 }, None);
            assert_eq!(chip8.registers[r1 as usize], chip8.registers[r2 as usize]);
            chip8.execute(Instruction::Add { register1: r1, register2: r2 }, None);
            assert_eq!(chip8.registers[r1 as usize], Wrapping(b) + Wrapping(b));
        }

        #[test]
        fn draw_doesnt_crash(
            a in 0..(1 << 4),
            b in 0..(1 << 4),
            c in 0..(1 << 4),
        ) {
            let mut chip8 = Chip8::new(Instant::now());
            chip8.execute(Instruction::Draw {x_r: a as u8, y_r: b as u8, height:c as u8}, None);
        }

        #[test]
        fn memory_bothways(
            register in 0..(1 << 4) as u8,
            mem in 0..(1 << 11) as u16,
            seed in 0..32 as u64
        ) {
            let mut rng = StdRng::seed_from_u64(seed);
            let mut chip8 = Chip8::new(Instant::now());
            let mut vals = Vec::new();
            for i in 0..=register {
                let value = rng.gen();
                vals.push(value);
                chip8.execute(Instruction::SetRegister { register: i, value }, None);
                assert_eq!(chip8.registers[i as usize].0, value);
            }
            chip8.execute(Instruction::SetIndexRegister { value: mem }, None);
            assert_eq!(chip8.index_register.0, mem);
            chip8.execute(Instruction::StoreMemory { register: register }, None);
            for i in 0..=register {
                assert_eq!(vals[i as usize], chip8.memory[(mem + i as u16) as usize]);
                chip8.execute(Instruction::SetRegister { register: i , value: 0 }, None);
                assert_eq!(chip8.registers[i as usize].0, 0);
            }
            chip8.execute(Instruction::LoadMemory { register: register }, None);
            for i in 0..=register {
                assert_eq!(chip8.registers[i as usize].0, vals[i as usize]);
            }
        }
    }
}