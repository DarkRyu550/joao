use crossbeam::queue::{ArrayQueue, SegQueue};
pub struct Pool<T> {
	nelem: usize,
	queue: ArrayQueue<T>,
	sleep: SegQueue<std::thread::Thread>
}
impl<T: Clone> Pool<T> {
	pub fn new(nelem: usize, model: T) -> Pool<T> {
		if nelem == 0 {
			panic!("Cannot create a pool with zero elements")
		}

		Pool {
			nelem: nelem,
			queue: {
				let mut queue = ArrayQueue::new(nelem);
				
				(0..nelem)
					.into_iter()
					.skip(1)
					.for_each(|_| 
						queue.push(model.clone())
							.expect("Cannot initialize queue")
					);
				queue.push(model)
					.expect("Cannot initialize queue");

				queue
			},
			sleep: SegQueue::new(),
		}
	}
}
impl<T> Pool<T> {
	pub fn init<F: FnMut(usize) -> T>(nelem: usize, mut generator: F) -> Pool<T> {
		if nelem == 0 {
			panic!("Cannot create a pool with zero elements")
		}

		Pool {
			nelem: nelem,
			queue: {
				let mut queue = ArrayQueue::new(nelem);

				(0..nelem)
					.into_iter()
					.for_each(|i| 
						queue.push(generator(i))
							.expect("Cannot initialize queue")
					);

				queue
			},
			sleep: SegQueue::new()
		}
	}

	pub fn borrow<'a>(&'a self) -> PoolGuard<'a, T> {
		loop {
			match self.queue.pop() {
				Ok(value) =>
					return PoolGuard {
						pool: &self,
						val: value
					},
				Err(_) => {
					self.sleep.push(std::thread::current());
					std::thread::park()
				}
			}
		}
	}

	pub fn try_borrow<'a>(&'a self) -> Option<PoolGuard<'a, T>> {
		self.queue.pop()
			.map(|val| 
				PoolGuard {
					pool: &self,
					val:  val
				}
			).ok()
	}

	fn wake_all(&self) {
		while let Ok(thread) = self.sleep.pop() {
			thread.unpark()
		}
	}
}

pub struct PoolGuard<'a, T> {
	pool: &'a Pool<T>,
	val:  T
}
impl<'a, T> Drop for PoolGuard<'a, T> {
	fn drop(&mut self) {
		/* Reclaim the object that was given to us so that we may return it */
		use std::mem;
		let mut a: T = unsafe {
			mem::uninitialized()
		};
		mem::swap(&mut a, &mut self.val);
			
		self.pool.queue.push(a)
			.expect("Cannot return borrowed value back to pool");

		self.pool.wake_all();
	}
}

use core::ops::Deref;
impl<'a, T> Deref for PoolGuard<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		&self.val
	}
}

use core::ops::DerefMut;
impl<'a, T> DerefMut for PoolGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.val
	}
}
