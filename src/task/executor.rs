use core::task::{Context, Poll, Waker};
use alloc::{collections::btree_map::BTreeMap, sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::{hlt, interrupts};
use crate::task::{Task, TaskId};

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>
}

impl TaskWaker {
    pub fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(Self {
            task_id,
            task_queue,
        }))
    }

    pub fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    waker_cache: BTreeMap<TaskId, Waker>,
    task_queue: Arc<ArrayQueue<TaskId>>
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            waker_cache: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("Queue full");
    }

    pub fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            waker_cache,
            task_queue,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut ctx = Context::from_waker(waker);
            match task.poll(&mut ctx) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                    //we already did "task_queue.pop", just to clear confusion
                }, // task doen
                Poll::Pending => {},
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&mut self) {
        interrupts::disable();
        if self.task_queue.is_empty() {
            interrupts::enable();
            hlt();
        } else {
            interrupts::enable();
        }
    }
}
