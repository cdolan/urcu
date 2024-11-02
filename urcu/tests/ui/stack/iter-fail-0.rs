use urcu::prelude::*;

fn main() {
    let context = DefaultContext::rcu_register().unwrap();

    let stack = RcuStack::<u32>::new();
    let guard = context.rcu_read_lock();
    let mut iter = stack.iter(&guard);
    drop(stack);
    log::info!("{:?}", iter.next());
    drop(guard);
}
