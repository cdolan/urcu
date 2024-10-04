use urcu::flavor::RcuContextMemb;
use urcu::RcuList;
use urcu::{RcuContext, RcuRef};

fn main() {
    let mut context = RcuContextMemb::rcu_register().unwrap();
    let list = RcuList::<u32, RcuContextMemb>::new();

    let mut writer = list.writer().unwrap();
    writer.push_front(10);
    writer.push_front(20);
    writer.push_front(30);
    writer.push_front(40);
    writer.push_front(50);
    writer.push_front(60);

    let v10 = writer.pop_back().unwrap();
    let v20 = writer.pop_back().unwrap();
    writer.pop_back().unwrap().defer_cleanup(&mut context);
    writer.pop_back().unwrap().call_cleanup(&context);
    writer.pop_back().unwrap().safe_cleanup();

    let (v10, v20) = (v10, v20).take_ownership(&mut context);
    assert_eq!(*v10, 10);
    assert_eq!(*v20, 20);
}
