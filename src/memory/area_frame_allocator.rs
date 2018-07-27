//! code of the `blog-os by phil oppermann`
extern crate os_bootinfo;
use memory::{Frame, FrameAllocator};

use os_bootinfo::{MemoryMap, MemoryRegion, MemoryRegionType};

pub struct AreaFrameAllocator {
    next_free_frame: Frame,
    current_area: Option<&'static MemoryRegion>,
    areas: &'static MemoryMap,
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        // in `allocate_frame` in `impl FrameAllocator for AreaFrameAllocator`

        if let Some(area) = self.current_area {
            // "Clone" the frame to return it if it's free. Frame doesn't
            // implement Clone, but we can construct an identical frame.
            let frame = Frame {
                number: self.next_free_frame.number,
            };

            // the last frame of the current area
            let current_area_last_frame = {
                let address = area.range.end_addr();
                Frame::containing_address(address as usize)
            };

            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if self.current_area
                .expect("area_frame_allocator allocate_frame failed")
                .region_type != MemoryRegionType::Usable
            {
                self.choose_next_area();
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
    pub fn new(memory_areas: &'static MemoryMap) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(0),
            current_area: None,
            areas: &memory_areas,
        };
        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.areas
            .iter()
            .clone()
            .filter(|area| {
                let address = area.range.end_addr() - 1;
                Frame::containing_address(address as usize) >= self.next_free_frame
                    && area.region_type == MemoryRegionType::Usable
            })
            .min_by_key(|area| area.range.start_addr());

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.range.start_addr() as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
