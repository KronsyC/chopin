CROSS_COMPILE = riscv64-linux-gnu-
CC = $(CROSS_COMPILE)gcc
AS = $(CROSS_COMPILE)as
LD = $(CROSS_COMPILE)ld
OBJCOPY = $(CROSS_COMPILE)objcopy

CFLAGS = -march=rv64g -mabi=lp64 -static -mcmodel=medany \
         -fvisibility=hidden -nostdlib -nostartfiles -g -O0
QEMU = qemu-system-riscv64

all: kernel.elf

boot.o: src/boot.S
	$(CC) $(CFLAGS) -c src/boot.S -o boot.o

kernel.o: src/kernel.c
	$(CC) $(CFLAGS) -c src/kernel.c -o kernel.o

userland.o: src/userland.c 
	$(CC) $(CFLAGS) -c src/userland.c -o userland.o

trap.o: src/trap.c
	$(CC) $(CFLAGS) -c src/trap.c -o trap.o

debug_uart.o: src/debug_uart.c
	$(CC) $(CFLAGS) -c src/debug_uart.c -o debug_uart.o

device_tree.o: src/device_tree.c 
	$(CC) $(CFLAGS) -c src/device_tree.c -o device_tree.o

util.o: src/util.c 
	$(CC) $(CFLAGS) -c src/util.c -o util.o

kernel.elf: boot.o userland.o trap.o debug_uart.o kernel.o device_tree.o util.o linker.ld
	$(LD) -T linker.ld boot.o userland.o trap.o debug_uart.o device_tree.o util.o kernel.o -o kernel.elf



run: kernel.elf
	$(QEMU) -machine sifive_u,firmware=/usr/share/opensbi/lp64/generic/firmware/fw_dynamic.bin -cpu rv64 -smp 4 -m 512M \
		-nographic -kernel kernel.elf \
		 
clean:
	rm -f *.o kernel.elf



.PHONY: clean run
