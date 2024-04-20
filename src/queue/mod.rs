use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex};
use std::collections::VecDeque;

struct Node<T> {
    value: T,
    next: AtomicPtr<Node<T>>,
}

pub struct Queue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Node<T> {
        Node {
            value,
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        let dummy = Box::new(Node::new(unsafe { std::mem::zeroed() }));
        let dummy_ptr = Box::into_raw(dummy);
        Queue {
            head: AtomicPtr::new(dummy_ptr),
            tail: AtomicPtr::new(dummy_ptr),
        }
    }

    pub fn enqueue(&self, value: T) {
        let new_node = Box::new(Node::new(value));
        let new_node_ptr = Box::into_raw(new_node);

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*tail).next.load(Ordering::Acquire) };

            if tail == self.tail.load(Ordering::Relaxed) {
                if next.is_null() {
                    if unsafe {
                        (*tail).next.compare_exchange(
                            ptr::null_mut(),
                            new_node_ptr,
                            Ordering::Release,
                            Ordering::Relaxed,
                        )
                    }
                    .is_ok()
                    {
                        break;
                    }
                } else {
                    self.tail.compare_exchange(
                        tail,
                        next,
                        Ordering::Release,
                        Ordering::Relaxed,
                    )
                    .ok();
                }
            }
        }

        self.tail.compare_exchange(
            self.tail.load(Ordering::Relaxed),
            new_node_ptr,
            Ordering::Release,
            Ordering::Relaxed,
        )
        .ok();
    }

    pub fn dequeue(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };

            if head == self.head.load(Ordering::Relaxed) {
                if head == tail {
                    if next.is_null() {
                        return None;  // Queue is empty
                    }
                    self.tail.compare_exchange(
                        tail,
                        next,
                        Ordering::Release,
                        Ordering::Relaxed,
                    )
                    .ok();
                } else {
                    if let Some(next_node) = unsafe { next.as_ref() } {
                        let res = unsafe { ptr::read(&next_node.value) };
                        if self.head.compare_exchange(
                            head,
                            next,
                            Ordering::Release,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                        {
                            unsafe { let _ = Box::from_raw(head); }
                            return Some(res);
                            
                        }
                    }
                }
            }
        }
    }
}


// Lock Based Implementation
pub struct LockQueue<T> {
    head: Mutex<VecDeque<T>>,
    tail: Mutex<VecDeque<T>>,
}

impl<T> LockQueue<T> {
    pub fn new() -> Self {
        LockQueue {
            head: Mutex::new(VecDeque::new()),
            tail: Mutex::new(VecDeque::new()),
        }
    }

    pub fn enqueue(&self, data: T) {
        let mut tail = self.tail.lock().unwrap();
        tail.push_back(data);

        // Move elements to head if empty and not locked
        if self.head.lock().unwrap().is_empty() {
            while let Some(value) = tail.pop_front() {
                self.head.lock().unwrap().push_back(value);
            }
        }
    }

    pub fn dequeue(&self) -> Option<T> {
        let mut head = self.head.lock().unwrap();
        if let Some(value) = head.pop_front() {
            return Some(value);
        }

        // If head is empty, try to transfer from tail
        drop(head); // Release head lock before acquiring tail lock
        let mut tail = self.tail.lock().unwrap();
        if tail.is_empty() {
            return None;
        }

        // Transfer elements from tail to head
        while let Some(value) = tail.pop_front() {
            self.head.lock().unwrap().push_back(value);
        }
        drop(tail); // Release tail lock

        // Try dequeue again
        self.head.lock().unwrap().pop_front()
    }
}


//Lock Based approach by thaodt
pub struct SingleVecLockQueue<T> {
    queue: Mutex<VecDeque<T>>,
}

impl<T> SingleVecLockQueue<T> {
    pub fn new() -> Self {
        SingleVecLockQueue {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn enqueue(&self, data: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(data);
    }

    pub fn dequeue(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }
}



