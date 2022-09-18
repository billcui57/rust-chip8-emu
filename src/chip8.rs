use std::fs;
#[cfg(test)]
mod tests;

pub struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delayTimer: u8,
    soundTimer: u8,
    keypad: [u8; 16],
    video: [u32; 64 * 32],
}

const START_ADDRESS: u16 = 0x200;

const FONTSET_SIZE: u16 = 80;

const FONTSET_START_ADDRESS: u16 = 0x50;

const fontset: [u8; FONTSET_SIZE as usize] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

trait Mem {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let hi = self.mem_read(pos) as u16;
        let lo = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let lo = (data >> 8) as u8;
        let hi = (data & 0xff) as u8;
        self.mem_write(pos, hi);
        self.mem_write(pos + 1, lo);
    }
}

impl Mem for Chip8 {
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

enum Instruction {
    CLS,
    RET,
    JP { addr: u16 },
    BRK,
    CALL { addr: u16 },
    SE_IMD { cmp_val: u8, reg_num: u8 },
    SNE_IMD { cmp_val: u8, reg_num: u8 },
    SE_REG { reg_num_x: u8, reg_num_y: u8 },
    LD_IMD { reg_num: u8, byte: u8 },
    ADD_IMD { reg_num: u8, byte: u8 },
}

impl Chip8 {
    pub fn new() -> Self {
        let mut new_chip8 = Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: 0,
            stack: [0; 16],
            sp: 0,
            delayTimer: 0,
            soundTimer: 0,
            keypad: [0; 16],
            video: [0; 64 * 32],
        };

        new_chip8.reset();

        new_chip8
    }
    pub fn load(&mut self, program: &Vec<u16>) {
        let u8program = program
            .into_iter()
            .flat_map(|inst| u16::to_be_bytes(*inst))
            .collect::<Vec<u8>>();

        self.memory[START_ADDRESS as usize..((START_ADDRESS as usize + u8program.len()) as usize)]
            .copy_from_slice(u8program.as_slice());
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDRESS;
        self.memory
            [(FONTSET_START_ADDRESS as usize)..((FONTSET_START_ADDRESS + FONTSET_SIZE) as usize)]
            .copy_from_slice(&fontset);
    }

    pub fn decode(&mut self, instr: u16) -> Instruction {
        let opcode = instr >> (3 * 4);

        if opcode == 0x0 {
            if instr == 0x00E0 {
                return Instruction::CLS;
            }
            if instr == 0x00EE {
                return Instruction::RET;
            }
        }

        if opcode == 0x1 {
            let param = instr & 0x0FFF;

            return Instruction::JP { addr: param };
        }

        if opcode == 0x2 {
            let param = instr & 0x0FFF;
            return Instruction::CALL { addr: param };
        }

        if opcode == 0x3 {
            let reg_num = ((instr & 0x0F00) >> 8) as u8;
            let cmp_val = (instr & 0x00FF) as u8;
            return Instruction::SE_IMD {
                cmp_val: cmp_val,
                reg_num: reg_num,
            };
        }

        if opcode == 0x4 {
            let reg_num = ((instr & 0x0F00) >> 8) as u8;
            let cmp_val = (instr & 0x00FF) as u8;
            return Instruction::SNE_IMD {
                cmp_val: cmp_val,
                reg_num: reg_num,
            };
        }

        if opcode == 0x5 {
            let reg_num_x = ((instr & 0x0F00) >> 8) as u8;
            let reg_num_y = ((instr & 0x00F0) >> 4) as u8;
            return Instruction::SE_REG {
                reg_num_x: reg_num_x,
                reg_num_y: reg_num_y,
            };
        }

        if opcode == 0x6 {
            let reg_num = ((instr & 0x0F00) >> 8) as u8;
            let byte = (instr & 0x00FF) as u8;

            return Instruction::LD_IMD {
                reg_num: reg_num,
                byte: byte,
            };
        }

        if opcode == 0x7 {
            let reg_num = ((instr & 0x0F00) >> 8) as u8;
            let byte = (instr & 0x00FF) as u8;

            return Instruction::ADD_IMD {
                reg_num: reg_num,
                byte: byte,
            };
        }

        Instruction::BRK
    }

    pub fn run(&mut self) {
        loop {
            let instr = self.decode(self.mem_read_u16(self.pc));

            self.pc += 2;

            match instr {
                Instruction::CLS => self.cls(),
                Instruction::BRK => break,
                Instruction::RET => self.ret(),
                Instruction::JP { addr } => self.jp(addr),
                Instruction::CALL { addr } => self.call(addr),
                Instruction::SE_IMD { cmp_val, reg_num } => self.se_imd(reg_num, cmp_val),
                Instruction::SNE_IMD { cmp_val, reg_num } => self.sne_imd(reg_num, cmp_val),
                Instruction::SE_REG {
                    reg_num_x,
                    reg_num_y,
                } => self.se_reg(reg_num_x, reg_num_y),
                Instruction::LD_IMD { reg_num, byte } => self.ld_imd(reg_num, byte),
                Instruction::ADD_IMD { reg_num, byte } => self.add_imd(reg_num, byte),
                _ => panic!("Shouldnt be here"),
            }
        }
    }

    fn push(&mut self, val: u16) {
        assert!(self.sp < 16);
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        assert!(self.sp != 0);
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn cls(&mut self) {
        self.video.fill(0);
    }

    fn ret(&mut self) {
        self.pc = self.pop();
    }

    fn jp(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn call(&mut self, addr: u16) {
        self.push(self.pc);
        self.pc = addr;
    }

    fn se_imd(&mut self, reg_num: u8, cmp_val: u8) {
        if self.registers[reg_num as usize] == cmp_val {
            self.pc += 2;
        }
    }

    fn sne_imd(&mut self, reg_num: u8, cmp_val: u8) {
        if self.registers[reg_num as usize] != cmp_val {
            self.pc += 2;
        }
    }

    fn se_reg(&mut self, reg_num_x: u8, reg_num_y: u8) {
        if self.registers[reg_num_x as usize] == self.registers[reg_num_y as usize] {
            self.pc += 2;
        }
    }

    fn ld_imd(&mut self, reg_num: u8, byte: u8) {
        self.registers[reg_num as usize] = byte;
    }

    fn add_imd(&mut self, reg_num: u8, byte: u8) {
        self.registers[reg_num as usize] += byte;
    }

    fn ld_reg(&mut self, reg_num_x: u8, reg_num_y: u8) {}
}
