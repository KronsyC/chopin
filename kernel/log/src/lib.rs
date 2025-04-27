#![no_std]

extern crate alloc;

pub static KERNEL_LOGGER : ChopinLogger = ChopinLogger;


pub fn initialize_logger(){
    log::set_logger(&KERNEL_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
}




pub struct ChopinLogger;


impl log::Log for ChopinLogger{
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        let message = alloc::format!("{} ({}) :: {}", record.level(), record.target(), record.args());

        for c in message.chars(){
            sbi::legacy::console_putchar(c as u8);
        }
        sbi::legacy::console_putchar(b'\n');
    }

   fn flush(&self) {

    }
}
