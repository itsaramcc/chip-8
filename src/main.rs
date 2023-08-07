// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

#![allow(arithmetic_overflow)]

use std::{ fs::File, io::Read };
use rand::{ thread_rng, Rng };
use minifb::{Key, Window, WindowOptions, Scale};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const CHIP8_FONTSET: [u8; 80] = [ 
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
	opcode: u16,
	memory: [u8; 4096],
	v: [u8; 16],			// Registers
	i: u16,					// Index Register
	pc: u16,

	gfx: [u8; 2048],

	delay_timer: u8,
	sound_timer: u8,

	stack: [u16; 16],
	sp: u16,

	keys: [u8; 16],

}

impl Chip8 {
	fn initialize() -> Self {
		// Initialize registers and memory once

		let pc = 0x200;	// Program counter starts at 0x200
		let opcode = 0;	// Reset current opcode	
		let i = 0;			// Reset index register
		let sp = 0; 		// Rest stack pointer

		// Clear display
		let gfx = [0; 2048];
		// Clear stack
		let stack = [0; 16];	
		// Clear registers V0-VF
		let v = [0; 16];
		// Clear memory
		let mut memory = [0; 4096];

		// Load fontset
		for i in 0..80 {
			memory[i] = CHIP8_FONTSET[i];	
		}

		let keys = [0; 16];

		// Reset timers
		let delay_timer = 0;
		let sound_timer = 0;

		Self { opcode, memory, v, i, pc, gfx, delay_timer, sound_timer, stack, sp, keys }
	}

	fn load_rom(&mut self, path: &str) {
		let mut file = File::open(path).unwrap();
		let mut buffer: Vec<u8> = vec![];
		file.read_to_end(&mut buffer).unwrap();

		for i in 0..buffer.len() {
			self.memory[i + 0x200] = buffer[i];
		}
	}
	
	fn cycle(&mut self) {
		// Fetch Opcode
		self.opcode = ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[self.pc as usize + 1] as u16);

		// Decode Opcode
		match self.opcode & 0xF000 {
			0x0000 => {
				if self.opcode == 0x00E0 {				// 00E0 - CLS
					self.gfx = [0; 2048];
				}
				else if self.opcode == 0x00EE {			// 00EE - RET
					self.sp -= 1;
					self.pc = self.stack[self.sp as usize];
				}
				else { }								// 0nnn - SYS addr

				self.pc += 2;
			},
			0x1000 => self.pc = self.opcode & 0x0FFF,	// 1nnn - JP addr
			0x2000 => {									// 2nnn - CALL addr
				self.stack[self.sp as usize] = self.pc; 
				self.sp += 1;
				self.pc = self.opcode & 0x0FFF;
			},
			0x3000 => {									// 3xkk - SE Vx, byte
				let x = (self.opcode & 0x0F00) >> 8;
				if self.v[x as usize] == (self.opcode & 0x00FF) as u8 {
					self.pc += 2;
				}
				self.pc += 2;
			},
			0x4000 => {									// 4xkk - SNE Vx, byte
				let x = (self.opcode & 0x0F00) >> 8;
				if self.v[x as usize] != (self.opcode & 0x00FF) as u8 {
					self.pc += 2;
				}
				self.pc += 2;
			},
			0x5000 => {									// 5xy0 - SE Vx, Vy
				let x = (self.opcode & 0x0F00) >> 8;
				let y = (self.opcode & 0x00F0) >> 4;
				if self.v[x as usize] == self.v[y as usize] {
					self.pc += 2;
				}
				self.pc += 2;
			},
			0x6000 => {									// 6xkk - LD Vx, byte
				let x = (self.opcode & 0x0F00) >> 8;
				self.v[x as usize] = (self.opcode & 0x0FF) as u8;
				self.pc += 2;
			},
			0x7000 => {									// 7xkk - ADD Vx, byte
				let x = (self.opcode & 0x0F00) >> 8;
				self.v[x as usize] = self.v[x as usize].overflowing_add(self.opcode as u8 & 0x0FF).0;
				self.pc += 2;
			},
			0x8000 => {									// 8xyn - ALU operations
				let x = ((self.opcode & 0x0F00) >> 8) as usize;
				let y = ((self.opcode & 0x00F0) >> 4) as usize;
				match self.opcode & 0x000F {
					0x0 =>  self.v[x] = self.v[y],		// 8xy0 - LD Vx, Vy
					0x1 => self.v[x] |= self.v[y],		// 8xy1 - OR Vx, Vy
					0x2 => self.v[x] &= self.v[y],		// 8xy2 - AND Vx, Vy
					0x3 => self.v[x] ^= self.v[y],		// 8xy3 - XOR Vx, Vy
					
					0x4 => {							// 8xy4 - ADD Vx, Vy
						let mut val = self.v[x] as u16;
						val += self.v[y] as u16;

						self.v[0xF] = if val > 255 { 1 } else { 0 };
						self.v[x] = val as u8;
					},	
					0x5 => {							// 8xy5 - SUB Vx, Vy
						self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
						self.v[x] = self.v[x].overflowing_sub(self.v[y]).0;
					},	
					0x6 => {							// 8xy6 - SHR Vx {, Vy}
						self.v[0xF] = self.v[x] & 1;
						self.v[x] >>= 1;
					},
					0x7 => {							// 8xy7 - SUBN Vx, Vy
						self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
						self.v[x] = self.v[y].overflowing_sub(self.v[x]).0;
					},	
					0xE => {							// 8xyE - SHL Vx {, Vy}
						self.v[0xF] = if self.v[x] & 0x80 !=0 { 1 } else { 0 };
						let val = self.v[x] as u16 * 2;
						self.v[x] = val as u8;
					},

					_ => println!("Unkown opcode: {:#06x}", self.opcode)
				}
				self.pc += 2;
			},
			0x9000 => { 								// 9xy0 - SNE Vx, Vy
				let x = (self.opcode & 0x0F00) >> 8;
				let y = (self.opcode & 0x00F0) >> 4;
				if self.v[x as usize] != self.v[y as usize] {
					self.pc += 2;
				}
				self.pc += 2;
			},
			0xA000 => { 								// Annn - LD I, addr
				self.i = self.opcode & 0x0FFF;
				self.pc += 2;
			},
			0xB000 =>{									// Bnnn - JP V0, addr
				self.pc = (self.opcode & 0x0FFF) + self.v[0] as u16;
			},
			0xC000 => { 								// Cxkk - RND Vx, byte
				let x = (self.opcode & 0x0F00) >> 8;
				self.v[x as usize] =  thread_rng().gen::<u8>() & (self.opcode & 0x00FF) as u8;
				self.pc += 2;
			},
			0xD000 => { 								//  Dxyn - DRW Vx, Vy, nibble
				self.v[0xF] = 0;

				let x = ((self.opcode & 0x0F00) >> 8) as usize;
				let y = ((self.opcode & 0x00F0) >> 4) as usize;
				let n = (self.opcode & 0x000F) as u8;

				for y_line in 0..n {
					let pixel = self.memory[(self.i + y_line as u16) as usize];
					for x_line in 0..8 {
						if pixel & (0x80 >> x_line) != 0 {
							let index = ((self.v[x] + x_line) as usize % 64 + ((self.v[y] + y_line) as usize % 32) * 64) as usize;

							if self.gfx[index] == 1 {
								self.v[0xF] = 1;
							}
							self.gfx[index] ^= 1;
						}
					}
				}
				self.pc += 2;
			},
			0xE000 => { 
				match self.opcode & 0x00FF {
					0x9E => {							// Ex9E - SKP Vx
						let x = (self.opcode & 0x0F00) >> 8;
						if self.keys[self.v[x as usize] as usize] != 0 {
							self.pc += 2;
						}
					},
					0xA1 => {							// ExA1 - SKNP Vx
						let x = (self.opcode & 0x0F00) >> 8;
						if self.keys[self.v[x as usize] as usize] == 0 {
							self.pc += 2;
						}
					},

					_ => println!("Unkown opcode: {:#06x}", self.opcode),
				}
				self.pc += 2;
			},
			0xF000 => { 
				let x = ((self.opcode & 0x0F00) >> 8) as usize;
				match self.opcode & 0x00FF {
					0x07 => 							// Fx07 - LD Vx, DT
						self.v[x] = self.delay_timer,
					0x0A => {							// Fx0A - LD Vx, K
						let mut key_pressed = false;
						for (idx, key) in self.keys.iter().enumerate() {
							if *key != 0 {
								self.v[x] = idx as u8;
								key_pressed = true;
								break;
							}
						}

						if !key_pressed { return; }
					},
					0x15 =>								// Fx15 - LD DT, Vx
						self.delay_timer = self.v[x],
					0x18 =>								// Fx18 - LD ST, Vx
						self.sound_timer = self.v[x],
					0x1E =>								// Fx1E - ADD I, Vx
						self.i += self.v[x] as u16,
					0x29 => {							// Fx29 - LD F, Vx
						if self.v[x] < 16 {
							self.i = self.v[x] as u16 * 0x5;
						}
					},
					0x33 => {							// Fx33 - LD B, Vx
						self.memory[self.i as usize]    = self.v[x] /100;
						self.memory[self.i as usize +1] = (self.v[x] /10) %10;
						self.memory[self.i as usize +2] = self.v[x] %10;
					},
					0x55 => {							// Fx55 - LD [I], Vx
						for i in 0..(x+1) {
							self.memory[self.i as usize +i] = self.v[i];
						}
					},
					0x65 => {							//  Fx65 - LD Vx, [I]
						for i in 0..(x+1) {
							self.v[i] = self.memory[self.i as usize +i];
						}
					},

					_ => println!("Unkown opcode: {:#06x}", self.opcode),
				}
				self.pc += 2;
			},

			_ => println!("Unkown opcode: {:#06x}", self.opcode)
		}

		// Update timers
		if self.delay_timer > 0 {
			self.delay_timer -= 1;
		}

		if self.sound_timer > 0 {
			if self.sound_timer == 1 {
				println!("Beep!");
				self.sound_timer -= 1;
			}
		}

	}
}

fn main() {

	let mut emulator = Chip8::initialize();
	emulator.load_rom("rom.ch8");

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions { 
			scale: Scale::X16,
			..WindowOptions::default()
		 },
    )
    .unwrap();

	window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

	while window.is_open() && !window.is_key_down(Key::Escape) {
        emulator.cycle();

		// Graphics
		for i in 0..2048 {
			buffer[i] = emulator.gfx[i] as u32 * 0xFFFFFFFF;
		}


		// Input
		window.get_keys().iter().for_each(|key| match key {
            Key::Key0 => emulator.keys[0x0] = 1,
            Key::Key1 => emulator.keys[0x1] = 1,
			Key::Key2 => emulator.keys[0x2] = 1,
			Key::Key3 => emulator.keys[0x3] = 1,
			Key::Key4 => emulator.keys[0x4] = 1,
            Key::Key5 => emulator.keys[0x5] = 1,
			Key::Key6 => emulator.keys[0x6] = 1,
			Key::Key7 => emulator.keys[0x7] = 1,
			Key::Key8 => emulator.keys[0x8] = 1,
            Key::Key9 => emulator.keys[0x9] = 1,
			Key::A    => emulator.keys[0xa] = 1,
			Key::B    => emulator.keys[0xb] = 1,
			Key::C	  => emulator.keys[0xc] = 1,
            Key::D    => emulator.keys[0xd] = 1,
			Key::E    => emulator.keys[0xe] = 1,
			Key::F    => emulator.keys[0xf] = 1,
            _ => (),
        });

        window.get_keys_released().iter().for_each(|key| match key {
            Key::Key0 => emulator.keys[0x0] = 0,
            Key::Key1 => emulator.keys[0x1] = 0,
			Key::Key2 => emulator.keys[0x2] = 0,
			Key::Key3 => emulator.keys[0x3] = 0,
			Key::Key4 => emulator.keys[0x4] = 0,
            Key::Key5 => emulator.keys[0x5] = 0,
			Key::Key6 => emulator.keys[0x6] = 0,
			Key::Key7 => emulator.keys[0x7] = 0,
			Key::Key8 => emulator.keys[0x8] = 0,
            Key::Key9 => emulator.keys[0x9] = 0,
			Key::A    => emulator.keys[0xa] = 0,
			Key::B    => emulator.keys[0xb] = 0,
			Key::C	  => emulator.keys[0xc] = 0,
            Key::D    => emulator.keys[0xd] = 0,
			Key::E    => emulator.keys[0xe] = 0,
			Key::F    => emulator.keys[0xf] = 0,
            _ => (),
        });


        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }

}
