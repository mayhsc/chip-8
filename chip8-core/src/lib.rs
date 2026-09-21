pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    regs: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            regs: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Execute
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let b0 = self.ram[self.pc as usize] as u16;
        let b1 = self.ram[(self.pc + 1) as usize] as u16;
        let op: u16 = (b0 << 8) | b1;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;
        match (digit1, digit2, digit3, digit4) {
            // 0000 - Nop
            (0, 0, 0, 0) => return,
            // 00E0 - Clear screen
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            // 00EE - Return from Subroutine
            (0, 0, 0xE, 0xE) => self.pc = self.pop(),
            // 1NNN - Jump
            (1, _, _, _) => self.pc = op & 0xFFF,
            // 2NNN - Call Subroutine
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = op & 0xFFF
            }
            // 3XNN - Skip next if VX == NN
            (3, x, _, _) => {
                if self.regs[x as usize] == (op & 0xFF) as u8 {
                    self.pc += 2
                }
            }
            // 4XNN - Skip next if VX != NN
            (4, x, _, _) => {
                if self.regs[x as usize] != (op & 0xFF) as u8 {
                    self.pc += 2
                }
            }
            // 5XY0 - Skip next if VX == VY
            (5, x, y, _) => {
                if self.regs[x as usize] == self.regs[y as usize] {
                    self.pc += 2
                }
            }
            //  6XNN - VX = NN
            (6, x, _, _) => self.regs[x as usize] = (op & 0xFF) as u8,
            // 7XNN - VX += NN
            (7, x, _, _) => {
                self.regs[x as usize] = self.regs[x as usize].wrapping_add((op & 0xFF) as u8)
            }
            // 8XY0 - VX = VY
            (8, x, y, 0) => self.regs[x as usize] = self.regs[y as usize],
            // VX |= VY
            (8, x, y, 1) => {
                self.regs[x as usize] |= self.regs[y as usize];
            }
            // VX &= VY
            (8, x, y, 2) => {
                self.regs[x as usize] &= self.regs[y as usize];
            }
            // VX = !VY
            (8, x, y, 3) => {
                self.regs[x as usize] = !self.regs[y as usize];
            }
            // 8XY4 - VX += VY
            (8, x, y, 4) => {
                let (new_vx, carry) = self.regs[x as usize].overflowing_add(self.regs[y as usize]);
                self.regs[x as usize] = new_vx;
                self.regs[0xF] = if carry { 1 } else { 0 };
            }
            // 8XY5 - VX -= VY
            (8, x, y, 5) => {
                let (new_vx, borrow) = self.regs[x as usize].overflowing_sub(self.regs[y as usize]);
                self.regs[x as usize] = new_vx;
                self.regs[0xF] = if borrow { 0 } else { 1 };
            }
            // 8XY6 - VX >>= 1
            (8, x, _, 6) => {
                let lsb = self.regs[x as usize] & 1;
                self.regs[x as usize] >>= 1;
                self.regs[0xF] = lsb;
            }
            // 8XY7 - VX = VY - VX
            (8, x, y, 7) => {
                let (new_vx, borrow) = self.regs[y as usize].overflowing_sub(self.regs[x as usize]);
                self.regs[x as usize] = new_vx;
                self.regs[0xF] = if borrow { 0 } else { 1 };
            }
            // 8XYE - VX <<= 1
            (8, x, _, 0xE) => {
                let msb = (self.regs[x as usize] >> 7) & 1;
                self.regs[x as usize] <<= 1;
                self.regs[0xF] = msb;
            }
            // 9XY0 - Skip next if VX != VY
            (9, x, y, _) => {
                if self.regs[x as usize] != self.regs[y as usize] {
                    self.pc += 2
                }
            }
            // ANNN - I = NNN
            (0xA, _, _, _) => self.i_reg = op & 0xFFF,
            // BNNN - Jump to V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.regs[0] as u16) + nnn;
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st == 1 {
            // BEEP
        }
        self.st -= 1;
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.regs = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }
}
