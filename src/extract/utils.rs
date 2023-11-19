// Borrowed from axum
macro_rules! define_rejection {
    (
        #[status = $status:ident]
        #[body = $body:expr]
        $(#[$m:meta])*
        pub struct $name:ident;
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        #[non_exhaustive]
        pub struct $name;

        impl $crate::response::IntoResponse for $name {
            type Body = &'static [u8];

            fn into_response(self) -> $crate::response::Response<Self::Body> {
                (self.status(), $body).into_response()
            }
        }

        impl $name {
            pub fn status(&self) -> $crate::http::StatusCode {
                $crate::http::StatusCode::$status
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", $body)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self
            }
        }
    };
}
pub(crate) use define_rejection;

// Also borrowed from axum
macro_rules! composite_rejection {
    (
        $(#[$m:meta])*
        pub enum $name:ident {
            $($variant:ident),+
            $(,)?
        }
    ) => {
        $(#[$m])*
        #[derive(Debug)]
        #[non_exhaustive]
        pub enum $name {
            $(
                #[allow(missing_docs)]
                $variant($variant)
            ),+
        }

        impl $crate::response::IntoResponse for $name {
            type Body = &'static [u8];

            fn into_response(self) -> $crate::response::Response<Self::Body> {
                match self {
                    $(
                        Self::$variant(inner) => inner.into_response(),
                    )+
                }
            }
        }

        impl $name {
            /// Get the status code used for this rejection.
            pub fn status(&self) -> $crate::http::StatusCode {
                match self {
                    $(
                        Self::$variant(inner) => inner.status(),
                    )+
                }
            }
        }

        $(
            impl From<$variant> for $name {
                fn from(inner: $variant) -> Self {
                    Self::$variant(inner)
                }
            }
        )+

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(
                        Self::$variant(inner) => write!(f, "{}", inner),
                    )+
                }
            }
        }
    };
}
pub(crate) use composite_rejection;
