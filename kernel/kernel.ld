OUTPUT_ARCH("riscv")
ENTRY(_start)
. = 0x80200000; /* Hint where to place the code to QEMU, the kernel is in practice Position Independant */

SECTIONS
{
    
    .text : {
        *(.text.init)    /* Startup code first */
        *(.text.trap)    /* Then trap handlers */
        *(.text)         /* Regular code */
        *(.text.*)       /* Other text sections */
        . = ALIGN(8);
    }

    .rodata : {
        *(.rodata)
        *(.rodata.*)
        *(.srodata.*)
        . = ALIGN(8);
    }

    .data : {
        *(.data)
        *(.data.*)
        *(.sdata)
        *(.sdata.*)
        . = ALIGN(8);
    }

    .bss : {
        *(.bss)
        *(.bss.*)
        *(.sbss)
        *(.sbss.*)
        *(COMMON)
        . = ALIGN(8);
    }

    /* Stack setup */
    .stack (NOLOAD) : {
        . = ALIGN(16);
        . += 0x4000;     /* 16KB stack */
        stack_top = .;
    }

    .kernel_end : {
        . = ALIGN(8);
        PROVIDE(CHOPIN_kernel_memory_end = .);
    }

}
