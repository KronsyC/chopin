#ifndef CHOPIN_DEVICE_TREE_H
#define CHOPIN_DEVICE_TREE_H

#include "./types.h"

struct fdt_header {
    uint32_t magic;
    uint32_t totalsize;
    uint32_t off_dt_struct;
    uint32_t off_dt_strings;
    uint32_t off_mem_rsvmap;
    uint32_t version;
    uint32_t last_comp_version;
    uint32_t boot_cpuid_phys;
    uint32_t size_dt_strings;
    uint32_t size_dt_struct;
};


/**
 * These entries represent reserved
 * memory blocks
 */
struct fdt_reserve_entry{
  uint64_t address;
  uint64_t size;
};

typedef const char* fdt_strings_block;

typedef BYTE* fdt_data_block;





struct FlattenedDeviceTree{
  struct fdt_header header;
};

struct fdt_prop{
  uint32_t len;
  uint32_t name_offset;
};


enum fdt_item{
  FDT_ITEM_PROP_VALUE,
  FDT_ITEM_SUBTREE,
  FDT_ITEM_NOMATCH
};

struct fdt_prop_value{
  uint32_t len;
  BYTE* value;
};


union fdt_lookup_value{
  struct fdt_prop_value prop;
  BYTE* subtree;
};

struct fdt_lookup_result{
  union fdt_lookup_value value;
  enum fdt_item discrim;
};

/**
 * Perform a lookup for a piece of data in an FDT 
 *
 * This can be either a property value or a nested 
 * FDT object
 *
 * The path is to be delimeted with forward slashes
 *
 * All lookups should start with a slash due to the technicality 
 * of the first block having a 0-length name
 *
 * If allow_any_postfix is specified, it will match the first 
 * value that matches the regex: ${path}.*
 */
struct fdt_lookup_result fdt_lookup(fdt_data_block data, const char* path, const char* string_table, bool allow_any_postfix);

#endif
