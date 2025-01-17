use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

use container_of::container_of;
use urcu_sys::RcuHead;

/// This trait defines a callback to be invoked after the next RCU grace period.
///
/// #### Implementation
///
/// Each flavor have a RCU linked list of `(callback, head)` pointers. Each RCU
/// threads can queue callbacks in this list. A helper thread will pop callbacks
/// and execute them with `callback(head)`.
///
/// #### Safety
///
/// When [`RcuCall::configure`] is called, you must deliberately leak your type
/// (e.g. [`Box::into_raw`]) to prevent the memory from being freed. Upon execution
/// of the callback, you must get back ownership (e.g. [`Box::from_raw`]) and properly
/// free up memory. For an example, see [`RcuCallFn`].
///
/// Unlike [`RcuDefer`], we resulting pointer must be an [`RcuHead`] inside
/// your data. You can use [`container_of!`] to get back the type implementing this
/// trait.
pub unsafe trait RcuCall {
    /// Configures the callback for execution.
    fn configure<F>(self: Box<Self>, func: F)
    where
        F: FnOnce(NonNull<RcuHead>, unsafe extern "C" fn(head: *mut RcuHead));
}

/// Defines a callback executed after the next RCU grace period.
pub struct RcuCallFn<F> {
    func: F,
    head: RcuHead,
}

impl<F> RcuCallFn<F> {
    /// Create a simple RCU callback.
    pub fn new(func: F) -> Box<Self> {
        Box::new(Self {
            func,
            head: Default::default(),
        })
    }

    unsafe extern "C" fn rcu_callback(head_ptr: *mut RcuHead)
    where
        F: FnOnce(),
    {
        // SAFETY: The pointers should always be valid.
        let node = Box::from_raw(container_of!(head_ptr, Self, head));

        (node.func)();
    }
}

/// #### Safety
///
/// The memory of [`Box<Self>`] is properly reclaimed upon the RCU callback.
unsafe impl<F> RcuCall for RcuCallFn<F>
where
    F: FnOnce(),
{
    fn configure<P>(self: Box<Self>, func: P)
    where
        P: FnOnce(NonNull<RcuHead>, unsafe extern "C" fn(head: *mut RcuHead)),
    {
        let node_ptr = Box::into_raw(self);
        let node = unsafe { &mut *node_ptr };

        unsafe {
            func(NonNull::new_unchecked(&mut node.head), Self::rcu_callback);
        }
    }
}

/// #### Safety
///
/// The callback can be sent to another thread if the reference implements [`Send`].
unsafe impl<F> Send for RcuCallFn<F> where F: Send {}

/// This trait defines a callback to be invoked after the next RCU grace period.
///
/// #### Implementation
///
/// Each RCU thread have an array of `(callback, data)` pointers. When the next RCU
/// grace period finishes, the thread goes over each of its entry and execute
/// `callback(data)`.
///
/// #### Safety
///
/// When [`RcuDefer::configure`] is called, you must deliberately leak your type
/// (e.g. [`Box::into_raw`]) to prevent the memory from being freed. Upon execution
/// of the callback, you must get back ownership (e.g. [`Box::from_raw`]) and properly
/// free up memory. For an example, see [`RcuDeferFn`].
pub unsafe trait RcuDefer {
    /// Configures the callback for execution.
    fn configure<F>(self: Box<Self>, func: F)
    where
        F: FnOnce(NonNull<c_void>, unsafe extern "C" fn(head: *mut c_void));
}

/// Defines a defer callback executed after the next RCU grace period.
pub struct RcuDeferFn<F, C> {
    func: F,
    // Also prevents Send+Sync auto-trait implementations.
    _context: PhantomData<*mut C>,
}

/// #### Safety
///
/// The memory of [`Box<Self>`] is properly reclaimed upon the RCU callback.
impl<F, C> RcuDeferFn<F, C> {
    /// Creates a callback.
    pub fn new(func: F) -> Box<Self> {
        Box::new(Self {
            func,
            _context: PhantomData,
        })
    }

    unsafe extern "C" fn callback(ptr: *mut c_void)
    where
        F: FnOnce(),
    {
        // SAFETY: The pointers should always be valid.
        let node = Box::from_raw(ptr as *mut Self);

        (node.func)();
    }
}

unsafe impl<F, C> RcuDefer for RcuDeferFn<F, C>
where
    F: FnOnce(),
{
    fn configure<P>(self: Box<Self>, func: P)
    where
        P: FnOnce(NonNull<c_void>, unsafe extern "C" fn(head: *mut c_void)),
    {
        let ptr = Box::into_raw(self) as *mut c_void;

        // SAFETY: The pointer is never null.
        unsafe { func(NonNull::new_unchecked(ptr), Self::callback) }
    }
}
