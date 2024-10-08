pub(crate) mod reference;

pub use crate::shared::reference::*;

mod asserts {
    use super::*;

    use static_assertions::{assert_impl_all, assert_not_impl_all};

    use crate::rcu::DefaultContext;
    use crate::utility::asserts::*;

    mod rcu_ref {
        use super::*;

        // T: Send + !Sync
        assert_impl_all!(BoxRef<SendButNotSync, DefaultContext>: Send);
        assert_not_impl_all!(BoxRef<SendButNotSync, DefaultContext>: Sync);

        // T: Send + Sync
        assert_impl_all!(BoxRef<SendAndSync, DefaultContext>: Send);
        assert_not_impl_all!(BoxRef<SendAndSync, DefaultContext>: Sync);
    }

    mod rcu_ref_owned {
        use super::*;

        // T: !Send + !Sync
        assert_not_impl_all!(BoxRefOwned<NotSendNotSync>: Send);
        assert_not_impl_all!(BoxRefOwned<NotSendNotSync>: Sync);

        // T: Send + !Sync
        assert_impl_all!(BoxRefOwned<SendButNotSync>: Send);
        assert_not_impl_all!(BoxRefOwned<SendButNotSync>: Sync);

        // T: !Send + Sync
        assert_not_impl_all!(BoxRefOwned<NotSendButSync>: Send);
        assert_impl_all!(BoxRefOwned<NotSendButSync>: Sync);

        // T: Send + Sync
        assert_impl_all!(BoxRefOwned<SendAndSync>: Send);
        assert_impl_all!(BoxRefOwned<SendAndSync>: Sync);
    }
}