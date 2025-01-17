use urcu::prelude::*;

fn main() {
    let context = RcuDefaultFlavor::rcu_context_builder().with_read_context().register_thread().unwrap();

    let map = RcuHashMap::<u32, u32>::new().unwrap();
    let guard = context.rcu_read_lock();
    let value = map.get(&0, &guard);
    log::info!("{:?}", value);
    drop(guard);
    drop(map);
}
