#include "./types.h"


#ifndef CHOPIN_DEBUG_UART_H
#define CHOPIN_DEBUG_UART_H



void uart_print(char* d);
void uart_putc(char c);


void uart_put_reg_hex(REG r);

#endif
