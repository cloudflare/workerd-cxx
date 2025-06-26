pub mod repr {
    use crate::repr::Own;

    pub trait Refcounted {
        fn is_shared() -> u32;
    }

    pub struct Rc<T> {
        pub(crate) own: Own<T>,
    }

    impl<T> Rc<T> {
        fn to_own(self) -> Own<T> {
            self.own
        }

        fn add_ref(&mut self) -> Own<T> {
            todo!()
        }        
    }

    impl<T> Clone for Rc<T> {
        fn clone(&self) -> Self {
            todo!()
        }
    }
}
