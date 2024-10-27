///
/// Process Context
///
/// This is the submodule responsible
/// for various types relating to process 
/// context
///
///
/// used for trap handling and 
///
///

#include "./types.h"


#ifndef CHOPIN_PROC_CONTEXT_H
#define CHOPIN_PROC_CONTEXT_H


struct RegisterContext{
  REG return_address;
  REG global_pointer;
  REG thread_pointer;
  REG temp0;
  REG temp1;
  REG temp2;
  REG saved0;
  REG saved1;
  REG arg0;
  REG arg1;
  REG arg2;
  REG arg3;
  REG arg4;
  REG arg5;
  REG arg6;
  REG arg7;
  REG saved2;
  REG saved3;
  REG saved4;
  REG saved5;
  REG saved6;
  REG saved7;
  REG saved8;
  REG saved9;
  REG saved10;
  REG saved11;
  REG temp3;
  REG temp4;
  REG temp5;
  REG temp6;
  REG stack_ptr;
  REG program_return_addr;
};



#endif
