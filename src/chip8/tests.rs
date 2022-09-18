mod tests {
    use std::vec;

    use crate::chip8::{self, Chip8};

    fn make_video_dirty(chip8: &mut Chip8) {
        chip8.video.fill(4);
    }

    fn is_video_cleared(chip8: &Chip8) -> bool {
        return chip8.video == [0; 64 * 32];
    }

    #[test]
    fn cls() {
        let mut chip8 = Chip8::new();
        let program: Vec<u16> = vec![0x00E0];
        chip8.load(&program);
        make_video_dirty(&mut chip8);
        chip8.run();
        assert!(is_video_cleared(&chip8));
    }
    #[test]
    fn ret() {
        let mut chip8 = Chip8::new();
        chip8.push(0xFF0);
        let program: Vec<u16> = vec![0x00E0, 0x00E0, 0x00EE];
        chip8.load(&program);
        chip8.run();
        assert!(chip8.pc == 0xFF2);
    }

    #[test]
    fn jp() {
        let mut chip8 = Chip8::new();
        let program: Vec<u16> = vec![0x1204, 0x00E0];
        make_video_dirty(&mut chip8);
        chip8.load(&program);
        chip8.run();
        assert!(!is_video_cleared(&chip8));
    }

    #[test]
    fn call() {
        let mut chip8 = Chip8::new();
        let program: Vec<u16> = vec![0x2204, 0x00E0, 0x00E0];
        chip8.load(&program);
        chip8.run();
        assert!(chip8.pop() == 0x202);
    }

    #[test]
    fn seimd_eq() {
        let mut chip8 = Chip8::new();
        chip8.registers[4] = 17;
        let program: Vec<u16> = vec![0x3411, 0x00E0];
        make_video_dirty(&mut chip8);
        chip8.load(&program);
        chip8.run();
        assert!(!is_video_cleared(&chip8));
    }

    #[test]
    fn seimd_neq() {
        let mut chip8 = Chip8::new();
        chip8.registers[4] = 16;
        let program: Vec<u16> = vec![0x3411, 0x00E0];
        make_video_dirty(&mut chip8);
        chip8.load(&program);
        chip8.run();
        assert!(is_video_cleared(&chip8));
    }

    #[test]
    fn sneimd_eq() {
        let mut chip8 = Chip8::new();
        chip8.registers[4] = 17;
        let program: Vec<u16> = vec![0x4411, 0x00E0];
        make_video_dirty(&mut chip8);
        chip8.load(&program);
        chip8.run();
        assert!(is_video_cleared(&chip8));
    }

    #[test]
    fn sneimd_neq() {
        let mut chip8 = Chip8::new();
        chip8.registers[4] = 16;
        let program: Vec<u16> = vec![0x4411, 0x00E0];
        make_video_dirty(&mut chip8);
        chip8.load(&program);
        chip8.run();
        assert!(!is_video_cleared(&chip8));
    }

    #[test]
    fn sereg_eq() {
        let mut chip8 = Chip8::new();
        chip8.registers[4] = 17;
        chip8.registers[3] = 17;
        let program: Vec<u16> = vec![0x5430, 0x00E0];
        make_video_dirty(&mut chip8);
        chip8.load(&program);
        chip8.run();
        assert!(!is_video_cleared(&chip8));
    }

    #[test]
    fn sereg_neq() {
        let mut chip8 = Chip8::new();
        chip8.registers[4] = 17;
        chip8.registers[3] = 16;
        let program: Vec<u16> = vec![0x5430, 0x00E0];
        make_video_dirty(&mut chip8);
        chip8.load(&program);
        chip8.run();
        assert!(is_video_cleared(&chip8));
    }

    #[test]
    fn ldimd() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 7;
        let program: Vec<u16> = vec![0x6019];
        chip8.load(&program);
        chip8.run();
        assert!(chip8.registers[0] == 0x19);
        for i in 1..chip8.registers.len() {
            assert!(chip8.registers[i as usize] == 0);
        }
    }

    #[test]
    fn addimd() {
        let mut chip8 = Chip8::new();
        chip8.registers[0] = 7;
        let program: Vec<u16> = vec![0x7019];
        chip8.load(&program);
        chip8.run();
        assert!(chip8.registers[0] == 0x19 + 0x7);
        for i in 1..chip8.registers.len() {
            assert!(chip8.registers[i as usize] == 0);
        }
    }
}
