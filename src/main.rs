use std::collections::HashMap;

struct BlockDesc{
    size : u8,
    start : usize,
    end : usize
}

struct BuddyAllocator{
    arena : Vec<u8>,
    heap_size : usize,
    free_list : HashMap<u8, Vec<BlockDesc>>
}

impl BuddyAllocator{
    fn new(heap_size : usize) -> Self{
        let mut m = HashMap::new();
        m.insert(4, vec![]);
        m.insert(8, vec![]);
        m.insert(16, vec![]);
        m.insert(32, vec![]);
        m.insert(64, vec![]);
        let mut v = Vec::with_capacity(heap_size);
        for i in 0..heap_size {
            v.push(0);
        }
        BuddyAllocator{
            arena : v,
            heap_size : heap_size,
            free_list : m
        }
    }

    fn alloc<T>(&mut self) -> *mut T{
        let size_needed = std::mem::size_of::<T>();
        let adjusted_size = BuddyAllocator::get_adjusted_size(size_needed);
        if adjusted_size as usize > self.heap_size {
            panic!("Out of memory!");
        }

        println!("size needed : {}, rounded size : {}", size_needed, adjusted_size);
        if self.free_list[&adjusted_size].is_empty(){

            
        }
        self.arena[0..4].as_mut_ptr() as *mut T 
    }

    fn get_adjusted_size(size : usize) -> u8{
        let mut _s = 0u8;
        for i in 2u32..{
            let d = 2i32.pow(i);
            if d >= size as i32{
                _s = d as u8;
                break;
            }
        }
        _s
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
fn test_get_adjusted_size(){
    assert_eq!(BuddyAllocator::get_adjusted_size(19usize), 32u8);
    assert_eq!(BuddyAllocator::get_adjusted_size(32usize), 32u8);
    assert_eq!(BuddyAllocator::get_adjusted_size(1usize), 4u8);
    assert_eq!(BuddyAllocator::get_adjusted_size(0usize), 4u8);
}
