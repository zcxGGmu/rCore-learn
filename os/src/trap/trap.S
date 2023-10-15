.altmacro
.macro SAVE_GP n
	sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
	ld x\n, \n*8(sp)
.endm
	.section .text.trampoline
	.global __alltraps
	.global __restore
	.align 2
__alltraps:
    # now sp->user_stack, sscratch->*TrapContext
	csrrw sp, sscratch, sp

    # skip x0,tp(x4), x2(sp) deny save
	# save other general registers
    sd x1, 1*8(sp)
	sd x3, 3*8(sp)
	.set n, 5
	.rept 27
		SAVE_GP %n
		.set n, n+1
	.endr

    # save CSR:sstatus/sepc
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
	csrr t0, sstatus
	csrr t1, sepc
	sd t0, 32*8(sp)
	sd t1, 33*8(sp)
	
	# read user_sp from sscratch and save it in TrapContext
	csrr t2, sscratch
	sd t2, 2*8(sp)

    # Previously, we used sp to save the context \
    # in the memory corresponding to the TrapContext
    
    # Load kernel_satp/trap_handler/kernel_sp into t0/t1/sp
    ld t0, 34*8(sp)
    ld t1, 36*8(sp)
    ld sp, 35*8(sp)
    
    # switch to kernel_space
    csrw satp, t0
    sfence.vma
    
    # jump to trap_handler
    jr t1

__restore:	
	# case1: trap_handler return to user
	# case2: run_next_task():first run task/continue run next task

    # __restore have two input args:
    # a0 => *TrapContext in user space(Constant)
    # a1 => user space token

    # switch to user space
    /*
      We must switch to the application address space at the beginning 
      of `__restore` so that the subsequent CPU can restore register 
      translation through the correct "table walk".
    */
    csrw satp, a1
    sfence.vma
    
    # sscratch need to save *TrapContext before back to user
    csrw sscratch, a0

    # now sp points to TrapContext in user space, start restoring based on it
    mv sp, a0
    
    # restore CSR:sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    
    # restore general registers x1/x3/x5~x31
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n + 1
    .endr

    # restore user_sp to sp register
    ld sp, 2*8(sp)
    
    # back to user
    sret
