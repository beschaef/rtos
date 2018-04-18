extern crate os_bootinfo;
use memory::{Frame, FrameAllocator};

use os_bootinfo::{MemoryMap, MemoryRegion};

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: MemoryRegion,
    areas: &'static MemoryMap,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        // in `allocate_frame` in `impl FrameAllocator for AreaFrameAllocator`
        println!("{:x?}", Some(self.current_area));

        if let Some(area) = Some(self.current_area) {
            // "Clone" the frame to return it if it's free. Frame doesn't
            // implement Clone, but we can construct an identical frame.
            let frame = Frame{ number: self.next_free_frame.number };

            // the last frame of the current area
            let current_area_last_frame = {
                let length = area.range.end_addr() - area.range.start_addr();
                let address = area.range.start_addr() + length -1;
                Frame::containing_address(address as usize)
            };

            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                // `frame` is used by the kernel
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame = Frame {
                    number: self.multiboot_end.number + 1
                };
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame.number += 1;
                return Some(frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frame()
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        // TODO (see below)
    }
}

impl AreaFrameAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize,
               multiboot_start: usize, multiboot_end: usize,
               memory_areas: &'static MemoryMap) -> AreaFrameAllocator
    {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(0),
            current_area: MemoryRegion::empty(),
            areas: &memory_areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = *self.areas.iter().clone().filter(|area| {
            let length = area.range.end_addr() - area.range.start_addr();
            let address = area.range.start_addr() + length -1;
            Frame::containing_address(address as usize) >= self.next_free_frame
        }).min_by_key(|area| area.range.start_addr()).unwrap();

        if let Some(area) = Some(self.current_area) {
            let start_frame = Frame::containing_address(area.range.start_addr() as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}