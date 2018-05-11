extern crate core;

#[repr(C)]
#[derive(Debug)]
pub struct ThreadState {
    rsp: *mut usize,
    rdi: *mut usize,
    rsi: *mut usize,
    rdx: *mut usize,
    rcx: *mut usize,
    rip: *mut usize,
}

#[repr(C)]
pub struct Thread {
    state_ptr: *mut ThreadState,
    id: usize,
    stack: [u8;4*1024],
}

struct FnPtr {
    pub f: fn(&mut Thread, &mut Thread, usize),
}


impl core::fmt::Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        let rip = if self.state_ptr as usize == 0 { 0 } else { self.state().rip as usize};
        write!(f, "[Id:{:x} &ctxt:{:x} rip:{:x}]", self.id, self.state_ptr as usize, rip)
    }
}

impl Thread {
    pub fn new(id: usize) -> Thread {
        Thread {
            state_ptr: core::ptr::null_mut(),
            id,
            stack: [0x0u8; 4*1024],
        }
    }

    fn state_mut(&mut self) -> &mut ThreadState {
        unsafe {
            &mut *self.state_ptr
        }
    }

    fn state(&self) -> &ThreadState {
        unsafe {
            &*self.state_ptr
        }
    }

    pub fn prepare(&mut self, prev_thread: &mut Thread, f: fn(&mut Thread, &mut Thread, usize), arg: usize) {
        unsafe {
            let stack_needed = core::mem::size_of::<ThreadState>() as usize;
            let stack_start_offset = (self.stack.len() as usize) - stack_needed;
            self.state_ptr = (&mut self.stack[0] as *mut u8).offset(stack_start_offset as isize) as *mut ThreadState;
            self.state_mut().rsp = core::ptr::null_mut();
            self.state_mut().rdi = f as *mut usize;
            self.state_mut().rsi = prev_thread as *mut Thread as *mut usize;
            self.state_mut().rdx = self as *mut Thread as *mut usize;
            self.state_mut().rcx = arg as *mut usize;
            self.state_mut().rip = Thread::thread_start as *mut usize;
        }
        // kprintln!(CONTEXT, "Prepare: {:?}", self);
    }

    unsafe extern "C" fn thread_start(f_ptr: FnPtr, prev_thread: &mut Thread, this_thread: &mut Thread, arg: usize) {
        // let ip = f_ptr.f as *const u8;
        // kprintln!(CONTEXT, "thread_start f:{:?} arg:{:x} prev:{:?} current:{:?}", ip, arg, prev_thread, this_thread);
        (f_ptr.f)(prev_thread, this_thread, arg);
        panic!("Thread over!");
    }

    // Note: the calling convention seems to be ignored for x64
    // and is always https://en.wikipedia.org/wiki/X86_calling_conventions#System_V_AMD64_ABI
    // RDI, RSI, RDX, RCX, R8, R9
    // If the callee wishes to use registers RBP, RBX, and R12–R15,
    // it must restore their original values before returning control to the caller.
    // All other registers must be saved by the caller if it wishes to preserve their values.[

    #[naked]
    #[inline(never)]
    pub extern "C" fn switch_to(&mut self, next: &Thread) -> () {
        println!("switch_to enter cur:{:?} next:{:?}", self, next);
        unsafe {
            asm!("
            //mov rax, [rsp]
            //int 3
            push rbp
            push r15
            push r14
            push r13
            push r12
            push r11
            push r10
            push r9
            push r8
            push rsi
            push rdi
            push rdx
            push rcx
            push rbx
            push rax

            xor rax,rax //dummy rsp
            push rax



            // everything is now stored
            // save the stack pointer
            mov [rdi], rsp

            // switch to the other stack
            mov rsp, [rsi]

            // null out context ptr as we are now active in that context
            xor rax, rax
            mov [rsi], rax

            add rsp, 8 // skip dummy rsp


            // restore state
            pop rax
            pop rbx

            pop rcx
            pop rdx
            pop rdi
            pop rsi
            pop r8
            pop r9
            int 3
            pop r10
            pop r11
            pop r12
            pop r13
            pop r14
            pop r15
            pop rbp


            //mov rax, [rsp]
            //int 3

            ret
            "
            : // no outputs
            : "{rdi}"(self as *const Thread), "{rsi}"(next as *const Thread)//, s"(body as fn())
            : // no clobbers
            : "volatile", "intel");

        }
        panic!("Fell out of switch_to");
    }

}