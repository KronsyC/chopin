
#include "./process_context.h"
#include "./trap.h"
#include "./debug_uart.h"
struct FDT{

};




enum SystemCalls{
  SYS_EXIT = 0x42,
};



void userland(REG hart_id, REG devicetree){


  uart_print("Hello from userland C code!\n");

  uart_print("Running on HART: 0x");
  uart_put_reg_hex(hart_id);
  uart_print("\n");

  uart_print("Device tree is at: ");
  uart_put_reg_hex((REG)devicetree);
  uart_print("\n");


  char* some_address = (char*)0xDEADBEEFDEADBEEF;

  char c = *some_address;

  uart_print("Loaded Address\n");

  register int id asm("a0") = SYS_EXIT;
  asm volatile ("ecall");
}


