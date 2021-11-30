use alloc::boxed::Box;

use crate::println;

#[repr(C)]
pub struct TCBInfo {
    stack_pointer: usize,
}

impl TCBInfo {
    pub fn new(stack_pointer: usize) -> TCBInfo {
        TCBInfo {
            stack_pointer: stack_pointer,
        }
    }
}

type ThreadTask = dyn 'static + FnOnce() + Send + Sync;
pub trait TCB: Send + Sync {
    fn get_info(&mut self) -> *mut TCBInfo;
    fn get_work(&mut self) -> Box<ThreadTask>;
}

#[repr(C)]
pub struct TCBImpl {
    tcb_info: TCBInfo,
    stack: Box<[u64]>,
    work: Option<Box<ThreadTask>>,
}

impl TCBImpl {
    const NUM_CALLEE_SAVED: usize = 6;

    pub fn new(work: Box<ThreadTask>) -> TCBImpl {
        let mut stack: Box<[u64]> = box [0; 512];
        let end_of_stack = 511;
        stack[end_of_stack] = thread_entry_point as *const () as u64;
        let index: usize = end_of_stack - TCBImpl::NUM_CALLEE_SAVED - 1;
        stack[index] = 0; // Flags
        stack[index - 1] = 0; // CR2
        let stack_ptr = Box::into_raw(stack);
        let stack_ptr_as_usize = stack_ptr as *mut u64 as usize;
        stack = unsafe { Box::from_raw(stack_ptr) };
        let stack_ptr_start = stack_ptr_as_usize + ((index - 1) * core::mem::size_of::<usize>());
        let tcb_info = TCBInfo::new(stack_ptr_start);
        TCBImpl {
            tcb_info: tcb_info,
            stack: stack,
            work: Some(Box::new(work)),
        }
    }
}

impl TCB for TCBImpl {
    fn get_info(&mut self) -> *mut TCBInfo {
        &mut self.tcb_info as *mut TCBInfo
    }
    fn get_work(&mut self) -> Box<ThreadTask> {
        let mut work = None;
        core::mem::swap(&mut work, &mut self.work);
        match work {
            Some(task) => task,
            None => panic!("TCBImpl had no work!"),
        }
    }
}

extern "C" {
    fn context_switch(current: *mut TCBInfo, next: *mut TCBInfo);
}
pub fn context_switch_test() {
    let mut test1 = Box::new(TCBImpl::new(Box::new(move || ())));
    let mut test2 = Box::new(TCBImpl::new(Box::new(move || ())));
    unsafe {
        context_switch(test1.get_info(), test2.get_info());
    }
}

#[no_mangle]
pub extern "C" fn thread_entry_point() -> ! {
    println!("Thread made it to entry point!");
    loop {}
}
