#include "./types.h"
#include "./process_context.h"

#ifndef CHOPIN_TRAP_H
#define CHOPIN_TRAP_H


/**
 * Return control to the userspace after handling 
 * a trap
 */
void CHOPIN_kern_trap_return();



void CHOPIN_kern_trap_handle_unimplemented(REG trap_code, struct RegisterContext* ctx);
void CHOPIN_kern_trap_handle_ecall(REG trap_code, struct RegisterContext* ctx);
void CHOPIN_kern_trap_handle_instruction_access_fault(REG trap_code, struct RegisterContext* ctx);
void CHOPIN_kern_trap_handle_load_access_fault(REG trap_code, struct RegisterContext* ctx);
void CHOPIN_kern_trap_handle_illegal_instruction(REG trap_code, struct RegisterContext* ctx);




#endif
