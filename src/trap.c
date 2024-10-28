#include "./trap.h"
#include "./debug_uart.h"
void CHOPIN_kern_trap_handle_instruction_access_fault(REG code, struct RegisterContext* registers){

  REG offending_instruction_addr = registers->program_return_addr;

  uart_print("Instruction Access Fault Occurred at: 0x");
  uart_put_reg_hex(offending_instruction_addr);

  uart_print("\n");

  while(1){}
}

void CHOPIN_kern_trap_handle_load_access_fault(REG trap_code, struct RegisterContext* ctx){
  REG stval;
  __asm__ volatile("csrr %0, stval" : "=r" (stval));

  uart_print("Failed to access memory address: ");
  uart_put_reg_hex(stval);
  uart_print("\n");
 
  ctx->program_return_addr += 4;

  while(1){}
}
void CHOPIN_kern_trap_handle_ecall(REG code, struct RegisterContext* ctx){
  // ECall handling code
  uart_print("Ecall has been received:\n");
  uart_print("Call ID: ");
  uart_put_reg_hex(ctx->arg0);

  ctx->program_return_addr += 4;

  CHOPIN_kern_trap_return();
  
}

void CHOPIN_kern_trap_handle_illegal_instruction(REG code, struct RegisterContext* registers){
  uart_print("An illegal instruction was attempted to be executed, halting...\n");
  

  while(1){}
}

void CHOPIN_kern_trap_handle_unimplemented(REG code, struct RegisterContext* registers){
  char code_char = '0';
  code_char += code;

  uart_putc(code_char);
  uart_print(" >> Unimplemented Trap Handler\n");
}
