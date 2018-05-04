pub use self::area_frame_allocator::AreaFrameAllocator;
use self::paging::PhysicalAddress;
pub use self::paging::test_paging;

mod area_frame_allocator;
//pub mod heap_allocator;
mod paging;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

pub const PAGE_SIZE: usize = 4096;

impl Frame {
    fn containing_address(address: usize) -> Frame {
        Frame {
            number: address / PAGE_SIZE,
        }
    }

    fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    fn clone(&self) -> Frame {
        Frame {
            number: self.number,
        }
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

use os_bootinfo::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    assert_has_not_been_called!("memory::init must be called only once");
    let memory_map_tag = &boot_info.memory_map;

    let mut frame_allocator = AreaFrameAllocator::new(memory_map_tag);
    unsafe {
        let mut active_table = paging::ActivePageTable::new();

        use self::paging::Page;
        use {HEAP_SIZE, HEAP_START};

        let heap_start_page = Page::containing_address(HEAP_START);
        let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

        for page in Page::range_inclusive(heap_start_page, heap_end_page) {
            active_table.map(page, paging::WRITABLE, &mut frame_allocator);
        }
    }
}
