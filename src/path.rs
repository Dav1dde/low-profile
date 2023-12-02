use core::str::FromStr;

pub trait PathSegments {
    type Output;

    fn parse(&self, path: &str) -> Option<Self::Output>;
}

impl PathSegments for &'static str {
    type Output = ();

    fn parse(&self, path: &str) -> Option<Self::Output> {
        (self == &path).then_some(())
    }
}

macro_rules! impl_tuple_path_segments {
    ($($name:ident),*) => {
        impl<$($name),*> PathSegments for ($($name,)*)
        where
            $($name: PathSegments),*
        {
            type Output = ($($name::Output,)*);

            #[allow(non_snake_case)]
            fn parse(&self, path: &str) -> Option<Self::Output> {
                let mut parts = path.strip_prefix('/')?.split('/');

                let ($($name,)*) = self;
                $(
                    let $name = $name.parse(parts.next()?)?;
                )*

                match parts.next() {
                    Some(_) => None,
                    None => Some(($($name,)*)),
                }
            }
        }
    };
}

impl_tuple_path_segments!(T1);
impl_tuple_path_segments!(T1, T2);
impl_tuple_path_segments!(T1, T2, T3);
impl_tuple_path_segments!(T1, T2, T3, T4);
impl_tuple_path_segments!(T1, T2, T3, T4, T5);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_tuple_path_segments!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

pub struct ParseSegment<T: FromStr>(core::marker::PhantomData<T>);

impl<T> PathSegments for ParseSegment<T>
where
    T: FromStr,
{
    type Output = T;

    fn parse(&self, path: &str) -> Option<Self::Output> {
        path.parse().ok()
    }
}

pub trait Segment {
    type P: PathSegments<Output = Self>;

    fn segment() -> Self::P;
}

impl<T> Segment for T
where
    T: FromStr,
{
    type P = ParseSegment<T>;

    fn segment() -> Self::P {
        ParseSegment(Default::default())
    }
}
