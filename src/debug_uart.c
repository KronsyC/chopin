#include "./types.h"
#include "./debug_uart.h"
void uart_put_reg_hex(REG r){
  for(int i = 7; i >= 0; i--){
    REG mask = 0xFF << i * 8;
    REG masked = r & mask;
    REG shifted = masked >> (i * 8);
    BYTE value = shifted;

    BYTE msn = value >> 4;
    BYTE lsn = value & 0x0F;

    char lookup[] = {'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'};

    char hex_msn = lookup[msn];
    char hex_lsn = lookup[lsn];

    uart_putc(hex_msn);
    uart_putc(hex_lsn);
  }
}
