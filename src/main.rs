
struct BlockDesc{
    is_free : bool,
    size : usize,
    start : usize,
    end : usize
}

impl BlockDesc{
    fn new(is_free : bool, size : usize, start : usize, end : usize) -> Self{
        BlockDesc{
            is_free : is_free,
            size : size,
            start : start,
            end : end
        }
    }
}

struct BuddyAllocator{
    arena : Vec<u8>,
    heap_size : usize,
    min_block_size : usize,
    blocks_tree : Vec<BlockDesc>
}

impl BuddyAllocator{
    fn new(heap_size : usize) -> Self{
        let mut v = Vec::with_capacity(heap_size);
        for i in 0..heap_size {
            v.push(0);
        }
        BuddyAllocator{
            arena : v,
            min_block_size : 4,
            heap_size : heap_size,
            blocks_tree : Vec::new()
        }
    }

    fn alloc_with_size<T>(&mut self, size: usize) -> *mut T{
        let adjusted_order = BuddyAllocator::get_adjusted_order(size) as u8;
        if 1 << adjusted_order as usize > self.heap_size {
            panic!("Out of memory!");
        }

        //println!("size needed : {}, rounded order : {}", size_needed, adjusted_order);
        let required_size = (1 << adjusted_order as u32) as usize;

        let mut start = 0;
        let mut end = 0;
        {
            let desc = self.get_block(required_size);

            start = (desc.unwrap()).start;
            end = desc.unwrap().end;

        }

        return self.arena[start..end + 1].as_mut_ptr() as *mut T ;

    }

    fn alloc<T : Sized>(&mut self) -> *mut T{
        let size_needed = std::mem::size_of::<T>();
        let adjusted_order = BuddyAllocator::get_adjusted_order(size_needed) as u8;
        if 1 << adjusted_order as usize > self.heap_size {
            panic!("Out of memory!");
        }

        println!("size needed : {}, rounded order : {}", size_needed, adjusted_order);
        let required_size = 2i32.pow(adjusted_order as u32) as usize;

        let mut start = 0;
        let mut end = 0;
        {
            let desc = self.get_block(required_size);

            start = (desc.unwrap()).start;
            end = desc.unwrap().end;
        }

        return self.arena[start..end + 1].as_mut_ptr() as *mut T ;
        panic!("Out of memory exception");
    }

    fn free<T>(&mut self, p : *mut T){
        let p_size = std::mem::size_of::<T>();
        let diff = p as usize - self.arena[0..].as_mut_ptr() as usize;
        let i = diff / p_size;
        let arena_index = i * p_size;

        let (start, end) = BuddyAllocator::get_block_range_start_end(self.get_level(p_size));
        //TODO can this loop be avoided 
        let mut target_index = 0;
        for i in start..end+1{
            if arena_index == self.blocks_tree[i as usize].start{
                self.blocks_tree[i as usize].is_free = true;
                target_index = i;
                break;
            }
        }

        //go till root pairing up buddies
        if target_index != 0{
            while target_index != 0{
                let mut sib_index =
                if target_index & 1 == 1{
                    target_index + 1
                }
                else{
                    target_index - 1 
                };

                if self.blocks_tree[sib_index as usize].is_free{
                    target_index = (target_index - 1) / 2;
                    self.blocks_tree[target_index as usize].is_free = true; 
                }
                else{
                    break;
                }
            }
        }
    }
    
    fn get_block(&mut self, requested_size : usize) -> Option<&BlockDesc>{
        if self.blocks_tree.is_empty(){
            let mut size = self.heap_size;
            let height = self.get_level(self.min_block_size);

            //create left and right
            for i in 0..height + 1{
                let mut start = 0;
                let mut end = size - 1;
                for j in 0..(1 << i){ //lateral loop

                    self.blocks_tree.push(BlockDesc::new(true,
                                                         size,
                                                         start,
                                                         end));
                    start = end + 1;
                    end += start;
                }
                size /= 2;

            }
        }

        let (start, end) = BuddyAllocator::get_block_range_start_end(self.get_level(requested_size));
        for i in start..end+1{
            if self.blocks_tree[i as usize].is_free{
                self.blocks_tree[i as usize].is_free = false;
                let mut j = i;
                while j != 0{
                    j = (j - 1)/2;
                    self.blocks_tree[j as usize].is_free = false; 
                }
                return Some(&self.blocks_tree[i as usize])            
            }
        }

        //TODO run garbage collection here?
        panic!("OOM");

    }

    fn get_adjusted_order(size : usize) -> u8{
        let mut order = 0u8;
        for i in 2u32..{
            let d = 2i32.pow(i);
            if d >= size as i32{
                order = i as u8;
                break;
            }
        }
        order 
    }

    fn get_level(&self, block_size : usize) -> u32{
        let mut a = self.heap_size;
        let mut level = 0;
        while a != block_size{
            a /= 2;
            level += 1;
        }
        level
    }
    
    fn get_block_range_start_end(level : u32) -> (u32, u32){
        let start = 2i32.pow(level) - 1;
        (start as u32, (start * 2) as u32)
    }
}

fn main() {
    let mut ba = BuddyAllocator::new(1024);
    let mut p = ba.alloc::<i32>();
    unsafe
    {
        *p = 43;
        println!("{}", *p);
    }
}


#[test]
fn test_get_adjusted_order(){
    assert_eq!(BuddyAllocator::get_adjusted_order(19usize), 5u8);
    assert_eq!(BuddyAllocator::get_adjusted_order(32usize), 5u8);
    assert_eq!(BuddyAllocator::get_adjusted_order(1usize), 2u8);
    assert_eq!(BuddyAllocator::get_adjusted_order(0usize), 2u8);
}


#[test]
fn test_get_level(){
    let ba = BuddyAllocator::new(16);
    assert_eq!(ba.get_level(2), 3);
    assert_eq!(ba.get_level(4), 2);
    assert_eq!(ba.get_level(8), 1);
    assert_eq!(ba.get_level(16), 0);
}

#[test]
fn test_get_block_range_start_end(){
    let ba = BuddyAllocator::new(16);
    assert_eq!(BuddyAllocator::get_block_range_start_end(ba.get_level(2)), (7, 14));
    assert_eq!(BuddyAllocator::get_block_range_start_end(ba.get_level(4)), (3, 6));
    assert_eq!(BuddyAllocator::get_block_range_start_end(ba.get_level(8)), (1, 2));
    assert_eq!(BuddyAllocator::get_block_range_start_end(ba.get_level(16)), (0, 0));
}

#[test]
fn test_get_block(){
    let mut ba = BuddyAllocator::new(16);
    ba.get_block(16);
    assert_eq!(ba.blocks_tree.len(), 7);
}


#[test]
fn test_blocks_tree_creation(){
    let mut ba = BuddyAllocator::new(16);
    ba.get_block(16);
    assert_eq!(ba.blocks_tree.len(), 7);
    assert_eq!(ba.blocks_tree[0].start, 0);
    assert_eq!(ba.blocks_tree[0].end, 15);
    assert_eq!(ba.blocks_tree[1].start, 0);
    assert_eq!(ba.blocks_tree[1].end, 7);
    assert_eq!(ba.blocks_tree[2].start, 8);
    assert_eq!(ba.blocks_tree[2].end, 15);
    assert_eq!(ba.blocks_tree[3].start, 0);
    assert_eq!(ba.blocks_tree[3].end, 3);
    assert_eq!(ba.blocks_tree[4].start, 4);
    assert_eq!(ba.blocks_tree[4].end, 7);

    let mut ba = BuddyAllocator::new(32);
    ba.get_block(16);
    assert_eq!(ba.blocks_tree.len(), 15);
}

#[test]
fn test_data_store_i32(){
    let mut ba = BuddyAllocator::new(16);

    unsafe{
        let mut p = ba.alloc::<i32>();
        assert!(!ba.blocks_tree[3].is_free);
        assert!(ba.blocks_tree[4].is_free);
        *p = 4;
        assert_eq!(*p, 4);
    }
}

#[test]
fn test_data_store_2_i32(){
    let mut ba = BuddyAllocator::new(16);

    unsafe{
        let mut p = ba.alloc::<i32>();
        assert!(!ba.blocks_tree[3].is_free);
        assert!(ba.blocks_tree[4].is_free);
        *p = 4;
        assert_eq!(*p, 4);

        let mut p2 = ba.alloc::<i32>();
        assert!(!ba.blocks_tree[4].is_free);
    }
}

#[test]
fn test_allocation_causes_parents_marked(){
    let mut ba = BuddyAllocator::new(16);

    unsafe{
        let mut p = ba.alloc::<i32>();
        assert!(!ba.blocks_tree[3].is_free);
        assert!(!ba.blocks_tree[1].is_free);
        assert!(!ba.blocks_tree[0].is_free);
    }
}
#[test]
fn test_free(){
    let mut ba = BuddyAllocator::new(16);
    unsafe{
        let mut p = ba.alloc::<i32>();
        assert!(!ba.blocks_tree[3].is_free);
        ba.free::<i32>(p);
        assert!(ba.blocks_tree[3].is_free);
        let mut p2 = ba.alloc::<i32>();
        assert!(!ba.blocks_tree[3].is_free);
        ba.free::<i32>(p2);
        assert!(ba.blocks_tree[3].is_free);
        //let mut p3 = ba.alloc::<i32>();
        //assert_eq!(ba.free::<i32>(p3), 8);
    }
}

#[test]
fn test_little_endianness(){
    let mut ba = BuddyAllocator::new(16);
    unsafe{
        let mut p = ba.alloc::<i32>();
        *p = 4;
        let u8_ptr : *const u8 = std::mem::transmute(p);
        assert_eq!(*u8_ptr, 4);
        assert_eq!(*u8_ptr.offset(1), 0);
        assert_eq!(*u8_ptr.offset(2), 0);
        assert_eq!(*u8_ptr.offset(3), 0);

    }
}

#[test]
fn test_free_traverse_till_root_clubbing_buddies(){
    let mut ba = BuddyAllocator::new(16);
    unsafe{
        let mut p = ba.alloc::<i32>();
        assert!(!ba.blocks_tree[3].is_free);
        assert!(!ba.blocks_tree[1].is_free);
        assert!(!ba.blocks_tree[0].is_free);

        ba.free::<i32>(p);

        assert!(ba.blocks_tree[3].is_free);
        assert!(ba.blocks_tree[1].is_free);
        assert!(ba.blocks_tree[0].is_free);
    }
}
