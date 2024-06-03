

all: build

build:
	avr-as emitted.asm -o emitted.o
	avr-ld emitted.o -o emitted
	rm *.o

host:
	simavr -m atmega328p -f 16000000 -g emitted

gdb:
	avr-gdb emitted -x setup.gdb
