#include "./device_tree.h"
#include "./assert.h"
#include "./debug_uart.h"
#include "./util.h"

struct sized_string next_segment(const char *path) {
  uint32_t remaining_length = strlen(path);

  struct sized_string s;
  s.data = path;
  for (uint32_t i = 0; i <= remaining_length; i++) {
    if (path[i] == '/') {
      s.length = i;
      return s;
    }
  }
  s.length = remaining_length;
  return s;
}

uint32_t* _parse_fdt_structure(BYTE* fdt_bytes, uint32_t depth, const char* strings){
  uint32_t* tag_window = (uint32_t*)fdt_bytes;

  while(1){
    uint32_t tag = be_to_native_u32(*tag_window);

    if(tag == 0x00000001){
      // We enter another scope

      for(int i = 0; i < depth; i++) uart_putc(' ');
      BYTE* node_data = (BYTE*)tag_window + 4;

      char* node_name = (char*)node_data;

      uint32_t name_len = strlen(node_name) + 1;

      uint32_t aligned_len = (name_len + 3) & ~3;

      node_data += aligned_len;
      tag_window = _parse_fdt_structure(node_data, depth + 1, strings);
      continue;
    }
    else if(tag == 0x00000002){
      // We leave this scope

      return tag_window + 1;
    }
    else if(tag == 0x00000003){
      // Define a property
      
      // struct fdt_prop* prop = (struct fdt_prop*)++tag_window;
      uint32_t num_bytes = be_to_native_u32(*++tag_window);
      uint32_t name_offset = be_to_native_u32(*++tag_window);

      char* prop_name = strings + name_offset; 
      for(int i = 0; i < depth; i++) uart_putc(' ');

      uint32_t aligned_len = (num_bytes + 3) & ~3;
      tag_window += aligned_len / 4;

    }
    else if(tag == 0x00000004){
      // no-op
    }
    else if(tag == 0x00000009){
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
struct fdt_lookup_result fdt_lookup(fdt_data_block data, const char *path,
                                    const char *stable, bool wildcard_postfix) {
  uint32_t remaining = strlen(path);
  struct sized_string search_segment = next_segment(path);


  bool is_final_segment = search_segment.length == remaining;

  uint32_t *tag_window = (uint32_t *)data;

  while (1) {
    uint32_t tag = be_to_native_u32(*tag_window);

    if (tag == 0x00000001) {
      // We enter another scope
      BYTE *node_data = (BYTE *)tag_window + 4;
      char *node_name = (char *)node_data;
      uint32_t name_len = strlen(node_name) + 1;

      uint32_t aligned_len = (name_len + 3) & ~3;

      node_data += aligned_len;
      tag_window = _parse_fdt_structure(node_data, 0, stable);
      struct sized_string sstr_node_name = cstr_to_sstr(node_name);

      uart_print("Parse node: ");
      uart_print(node_name);
      uart_print("\n");
      if (is_final_segment) {
        if (wildcard_postfix
                ? sstr_begins_with(&sstr_node_name, &search_segment)
                : sstr_equality(&sstr_node_name, &search_segment)) {
          struct fdt_lookup_result r;
          union fdt_lookup_value v;
          v.subtree = node_data;
          r.discrim = FDT_ITEM_SUBTREE;
          r.value = v;
          return r;
        } else {
          continue;
        }
      } else {

        if (sstr_equality(&sstr_node_name, &search_segment)) {
          uart_print("Recursing!\n");
          // Get the new name segment and recurse
          const char *next_lookup = path + search_segment.length + 1;
          uart_print(next_lookup);
          uart_print("\n");
          return fdt_lookup(node_data, next_lookup, stable, wildcard_postfix);
        } else {
          continue;
        }
      }

      continue;
    } else if (tag == 0x00000002) {
      uart_print("leave\n");
      // Come up short
      struct fdt_lookup_result r;
      r.discrim = FDT_ITEM_NOMATCH;
      tag_window++;
      return r;
    } else if (tag == 0x00000003) {
      // Define a property

      uint32_t num_bytes = be_to_native_u32(*++tag_window);
      uint32_t name_offset = be_to_native_u32(*++tag_window);
      tag_window += 1;
      const char *prop_name = stable + name_offset;
      uint32_t aligned_len = (num_bytes + 3) & ~3;
      BYTE* prop_value = (BYTE*)tag_window;
      tag_window += aligned_len / 4;
      struct sized_string sstr_prop_name = cstr_to_sstr(prop_name);
      if (is_final_segment) {

        if (wildcard_postfix
                ? sstr_begins_with(&sstr_prop_name, &search_segment)
                : sstr_equality(&sstr_prop_name, &search_segment)) {

          struct fdt_prop_value v;
          v.len = num_bytes;
          v.value = prop_value;

          struct fdt_lookup_result r;
          r.value.prop = v;
          r.discrim = FDT_ITEM_PROP_VALUE;

          return r;
        } else {
          continue;
        }

      } else {
        // There has been an issue with the path
        // its not the final segment but we cannot recurse
        struct fdt_lookup_result r;
        r.discrim = FDT_ITEM_NOMATCH;
        return r;
      }

    } else if (tag == 0x00000004) {
      // no-op
      tag_window++;
      continue;
    } else if (tag == 0x00000009) {
      // Come up short
      struct fdt_lookup_result r;
      r.discrim = FDT_ITEM_NOMATCH;
      return r;
    } else {
      uart_print("UNKNOWN TAG: ");
      uart_put_reg_hex(tag);
      uart_print("\n");
      ASSERT(0);
    }

  }

  // No luck
  struct fdt_lookup_result r;
  r.discrim = FDT_ITEM_NOMATCH;
  return r;
}
