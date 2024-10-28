#ifndef CHOPIN_DEVICE_TREE_H
#define CHOPIN_DEVICE_TREE_H

#include "./types.h"

struct __attribute__((packed)) fdt_header {
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

struct FlattenedDeviceTree{
  struct fdt_header header;
};

struct fdt_prop{
  uint32_t len;
  uint32_t name_offset;
};

#endif
