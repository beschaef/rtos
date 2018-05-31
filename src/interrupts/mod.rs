use cpuio;
use features::keyboard;
use memory::MemoryController;
use pic::ChainedPics;
use scheduler::schedule;
use spin::{Mutex, Once};
use x86_64::structures::idt::{ExceptionStackFrame, Idt, PageFaultErrorCode};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtualAddress;

mod gdt;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.divide_by_zero.set_handler_fn(divide_by_zero);
        idt.debug.set_handler_fn(debug);
        idt.non_maskable_interrupt
            .set_handler_fn(non_maskable_interrupt);
        idt.overflow.set_handler_fn(overflow);
        idt.bound_range_exceeded
            .set_handler_fn(bound_range_exceeded);
        idt.invalid_opcode.set_handler_fn(invalid_opcode);
        idt.device_not_available
            .set_handler_fn(device_not_available);
        idt.invalid_tss.set_handler_fn(invalid_tss);
        idt.segment_not_present.set_handler_fn(segment_not_present);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault);
        idt.page_fault.set_handler_fn(page_fault);
        idt.x87_floating_point.set_handler_fn(x87_floating_point);

        idt.virtualization.set_handler_fn(virtualization);
        idt.security_exception.set_handler_fn(security_exception);
        idt.simd_floating_point.set_handler_fn(simd_floating_point);
        idt.machine_check.set_handler_fn(machine_check);
        idt.alignment_check.set_handler_fn(alignment_check);

        idt.interrupts[0].set_handler_fn(timer_handler);
        idt.interrupts[1].set_handler_fn(keyboard_handler);
        idt.interrupts[2].set_handler_fn(handler_2);
        idt.interrupts[3].set_handler_fn(handler_3);
        idt.interrupts[4].set_handler_fn(handler_4);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
        }
        idt
    };
}

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<gdt::Gdt> = Once::new();

const DOUBLE_FAULT_IST_INDEX: usize = 0;

static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(0x20, 0xA0) });

pub fn init(memory_controller: &mut MemoryController) {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;
    use x86_64::structures::gdt::SegmentSelector;

    let double_fault_stack = memory_controller
        .alloc_stack(1)
        .expect("could not allocate double fault stack");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
            VirtualAddress(double_fault_stack.top());
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = gdt::Gdt::new();
        code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(&tss));
        gdt
    });
    gdt.load();

    unsafe {
        // reload code segment register
        set_cs(code_selector);
        // load TSS
        load_tss(tss_selector);

        PICS.lock().initialize();
    }

    IDT.load();
}

pub fn init_timer() {
    unsafe {
        println!("init timer");
        asm!("
           cli
           mov  al,34h
           out  43h,al

           nop
           nop

           mov  rcx,60000

           mov  al,cl
           out  40h,al
           nop
           nop
           mov  al,ch
           out  40h,al
           nop
           nop

           sti
           hlt


            "
            :::: "intel","volatile");
    }
    println!("never read this!!!");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut ExceptionStackFrame) {
    println!("KRASSE EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    unsafe{PICS.lock().notify_end_of_interrupt(0x03 as u8);}
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    println!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn divide_by_zero(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: divide_by_zero\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn debug(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: debug\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn non_maskable_interrupt(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: non_maskable_interrupt\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn overflow(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: overflow\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn bound_range_exceeded(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: bound_range_exceeded\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn invalid_opcode(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: invalid_opcode\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn device_not_available(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: device_not_available\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn invalid_tss(stack_frame: &mut ExceptionStackFrame, _error_code: u64) {
    println!("EXCEPTION: invalid_tss\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn segment_not_present(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    println!("EXCEPTION: segment_not_present\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn stack_segment_fault(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    println!("EXCEPTION: stack_segment_fault\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn general_protection_fault(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    println!("EXCEPTION: general_protection_fault\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn page_fault(
    stack_frame: &mut ExceptionStackFrame,
    _page_error_struct: PageFaultErrorCode,
) {
    println!("EXCEPTION: page_fault\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn x87_floating_point(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: x87_floating_point\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn virtualization(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: virtualization\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn security_exception(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    println!("EXCEPTION: security_exception\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn simd_floating_point(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: simd_floating_point\n{:#?}", stack_frame);
    loop {}
}
extern "x86-interrupt" fn machine_check(stack_frame: &mut ExceptionStackFrame) {
    println!("EXCEPTION: machine_check\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn alignment_check(stack_frame: &mut ExceptionStackFrame, _error_code: u64) {
    println!("EXCEPTION: alignment_check\n{:#?}", stack_frame);
    loop {}
}

pub fn trigger_test_interrupt() {
    println!("Triggering interrupt");
    unsafe {
        int!(0x03);
    }
    println!("Interrupt returned!");
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "x86-interrupt" fn timer_handler(stack_frame: &mut ExceptionStackFrame) {
    //println!("timer_handler");

    schedule(stack_frame);

    //reset timer
    unsafe {
        asm!("
                mov al, 0x34
                out 0x43, al

                mov rcx, 60000
                mov al, cl
                out 0x40, al
                mov al, ch
                out 0x40, al


            "::::"intel", "volatile");

        PICS.lock().notify_end_of_interrupt(0x20 as u8);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "x86-interrupt" fn keyboard_handler(stack_frame: &mut ExceptionStackFrame) {
    unsafe {
        let mut scancode: u8 = cpuio::UnsafePort::new(0x60).read();
        if let Some(c) = keyboard::from_scancode(scancode as usize) {
            println!("{:?}", c);
            if c == 'h' {
                loop {}
            }
        }
    }
    //println!("handler 1");
    unsafe {
        PICS.lock().notify_end_of_interrupt(0x21 as u8);
    }
}

extern "x86-interrupt" fn handler_2(stack_frame: &mut ExceptionStackFrame) {
    println!("handler 2");
}
extern "x86-interrupt" fn handler_3(stack_frame: &mut ExceptionStackFrame) {
    println!("handler 3");
}
extern "x86-interrupt" fn handler_4(stack_frame: &mut ExceptionStackFrame) {
    println!("handler 4");
}
