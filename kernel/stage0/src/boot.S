.section .text.init
.global _start

/* SiFive U peripheral addresses */
.equ UART0,       0x10010000
.equ UART_TXDATA, 0x00
.equ UART_RXDATA, 0x04
.equ UART_DIV,    0x18
.equ UART_TXCTRL, 0x08
.equ UART_RXCTRL, 0x0C

# .extern userland
# .extern uart_put_reg_hex
.extern CHOPIN_kern_trap_handle_unimplemented
.extern CHOPIN_kern_trap_handle_instruction_access_fault
.extern CHOPIN_kern_trap_handle_ecall
.extern CHOPIN_kern_trap_handle_load_access_fault
.extern CHOPIN_kern_trap_handle_illegal_instruction
.extern CHOPIN_kern_stage0



_start:

    ################################################
    ## EXECUTION ENVIRONMENT BARE MINIMUM
    ################################################

    # Load stack address into SP register 
    
    la      sp, stack_top


    

    ################################################
    ## DEBUG ENVIRONMENT
    ################################################

    # Initialize UART
    li      t0, UART0
    
    # Set baud rate divider
    li      t1, 16         # 115200 baud at 16MHz
    sw      t1, UART_DIV(t0)
    
    # Enable transmitter
    li      t1, 1          # Enable transmit
    sw      t1, UART_TXCTRL(t0)

    

    ############################################
    ## SETUP TRAP HANDLER FOR USERSPACE CODE 
    ############################################

    call setup_trap_vector 


    #########################################
    ## JUMP INTO KERNEL CODE
    #########################################

    call CHOPIN_kern_stage0

after_userland:
    # Loop forever
1:  wfi
    j       1b



# This is the prelude code run on the boot of every HART 
# Once basis init is complete 
.global CHOPIN_kern_stage0_kcore_init
CHOPIN_kern_stage0_kcore_init:
  # a1 contains boot flags
  # 0 => Initialize Stack (t/f)


  andi t0, a0, 1
  

setup_trap_vector:

  # Initialize supervisor mode trap vector handling 

  la t0, trap_handler # Load trap handler 

  csrw stvec, t0 # write to the stvec csr so that the trap_handler is invoked directly by all traps 

  ret

/* Trap handlers in their own section */
.section .text.trap
.align 4
trap_handler:
  csrw sscratch, sp 

  la sp, irq_stack_top
  addi sp, sp, -256

trap_reg_save:
  # Preserve registers in the IRQ stack 
  sd ra, 0(sp)
  sd gp, 8(sp)
  sd tp, 16(sp)
  sd t0, 24(sp)
  sd t1, 32(sp)
  sd t2, 40(sp)
  sd s0, 48(sp)
  sd s1, 56(sp)
  sd a0, 64(sp)
  sd a1, 72(sp)
  sd a2, 80(sp)
  sd a3, 88(sp)
  sd a4, 96(sp)
  sd a5, 104(sp)
  sd a6, 112(sp)
  sd a7, 120(sp)
  sd s2, 128(sp)
  sd s3, 136(sp)
  sd s4, 144(sp)
  sd s5, 152(sp)
  sd s6, 160(sp)
  sd s7, 168(sp)
  sd s8, 176(sp)
  sd s9, 184(sp)
  sd s10,192(sp)
  sd s11,200(sp)
  sd t3, 208(sp)
  sd t4, 216(sp)
  sd t5, 224(sp)
  sd t6, 232(sp)

  csrr t0, sscratch 

  sd t0, 240(sp)

  csrr t0, sepc

  sd t0, 248(sp)
  # We have now preserved all registers 
  # Branch into interrupt handling logic 
  csrr a0, scause
  add  a1, zero, sp
  # Calculate an address into the dispatch table 

  la t1, trap_dispatch_table
  slli t0, a0, 3 # Multiply by 4

  add t1, t1, t0
  
  ld t0, 0(t1) # Load the handler address into t0


  jalr ra, t0, 0 # Call the handler, providing the trap number
1:
  j       1b


.global CHOPIN_kern_trap_return
CHOPIN_kern_trap_return:

  # Return from a trap handler,
  # a0 => The address to return to
  la sp, irq_stack_top
  addi sp, sp, -256

  # Set the return address
  ld t0, 248(sp)

  csrw sepc, t0
 
  # Restore registers
  ld ra, 0(sp)
  ld gp, 8(sp)
  ld tp, 16(sp)
  ld t0, 24(sp)
  ld t1, 32(sp)
  ld t2, 40(sp)
  ld s0, 48(sp)
  ld s1, 56(sp)
  ld a0, 64(sp)
  ld a1, 72(sp)
  ld a2, 80(sp)
  ld a3, 88(sp)
  ld a4, 96(sp)
  ld a5, 104(sp)
  ld a6, 112(sp)
  ld a7, 120(sp)
  ld s2, 128(sp)
  ld s3, 136(sp)
  ld s4, 144(sp)
  ld s5, 152(sp)
  ld s6, 160(sp)
  ld s7, 168(sp)
  ld s8, 176(sp)
  ld s9, 184(sp)
  ld s10,192(sp)
  ld s11,200(sp)
  ld t3, 208(sp)
  ld t4, 216(sp)
  ld t5, 224(sp)
  ld t6, 232(sp)
  csrr sp, sscratch


  sret
  
  


.align 8
trap_dispatch_table:
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_instruction_access_fault
  .dword CHOPIN_kern_trap_handle_illegal_instruction 
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_load_access_fault
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_ecall 
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_unimplemented 
  .dword CHOPIN_kern_trap_handle_unimplemented 

/* Regular code */
.section .text

/* Read-only data */
.section .rodata
hello_msg:
    .string "Hello from SiFive U!\n"
default_trap_msg:
    .string "Default trap handler called\n"
user_msg:
    .string "Hello from userland!\n"
/* Read-write data */
.section .data
some_data:
    .word   0x12345678

/* Zero-initialized data */
.section .bss
irq_stack:
  .space 512
irq_stack_top:
