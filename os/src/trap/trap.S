.altmacro
.macro SAVE_GP n
	sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
	ld x\n, \n*8(sp)
.endm
	.section .text
	.global __alltraps
	.global __restore
	.align 2
__alltraps:
	csrrw sp, sscratch, sp
	addi sp, sp, -34*8
	
	# skip x0,tp(x4), x2(sp) deny save
	sd x1, 1*8(sp)
	sd x3, 3*8(sp)
	.set n, 5
	.rept 27
		SAVE_GP %n
		.set n, n+1
	.endr
	
	# use t0/t1/t2 do some things for CSRs:sstatus,sepc
	csrr t0, sstatus
	csrr t1, sepc
	sd t0, 32*8(sp)
	sd t1, 33*8(sp)
	
	# save user stack ptr
	csrr t2, sscratch
	sd t2, 2*8(sp)
	
	# set input args of trap_handler(cx: &mut TrapContext)
	mv a0, sp
	call trap_handler

__restore:
	
	# case1: trap_handler return
	# case2: run_next_app():first run app/continue run next app
	
	mv sp, a0;
	# first restore CSRs, because next we will restore t0/t1/t2
	ld t0, 32*8(sp)
	ld t1, 33*8(sp)
	ld t2, 2*8(sp)
	csrw sstatus, t0
	csrw sepc, t1
	csrw sscratch, t2
	
	# restore general registers except x0/tp
	ld x1, 1*8(sp)
	ld x3, 3*8(sp)
	.set n, 5
	.rept 27
		LOAD_GP %n
		.set n, n+1
	.endr
	
	# release TrapCotext on kernel stack
	addi sp, sp, 34*8
	
	# switch stack ptr
	csrrw sp, sscratch, sp
	sret
