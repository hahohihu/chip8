mod decode;
mod chip8;

use chip8::{Chip8, render_screen};

fn main() {
    let mut chip8 = Chip8::new();
    let rom_path = std::env::args().nth(1).expect("No ROM given");
    let file = std::fs::File::open(rom_path).expect("Couldn't find ROM path given");
    chip8.read_program(file).expect("Failed to read ROM");
    loop {
        if !chip8.pc_inbounds() {
            panic!("PC reached bad value: {}", chip8.pc);
        }
        let raw_instruction: u16 = chip8.get_instruction();
        // println!("Received raw instruction: {:#04x}", raw_instruction);
        chip8.pc += 2;
        if let Some(instruction) = decode::decode(raw_instruction) {
            // println!("Received instruction {:?}", instruction);
            chip8.execute(instruction);
        } else {
            panic!("Reached unimplemented or invalid instruction: {:#04x}", raw_instruction);
        }
        render_screen(&chip8.display);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
