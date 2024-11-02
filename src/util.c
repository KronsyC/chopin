#include "./util.h"

uint32_t strlen(const char* str){
  uint32_t len = 0;
  while(*str++ != 0) len++;
  return len;
}

struct sized_string cstr_to_sstr(const char* cstr){
  uint32_t len = strlen(cstr);

  struct sized_string s;
  s.length = len;
  s.data = cstr;
  return s;
}

uint32_t be_to_native_u32(uint32_t value){
  // For now, big to little endian
  
  return 
    value >> 24
    | (value >> 8) & 0x0000FF00
    | (value << 8) & 0x00FF0000
    | (value << 24);
}

bool sstr_equality(struct sized_string* s1, struct sized_string* s2){
  if(s1->length != s2->length) return false;

  for(uint32_t i = 0; i < s1->length; ++i){
    if(s1->data[i] != s2->data[i]) return false;
  }

  return true;
}

bool sstr_begins_with(struct sized_string* subject, struct sized_string* substr){
  if(substr->length > subject->length) return false;

  for(uint32_t i = 0; i < substr->length; i++){
    if(substr->data[i] != subject->data[i]) return false;
  }

  return true;
}
