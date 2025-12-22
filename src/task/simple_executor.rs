use core::{ptr::null, task::{Context, RawWaker, RawWakerVTable, Waker}};

use alloc::collections::vec_deque::VecDeque;

use crate::task::Task;

pub struct SimpleExecutor {
    queue: VecDeque<Task>
}

impl SimpleExecutor {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.queue.push_back(task);
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.queue.pop_front() {
            let waker = dummy_waker();
            let mut ctx = Context::from_waker(&waker);
            match task.poll(&mut ctx) {
                core::task::Poll::Ready(()) => {} // task doen
                core::task::Poll::Pending => self.queue.push_back(task), // "we will get back to it later" ahh
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(null(), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
