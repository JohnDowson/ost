use x86_64::instructions::{interrupts::without_interrupts, port::Port};

pub fn init() {
    let mut port0x70 = Port::new(0x70);
    let mut port0x71 = Port::new(0x71);
    unsafe {
        without_interrupts(|| {
            port0x70.write(0x8Au8);
            port0x71.write(0b0_010_0110u8);

            port0x70.write(0x8Bu8);
            port0x71.write(0b0_1_0_1_0000);
        })
    }
}
