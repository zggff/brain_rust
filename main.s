.include "macros.s"
.include "print.s"
.include "globals.s"


.text
	.align 2
	.global _start

	_start:
		print_str "12345678\n"
		
		syscall1 SYS_return, #0
.data
	value:		.quad 0x123456789AB00000

.bss
	buffer_size = 20
	.lcomm buffer, 20
