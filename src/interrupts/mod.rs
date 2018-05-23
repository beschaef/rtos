use cpuio;
use memory::MemoryController;
use pic::ChainedPics;
use spin::{Mutex, Once};
use x86_64::structures::idt::{ExceptionStackFrame, Idt};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtualAddress;

mod gdt;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
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

           mov  rcx,10000

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
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    println!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn interrupt_handler(stack_frame: &mut ExceptionStackFrame) {
    //println!("\nEXCEPTION: Interrupt\n{:#?}", stack_frame);
    println!("interrupt");
    stack_frame.instruction_pointer = VirtualAddress(0x2134d5);
    stack_frame.stack_pointer = VirtualAddress(0x57ac001ffdf8);
    stack_frame.cpu_flags = 0x246;
}

pub fn trigger_test_interrupt() {
    println!("Triggering interrupt");
    unsafe {
        int!(0x20);
    }
    println!("Interrupt returned!");
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "x86-interrupt" fn timer_handler(stack_frame: &mut ExceptionStackFrame) {
    //println!("timer_handler");

    //reset timer
    unsafe {
        asm!("
                mov al, 0x34
                out 0x43, al

                mov rcx, 10000
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
        let mut p = cpuio::UnsafePort::new(0x60);
        let val: u8 = p.read();
        println!("{:?}", val);
    }
    println!("handler 1");
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
