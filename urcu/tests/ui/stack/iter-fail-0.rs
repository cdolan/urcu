use urcu::prelude::*;

fn main() {
    let context = RcuDefaultFlavor::rcu_context_builder().with_read_context().register_thread().unwrap();

    let stack = RcuStack::<u32>::new();
    let guard = context.rcu_read_lock();
    let mut iter = stack.iter(&guard);
    drop(stack);
    log::info!("{:?}", iter.next());
    drop(guard);
}
