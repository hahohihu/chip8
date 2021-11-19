mod decode;
mod types;

use std::{io::Read};
use types::Instruction;

const INIT_INDEX: usize = 0x200;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
type Screen = [[bool; SCREEN_HEIGHT]; SCREEN_WIDTH];
const BLANK_SCREEN: Screen = [[false; SCREEN_HEIGHT]; SCREEN_WIDTH];
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

struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    pc: usize,
    index_register: u16,
    delay_timer: u8,
    sound_timer: u8,
    display: Screen
}

impl Chip8 {
    fn get_instruction(&self) -> u16 {
        self.memory[self.pc + 1] as u16 | (self.memory[self.pc] as u16) << 8
    }

    fn pc_inbounds(&self) -> bool {
        self.pc >= INIT_INDEX && self.pc < 4095
    }
    
    fn print_program(&self) {
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
}

fn execute(instruction: Instruction, chip8: &mut Chip8) {
    match instruction {
        Instruction::ClearScreen => {
            chip8.display = BLANK_SCREEN;
        },
        Instruction::Jump { dest } => {
            chip8.pc = dest as usize;
        },
        Instruction::SetRegister { register, value } => {
            chip8.registers[register as usize] = value;
        },
        Instruction::AddToRegister { register, value } => {
            chip8.registers[register as usize] += value; // TODO: not clear if this should set carry flag
        },
        Instruction::SetIndexRegister { value } => {
            chip8.index_register = value;
        },
        Instruction::Draw { x_r, y_r, height } => { // TODO: problem is probably here
            let x = chip8.registers[x_r as usize] % SCREEN_WIDTH as u8;
            let y = chip8.registers[y_r as usize] % SCREEN_HEIGHT as u8;
            println!("Sprite: {}", height);
            for row_index in 0..height {
                let mem_location = chip8.index_register + row_index as u16;
                let sprite_row = chip8.memory[mem_location as usize];
                println!("{:08b}", sprite_row);
                for bit_pos in 0..8 {
                    if ((1_u8 << bit_pos) & sprite_row) != 0 {
                        let pix_x = x + bit_pos;
                        let pix_y = y + row_index;
                        if pix_x >= x && pix_y >= y {
                            chip8.display[pix_x as usize][pix_y as usize] ^= true;
                        }
                    }
                }
            }
        },
    }
}

fn clear_terminal() {
    print!("{}[2J", 27 as char);
}

fn render_screen(display: &Screen) {
    // clear_terminal();
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            print!("{}", if display[x][y] { 'A' } else { ' ' });
        }
        println!("");
    }
}

fn main() {
    let mut chip8: Chip8 = Chip8 {
        registers: [0; 16],
        memory: [0; 4096],
        pc: INIT_INDEX,
        index_register: 0,
        delay_timer: 0,
        sound_timer: 0,
        display: BLANK_SCREEN
    };
    chip8.memory[0..FONT.len()].copy_from_slice(&FONT);
    let rom_path = std::env::args().nth(1).expect("No ROM given");
    let file = std::fs::File::open(rom_path).expect("Couldn't find ROM path given");
    let mut slice = &mut chip8.memory[INIT_INDEX .. ];
    let mut handle = file.take(slice.len() as u64);
    handle.read(&mut slice).expect("Failed to read ROM");
    loop {
        if !chip8.pc_inbounds() {
            panic!("PC reached bad value: {}", chip8.pc);
        }
        let raw_instruction: u16 = chip8.get_instruction();
        // println!("Received raw instruction: {:#04x}", raw_instruction);
        chip8.pc += 2;
        if let Some(instruction) = decode::decode(raw_instruction) {
            // println!("Received instruction {:?}", instruction);
            execute(instruction, &mut chip8);
        } else {
            panic!("Reached unimplemented or invalid instruction: {:#04x}", raw_instruction);
        }
        render_screen(&chip8.display);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
