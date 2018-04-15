use memory::PAGE_SIZE; // needed later
use self::paging::PhysicalAddress;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub struct Page {
    number: usize,
}

impl Page {

    fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }
}

