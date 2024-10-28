#ifndef CHOPIN_ASSERT_H
#define CHOPIN_ASSERT_H

#include "./debug_uart.h"



#define ASSERT(cond) \
  if(!(cond)){ \
    uart_print("Assertion failed at "); \
    uart_print(__FILE__); \
    uart_print(":"); \
    uart_put_reg_hex(__LINE__); \
    uart_print("\n"); \
    while(1) {} \
  }



#endif
