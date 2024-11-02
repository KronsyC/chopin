#ifndef CHOPIN_UTIL_H
#define CHOPIN_UTIL_H


#include "./types.h"

uint32_t strlen(const char* str);


struct sized_string{
  const char* data;
  uint32_t length;
};


struct sized_string cstr_to_sstr(const char* cstr);

bool sstr_equality(struct sized_string* s1, struct sized_string* s2);
bool sstr_begins_with(struct sized_string* subject, struct sized_string* substr);

uint32_t be_to_native_u32(uint32_t value);
#endif 
