#include "./assert.h"
#include "./debug_uart.h"
#include "./device_tree.h"
#include "./types.h"


uint32_t strlen(const char* str){
  uint32_t len = 0;
  while(*str++ != 0) len++;
  return len;
}

uint32_t be_to_native_u32(uint32_t value){
  // For now, big to little endian
  
  return 
    value >> 24
    | (value >> 8) & 0x0000FF00
    | (value << 8) & 0x00FF0000
    | (value << 24);
}

uint32_t* parse_fdt_structure(BYTE* fdt_bytes, uint32_t depth, const char* strings){
  uint32_t* tag_window = (uint32_t*)fdt_bytes;

  while(1){
    uint32_t tag = be_to_native_u32(*tag_window);

    if(tag == 0x00000001){
      // We enter another scope

      uart_print("Parse node: ");
      BYTE* node_data = (BYTE*)tag_window + 4;

      char* node_name = (char*)node_data;
      uart_print(node_name);
      uart_print("\n");

      uint32_t name_len = strlen(node_name) + 1;

      uint32_t aligned_len = (name_len + 3) & ~3;

      node_data += aligned_len;
      tag_window = parse_fdt_structure(node_data, depth + 1, strings);
      continue;
    }
    else if(tag == 0x00000002){
      uart_print("Exit node: ");
      uart_put_reg_hex(depth);
      uart_print("\n");
      // We leave this scope
      if(depth == 0){
        ASSERT(0);
      }

      return tag_window + 1;
    }
    else if(tag == 0x00000003){
      // Define a property
      
      // struct fdt_prop* prop = (struct fdt_prop*)++tag_window;
      uint32_t num_bytes = be_to_native_u32(*++tag_window);
      uint32_t name_offset = be_to_native_u32(*++tag_window);

      char* prop_name = strings + name_offset; 
      uart_print("Property: ");
      uart_print(prop_name);
      uart_print("\n");

      uint32_t aligned_len = (num_bytes + 3) & ~3;
      tag_window += aligned_len / 4;

    }
    else if(tag == 0x00000004){
      // no-op
      uart_print("no-op\n");
    }
    else if(tag == 0x00000009){
      uart_print("Finished\n");
      if(depth == 0){
        return 0x00;
      }
      else{
        
        // Exiting before closing scope
        ASSERT(0);
      }
    }
    else{
      // uart_print("UNKNOWN TAG: ");
      // uart_put_reg_hex(tag);
      // uart_print("\n");
    }

    tag_window++;


  }
}

void CHOPIN_kern_start(REG hart_id, struct FlattenedDeviceTree *fdt) {
  uart_print("Chopin Kernel!\n");

  struct fdt_header *fdt_header = (struct fdt_header *)fdt;

  fdt_header->magic = be_to_native_u32(fdt_header->magic);
  fdt_header->totalsize = be_to_native_u32(fdt_header->totalsize);
  fdt_header->off_dt_struct = be_to_native_u32(fdt_header->off_dt_struct);
  fdt_header->off_dt_strings = be_to_native_u32(fdt_header->off_dt_strings);
  fdt_header->off_mem_rsvmap = be_to_native_u32(fdt_header->off_mem_rsvmap);
  fdt_header->version = be_to_native_u32(fdt_header->version);
  fdt_header->last_comp_version = be_to_native_u32(fdt_header->last_comp_version);
  fdt_header->size_dt_struct = be_to_native_u32(fdt_header->size_dt_struct);
  ASSERT(fdt_header->magic == 0xD00DFEED);

  REG max_fdt_header_addr = (REG)((BYTE*)fdt + fdt_header->totalsize);

  uart_print("Max FDT Header Addr: 0x");
  uart_put_reg_hex(max_fdt_header_addr);
  uart_print("\n");

  // 1. Parse Memory Reservation Map
  // It goes from

  fdt_strings_block strings_block =
      (fdt_strings_block)fdt + fdt_header->off_dt_strings;
  // uint32_t strings_block_max_offset = fdt_header->size_dt_strings;

  BYTE *fdt_structure_block = (BYTE *)fdt + fdt_header->off_dt_struct;
  uint32_t fdt_structure_len = fdt_header->size_dt_struct;


  parse_fdt_structure(fdt_structure_block, 0, strings_block);

  uart_print("Parsed FDT\n");
}
