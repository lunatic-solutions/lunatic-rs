use std::marker::PhantomData;

/// Any function pointer that can be converted to and from usize
pub trait Fn<T> {
    fn id(self) -> usize;
    fn from_id(id: usize) -> T;
}

macro_rules! impl_types {
    ( $($ty:ident),* $(,)? ) => {
        impl<Res, $($ty,)*> Fn<Self> for fn($($ty,)*) -> Res
        {
            fn from_id(id: usize) -> Self {
                unsafe { std::mem::transmute(id) }
            }

            fn id(self) -> usize {
                self as usize
            }
        }
    };
}

// impl Fn<T> for most fn pointers
impl_types!();
impl_types!(T1);
impl_types!(T1, T2);
impl_types!(T1, T2, T3);
impl_types!(T1, T2, T3, T4);
impl_types!(T1, T2, T3, T4, T5);
impl_types!(T1, T2, T3, T4, T5, T6);
impl_types!(T1, T2, T3, T4, T5, T6, T7);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_types!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

/// Reference to a function that can be sent to another process
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FuncRef<T> {
    id: usize,
    phantom: PhantomData<T>,
}

impl<T> FuncRef<T>
where
    T: Fn<T>,
{
    pub fn new(func: T) -> Self {
        FuncRef {
            id: func.id(),
            phantom: PhantomData,
        }
    }

    pub fn get(self) -> T {
        T::from_id(self.id)
    }
}
