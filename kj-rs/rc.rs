pub mod repr {
    use crate::repr::Own;
    use std::pin::Pin;

    pub unsafe trait Refcounted {
        fn is_shared(&self) -> bool;
        fn add_refcount_internal(&self) -> Own<Self>;
        fn add_rc_refcount_internal(rc: &Rc<Self>) -> Rc<Self>;
    }

    pub struct Rc<T: Refcounted + ?Sized> {
        pub(crate) own: Own<T>,
    }

    impl<T> Rc<T>
    where
        T: Refcounted
    {
        pub fn as_mut(&mut self) -> Pin<&mut T> {
            self.own.as_mut()
        }

        pub fn to_own(self) -> Own<T> {
            self.own
        }

        pub fn add_ref(&self) -> Rc<T> {
            Refcounted::add_rc_refcount_internal(self)
        }        
    }

    impl<T> AsRef<T> for Rc<T>
    where
        T: Refcounted
    {
        fn as_ref(&self) -> &T {
            self.own.as_ref()
        }
    }

    impl<T> Clone for Rc<T>
    where
        T: Refcounted
    {
        fn clone(&self) -> Self {
            self.add_ref()
        }
    }
}
