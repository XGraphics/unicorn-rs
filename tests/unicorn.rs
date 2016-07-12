extern crate unicorn;

use unicorn::{Cpu, CpuX86, CpuARM, CpuMIPS, uc_handle};

#[test]
fn emulate_x86() {
    let x86_code32: Vec<u8> = vec![0x41, 0x4a]; // INC ecx; DEC edx

    let mut emu = CpuX86::new(unicorn::Mode::MODE_32).expect("failed to instantiate emulator");
    assert_eq!(emu.reg_write(unicorn::RegisterX86::EAX, 123), Ok(()));
    assert_eq!(emu.reg_read(unicorn::RegisterX86::EAX), Ok((123)));

    // Attempt to write to memory before mapping it.
    assert_eq!(emu.mem_write(0x1000, &x86_code32),
               (Err(unicorn::Error::WRITE_UNMAPPED)));

    assert_eq!(emu.mem_map(0x1000, 0x4000, unicorn::PROT_ALL), Ok(()));
    assert_eq!(emu.mem_write(0x1000, &x86_code32), Ok(()));
    assert_eq!(emu.mem_read(0x1000, x86_code32.len()),
               Ok(x86_code32.clone()));

    assert_eq!(emu.reg_write(unicorn::RegisterX86::ECX, 10), Ok(()));
    assert_eq!(emu.reg_write(unicorn::RegisterX86::EDX, 50), Ok(()));

    assert_eq!(emu.emu_start(0x1000,
                             (0x1000 + x86_code32.len()) as u64,
                             10 * unicorn::SECOND_SCALE,
                             1000),
               Ok(()));
    assert_eq!(emu.reg_read(unicorn::RegisterX86::ECX), Ok((11)));
    assert_eq!(emu.reg_read(unicorn::RegisterX86::EDX), Ok((49)));
}



#[test]
fn emulate_x86_negative_values() {
    let x86_code32: Vec<u8> = vec![0x41, 0x4a]; // INC ecx; DEC edx

    let emu = CpuX86::new(unicorn::Mode::MODE_32).expect("failed to instantiate emulator");

    assert_eq!(emu.mem_map(0x1000, 0x4000, unicorn::PROT_ALL), Ok(()));
    assert_eq!(emu.mem_write(0x1000, &x86_code32), Ok(()));

    assert_eq!(emu.reg_write_i32(unicorn::RegisterX86::ECX, -10), Ok(()));
    assert_eq!(emu.reg_write_i32(unicorn::RegisterX86::EDX, -50), Ok(()));

    assert_eq!(emu.emu_start(0x1000,
                             (0x1000 + x86_code32.len()) as u64,
                             10 * unicorn::SECOND_SCALE,
                             1000),
               Ok(()));
    assert_eq!(emu.reg_read_i32(unicorn::RegisterX86::ECX), Ok((-9)));
    assert_eq!(emu.reg_read_i32(unicorn::RegisterX86::EDX), Ok((-51)));
}

#[test]
fn x86_code_callback() {
    #[allow(unused_variables)]
    extern "C" fn callback(engine: uc_handle, address: u64, size: u32, user_data: *mut u64) {
        println!("in callback at 0x{:08x}!", address);
    }

    let x86_code32: Vec<u8> = vec![0x41, 0x4a]; // INC ecx; DEC edx

    let emu = CpuX86::new(unicorn::Mode::MODE_32).expect("failed to instantiate emulator");
    assert_eq!(emu.mem_map(0x1000, 0x4000, unicorn::PROT_ALL), Ok(()));
    assert_eq!(emu.mem_write(0x1000, &x86_code32), Ok(()));

    let hook = emu.add_code_hook(unicorn::HookType::BLOCK, 0x1000, 0x2000, callback)
        .expect("failed to add code hook");
    assert_eq!(emu.emu_start(0x1000, 0x1001, 10 * unicorn::SECOND_SCALE, 1000),
               Ok(()));
    assert_eq!(emu.remove_hook(hook), Ok(()));

}

#[test]
fn x86_mem_callback() {
    #[allow(unused_variables)]
    extern "C" fn callback(engine: uc_handle,
                           mem_type: unicorn::MemType,
                           address: u64,
                           size: i32,
                           value: i64,
                           user_data: *mut u64) {
        println!("unmapped mem read at 0x{:08x}!", address);
    }

    let x86_code32: Vec<u8> = vec![0x8b, 0x00]; // MOV eax, dword [eax]

    let mut emu = CpuX86::new(unicorn::Mode::MODE_32).expect("failed to instantiate emulator");
    assert_eq!(emu.mem_map(0x1000, 0x4000, unicorn::PROT_ALL), Ok(()));
    assert_eq!(emu.mem_write(0x1000, &x86_code32), Ok(()));

    let hook = emu.add_mem_hook(unicorn::HookType::MEM_READ_UNMAPPED,
                      0,
                      std::u64::MAX,
                      callback)
        .expect("failed to add memory hook");
    assert_eq!(emu.reg_write(unicorn::RegisterX86::EAX, 0x123), Ok(()));
    assert_eq!(emu.emu_start(0x1000, 0x1001, 10 * unicorn::SECOND_SCALE, 1),
               Err((unicorn::Error::READ_UNMAPPED)));
    assert_eq!(emu.remove_hook(hook), Ok(()));
}

#[test]
fn emulate_arm() {
    let arm_code32: Vec<u8> = vec![0x83, 0xb0]; // sub    sp, #0xc

    let mut emu = CpuARM::new(unicorn::Mode::THUMB).expect("failed to instantiate emulator");
    assert_eq!(emu.reg_write(unicorn::RegisterARM::R1, 123), Ok(()));
    assert_eq!(emu.reg_read(unicorn::RegisterARM::R1), Ok((123)));

    // Attempt to write to memory before mapping it.
    assert_eq!(emu.mem_write(0x1000, &arm_code32),
               (Err(unicorn::Error::WRITE_UNMAPPED)));

    assert_eq!(emu.mem_map(0x1000, 0x4000, unicorn::PROT_ALL), Ok(()));
    assert_eq!(emu.mem_write(0x1000, &arm_code32), Ok(()));
    assert_eq!(emu.mem_read(0x1000, arm_code32.len()),
               Ok(arm_code32.clone()));

    assert_eq!(emu.reg_write(unicorn::RegisterARM::SP, 12), Ok(()));
    assert_eq!(emu.reg_write(unicorn::RegisterARM::R0, 10), Ok(()));

    assert_eq!(emu.emu_start(0x1000,
                             (0x1000 + arm_code32.len()) as u64,
                             10 * unicorn::SECOND_SCALE,
                             1000),
               Ok(()));
    assert_eq!(emu.reg_read(unicorn::RegisterARM::SP), Ok((0)));
    assert_eq!(emu.reg_read(unicorn::RegisterARM::R0), Ok((10)));
}

#[test]
fn emulate_mips() {
    let mips_code32 = vec![0x56, 0x34, 0x21, 0x34]; // ori $at, $at, 0x3456;

    let mut emu = CpuMIPS::new(unicorn::Mode::MODE_32).expect("failed to instantiate emulator");
    assert_eq!(emu.mem_map(0x1000, 0x4000, unicorn::PROT_ALL), Ok(()));
    assert_eq!(emu.mem_write(0x1000, &mips_code32), Ok(()));
    assert_eq!(emu.mem_read(0x1000, mips_code32.len()),
               Ok(mips_code32.clone()));
    assert_eq!(emu.reg_write(unicorn::RegisterMIPS::AT, 0), Ok(()));
    assert_eq!(emu.emu_start(0x1000,
                             (0x1000 + mips_code32.len()) as u64,
                             10 * unicorn::SECOND_SCALE,
                             1000),
               Ok(()));
    assert_eq!(emu.reg_read(unicorn::RegisterMIPS::AT), Ok((0x3456)));
}

#[test]
fn mem_unmapping() {
    let emu = CpuX86::new(unicorn::Mode::MODE_32).expect("failed to instantiate emulator");
    assert_eq!(emu.mem_map(0x1000, 0x4000, unicorn::PROT_ALL), Ok(()));
    assert_eq!(emu.mem_unmap(0x1000, 0x4000), Ok(()));
}
