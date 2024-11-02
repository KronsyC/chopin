#include "./assert.h"
#include "./debug_uart.h"
#include "./device_tree.h"
#include "./types.h"
#include "./util.h"
#include <stdatomic.h>

uint32_t *parse_fdt_structure(BYTE *fdt_bytes, uint32_t depth,
                              const char *strings) {
  uint32_t *tag_window = (uint32_t *)fdt_bytes;

  while (1) {
    uint32_t tag = be_to_native_u32(*tag_window);

    if (tag == 0x00000001) {
      // We enter another scope

      for (int i = 0; i < depth; i++)
        uart_putc(' ');
      uart_print("Parse node: ");
      BYTE *node_data = (BYTE *)tag_window + 4;

      char *node_name = (char *)node_data;
      uart_print(node_name);
      uart_print("\n");

      uint32_t name_len = strlen(node_name) + 1;

      uint32_t aligned_len = (name_len + 3) & ~3;

      node_data += aligned_len;
      tag_window = parse_fdt_structure(node_data, depth + 1, strings);
      continue;
    } else if (tag == 0x00000002) {
      // We leave this scope
      if (depth == 0) {
        ASSERT(0);
      }

      return tag_window + 1;
    } else if (tag == 0x00000003) {
      // Define a property

      // struct fdt_prop* prop = (struct fdt_prop*)++tag_window;
      uint32_t num_bytes = be_to_native_u32(*++tag_window);
      uint32_t name_offset = be_to_native_u32(*++tag_window);

      char *prop_name = strings + name_offset;
      for (int i = 0; i < depth; i++)
        uart_putc(' ');
      uart_print("Property: ");
      uart_print(prop_name);
      uart_print("\n");

      uint32_t aligned_len = (num_bytes + 3) & ~3;
      tag_window += aligned_len / 4;

    } else if (tag == 0x00000004) {
      // no-op
    } else if (tag == 0x00000009) {
      if (depth == 0) {
        return 0x00;
      } else {

        // Exiting before closing scope
        ASSERT(0);
      }
    } else {
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
  fdt_header->last_comp_version =
      be_to_native_u32(fdt_header->last_comp_version);
  fdt_header->size_dt_struct = be_to_native_u32(fdt_header->size_dt_struct);
  ASSERT(fdt_header->magic == 0xD00DFEED);

  REG max_fdt_header_addr = (REG)((BYTE *)fdt + fdt_header->totalsize);

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


  struct fdt_lookup_result address_cells_lookup = fdt_lookup(fdt_structure_block, "/#address-cells", strings_block, false);
  struct fdt_lookup_result size_cells_lookup = fdt_lookup(fdt_structure_block, "/#size-cells", strings_block, false);


  ASSERT(address_cells_lookup.discrim == FDT_ITEM_PROP_VALUE);
  ASSERT(size_cells_lookup.discrim == FDT_ITEM_PROP_VALUE);

  uint32_t address_cells = be_to_native_u32(*(uint32_t*)address_cells_lookup.value.prop.value);
  uint32_t size_cells = be_to_native_u32(*(uint32_t*)size_cells_lookup.value.prop.value);

  ASSERT(address_cells == size_cells); // Simplification


  if(address_cells != 2){
    uart_print("Chopin only works with 64-bit memory\n");
    ASSERT(0);
  }


  struct fdt_lookup_result memory =
      fdt_lookup(fdt_structure_block, "/memory", strings_block, true);

  ASSERT(memory.discrim == FDT_ITEM_SUBTREE)

  struct fdt_lookup_result mem_device_type =
      fdt_lookup(memory.value.subtree, "device_type", strings_block, false);
  struct fdt_lookup_result mem_reg =
      fdt_lookup(memory.value.subtree, "reg", strings_block, false);

  ASSERT(mem_device_type.discrim == FDT_ITEM_PROP_VALUE);
  ASSERT(mem_reg.discrim == FDT_ITEM_PROP_VALUE);

  const char *device_type_str = mem_device_type.value.prop.value;

  uart_print("Device type: ");
  uart_print(device_type_str);
  uart_print("\n");

  uart_print("Reg len: ");
  uart_put_reg_hex(mem_reg.value.prop.len);
}
