OUTPUT_ARCH("riscv")
ENTRY(_start)

/* SiFive U memory layout */
MEMORY
{
    RAM   (wxa!ri) : ORIGIN = 0x80200000, LENGTH = 128M    /* Start with smaller RAM for testing */
}

SECTIONS
{
    /* Code starts at 0x80000000 */
    . = ORIGIN(RAM);

    .text : {
        *(.text.init)    /* Startup code first */
        *(.text.trap)    /* Then trap handlers */
        *(.text)         /* Regular code */
        *(.text.*)       /* Other text sections */
        . = ALIGN(8);
    } >RAM

    .rodata : {
        *(.rodata)
        *(.rodata.*)
        *(.srodata.*)
        . = ALIGN(8);
    } >RAM

    .data : {
        *(.data)
        *(.data.*)
        *(.sdata)
        *(.sdata.*)
        . = ALIGN(8);
    } >RAM

    .bss : {
        *(.bss)
        *(.bss.*)
        *(.sbss)
        *(.sbss.*)
        *(COMMON)
        . = ALIGN(8);
    } >RAM

    /* Stack setup */
    .stack (NOLOAD) : {
        . = ALIGN(16);
        . += 0x4000;     /* 16KB stack */
        stack_top = .;
    } >RAM
}
