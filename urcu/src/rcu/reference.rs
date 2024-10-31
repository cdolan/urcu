use crate::rcu::callback::{RcuCallFn, RcuDeferFn};
use crate::rcu::{RcuContext, RcuDeferContext, RcuReadContext};

/// This trait defines a RCU reference that can be owned after a RCU grace period.
///
/// #### Safety
///
/// * The underlying reference must be cleaned up upon dropping (see below).
/// * There may be immutable borrows to the underlying reference.
/// * There must not be any mutable borrows to the underlying reference.
///
/// #### Dropping
///
/// An [`RcuRef`] should always cleanup when [`Drop::drop`] is executed by taking
/// ownership and dropping the underlying value.
///
/// * We cannot call [`RcuContext::rcu_synchronize`] since we can't be sure that
///   a RCU read lock is currently held or not[^mborrow].
///
/// Because an [`RcuRef`] can be sent to any thread, we cannot guarantee that a
/// thread executing [`Drop::drop`] is properly registered.
///
/// * We cannot call [`RcuContext::rcu_defer`] since we can't enforce that the
///   thread is registered with the RCU defer mecanisms[^mborrow].
/// * We cannot call [`RcuReadContext::rcu_call`] since we can't enforce that the
///   thread is registered with the RCU read mecanisms[^cborrow].
///
/// The only way to keep the safety guarantees of this crate is to use the custom
/// cleanup thread through [`RcuRef::safe_cleanup`]. It is similar to the built-in
/// [`RcuReadContext::rcu_call`], except it doesn't expect the calling thread to be
/// registered with RCU in any way.
///
/// The downside is that it is most likely worst than [`RcuReadContext::rcu_call`] in
/// every way. If it is a performance problem, the owner of an [`RcuRef`] can alway
/// use [`RcuRef::defer_cleanup`] and [`RcuRef::call_cleanup`] before [`Drop::drop`]
/// is called.
///
/// [^mborrow]: Unless your [`RcuRef`] has a mutable borrow of an [`RcuContext`].
/// [^cborrow]: Unless your [`RcuRef`] has an immutable borrow of an [`RcuContext`].
#[must_use]
pub unsafe trait RcuRef<C> {
    /// The output type after taking ownership.
    type Output;

    /// Take ownership of the reference.
    ///
    /// #### Safety
    ///
    /// You must wait for the grace period before taking ownership.
    unsafe fn take_ownership_unchecked(self) -> Self::Output;

    /// Take ownership of the reference.
    fn take_ownership(self, context: &mut C) -> Self::Output
    where
        Self: Sized,
        C: RcuContext,
    {
        context.rcu_synchronize();

        // SAFETY: RCU grace period has ended.
        unsafe { self.take_ownership_unchecked() }
    }

    /// Configure a cleanup callback to be called after the grace period.
    ///
    /// #### Note
    ///
    /// The function might internally call [`RcuContext::rcu_synchronize`] and block.
    ///
    /// The callback is guaranteed to be executed on the current thread.
    fn defer_cleanup(self, context: &mut C)
    where
        Self: Sized,
        C: RcuDeferContext,
    {
        context.rcu_defer(RcuDeferFn::<_, C>::new(move || {
            // SAFETY: The caller already executed a RCU syncronization.
            unsafe {
                self.take_ownership_unchecked();
            }
        }))
    }

    /// Configure a cleanup callback to be called after the grace period.
    ///
    /// #### Note
    ///
    /// The function will internally call [`RcuReadContext::rcu_read_lock`].
    ///
    /// The reference must implement [`Send`] since the cleanup will be executed in an helper thread.
    fn call_cleanup(self, context: &C)
    where
        Self: Sized + Send + 'static,
        C: RcuReadContext + 'static,
    {
        context.rcu_call(RcuCallFn::new(move || {
            // SAFETY: The caller already executed a RCU syncronization.
            unsafe {
                self.take_ownership_unchecked();
            }
        }));
    }

    fn safe_cleanup(self)
    where
        Self: Sized + Send + 'static,
        C: RcuContext,
    {
        C::rcu_cleanup(Box::new(move |context| {
            context.rcu_synchronize();

            // SAFETY: An RCU syncronization barrier was called.
            unsafe {
                self.take_ownership_unchecked();
            }
        }));
    }
}

/// #### Safety
///
/// It is the responsability of the underlying type to be safe.
unsafe impl<T, C> RcuRef<C> for Option<T>
where
    T: RcuRef<C>,
{
    type Output = Option<T::Output>;

    unsafe fn take_ownership_unchecked(self) -> Self::Output {
        self.map(|r| r.take_ownership_unchecked())
    }
}

/// #### Safety
///
/// It is the responsability of the underlying type to be safe.
unsafe impl<T, C> RcuRef<C> for Vec<T>
where
    T: RcuRef<C>,
{
    type Output = Vec<T::Output>;

    unsafe fn take_ownership_unchecked(self) -> Self::Output {
        self.into_iter()
            .map(|r| r.take_ownership_unchecked())
            .collect()
    }
}

macro_rules! impl_rcu_ref_for_tuple {
    ($($x:literal),*) => {
        paste::paste!{
            /// #### Safety
            ///
            /// It is the responsability of the underlying types to be safe.
            unsafe impl<$([<T $x>]),*, C> RcuRef<C> for ($([<T $x>]),*)
            where
                $([<T $x>]: RcuRef<C>),*,
            {
                type Output = ($([<T $x>]::Output),*,);

                unsafe fn take_ownership_unchecked(self) -> Self::Output {
                    (
                        $(self.$x.take_ownership_unchecked()),*,
                    )
                }
            }
        }
    };
}

impl_rcu_ref_for_tuple!(0, 1);
impl_rcu_ref_for_tuple!(0, 1, 2);
impl_rcu_ref_for_tuple!(0, 1, 2, 3);
impl_rcu_ref_for_tuple!(0, 1, 2, 3, 4);
impl_rcu_ref_for_tuple!(0, 1, 2, 3, 4, 5);
impl_rcu_ref_for_tuple!(0, 1, 2, 3, 4, 5, 6);
