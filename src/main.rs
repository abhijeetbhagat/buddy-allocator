
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

struct Node{
    left : Option<Box<Node>>,
    block_desc : BlockDesc,
    right : Option<Box<Node>>,
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

    fn alloc<T>(&mut self) -> *mut T{
        let size_needed = std::mem::size_of::<T>();
        let adjusted_order = BuddyAllocator::get_adjusted_order(size_needed) as u8;
        if adjusted_order as usize > self.heap_size {
            panic!("Out of memory!");
        }

        println!("size needed : {}, rounded size : {}", size_needed, adjusted_order);
        let required_size = 2i32.pow(adjusted_order as u32) as usize;

        let desc = self.get_block(required_size);

        //self.arena[0..4].as_mut_ptr() as *mut T 
        std::ptr::null_mut()
    }

    fn get_block(&mut self, requested_size : usize) -> Option<&BlockDesc>{
        if self.blocks_tree.is_empty(){
            let mut start = 0;
            let mut end = self.heap_size - 1;
            let mut size = self.heap_size;

            //create root
            let block = BlockDesc::new(false,
                                       size,
                                       start,
                                       end);
            self.blocks_tree.push(block);
            if size == requested_size{
                return self.blocks_tree.last();
            }

            let mut left_child_index = 1;

            //create left and right
            loop{

                let middle = size / 2;

                let left = BlockDesc::new(true,
                                          size,
                                          start,
                                          middle - 1);
                self.blocks_tree.push(left);
                /*if size == requested_size{
                    self.blocks_tree.last_mut().is_free = false;
                    return self.blocks_tree.last(); 
                }*/

                let right = BlockDesc::new(true,
                                          size,
                                          middle,
                                          end);
                self.blocks_tree.push(right);

                if middle == requested_size{
                    self.blocks_tree.last_mut().unwrap().is_free = false;
                    return Some(&self.blocks_tree[left_child_index]); 
                }

                left_child_index = 2 * left_child_index + 1;
                size = middle;
                end = middle - 1;
            }
        }
        None
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
fn test_get_block(){
    let mut ba = BuddyAllocator::new(64);
    {
        let block_desc = ba.get_block(32).unwrap();
    }
    assert_eq!(ba.blocks_tree.len(), 3);

}
