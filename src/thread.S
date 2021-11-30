.intel_syntax noprefix
# context_switch(current: TCB, next: TCB)
.global context_switch
context_switch:
	push rbx
	push rbp
	push r12
	push r13
	push r14
	push r15

	mov rax, cr2
	push rax
	pushfq

	mov [rdi], rsp
	mov rsp, [rsi]
	cli
	popfq
	pop rax
	mov cr2, rax

	pop r15
	pop r14
	pop r13
	pop r12
	pop rbp
	pop rbx

	ret