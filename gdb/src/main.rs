use gdb::DebugServer;
use mips_emulator::{
    cpu::{DefaultExternalHandler, MipsCpu},
    memory::page_pool::MemoryDefaultAccess,
};

fn main() {
    let mut cpu =
        MipsCpu::<DefaultExternalHandler>::new_interface(DefaultExternalHandler::default());
    cpu.cpu_mut(|cpu| unsafe {
        cpu.raw_mem().set_u32_alligned_be(0, 0xFFBBCCDD);
    });
    let debugger = DebugServer::new(cpu);
    let _ = debugger.start_debug_server();
}
