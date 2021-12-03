pub unsafe fn userspace_program() {
    asm! {"\
        nop
        nop
        nop"
    }
}
