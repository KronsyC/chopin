#![no_std]



use core::panic::PanicInfo;




#[panic_handler]
pub fn chopin_panic_handle(info : &PanicInfo) -> !{

    let prefix = "CHOPIN PANIC :: ";

    for c in prefix.chars(){
        sbi::legacy::console_putchar(c as u8);
    }

    if let Some(message) = info.message().as_str(){
        for c in message.chars(){
            sbi::legacy::console_putchar(c as u8);
        }
    }

    sbi::legacy::console_putchar(b'\n');
   
    loop {

    }
}


