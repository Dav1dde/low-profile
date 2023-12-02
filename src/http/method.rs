use core::fmt;

use self::Inner::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Method<'a>(Inner<'a>);

impl<'a> Method<'a> {
    /// GET
    pub const GET: Method<'static> = Method(Get);
    /// POST
    pub const POST: Method<'static> = Method(Post);
    /// PUT
    pub const PUT: Method<'static> = Method(Put);
    /// DELETE
    pub const DELETE: Method<'static> = Method(Delete);
    /// HEAD
    pub const HEAD: Method<'static> = Method(Head);
    /// OPTIONS
    pub const OPTIONS: Method<'static> = Method(Options);
    /// CONNECT
    pub const CONNECT: Method<'static> = Method(Connect);
    /// PATCH
    pub const PATCH: Method<'static> = Method(Patch);
    /// TRACE
    pub const TRACE: Method<'static> = Method(Trace);

    /// Converts a str to a HTTP method.
    pub fn new(src: &'a str) -> Result<Method<'a>, InvalidMethod> {
        match src {
            "" => Err(InvalidMethod::new()),
            "GET" => Ok(Method(Get)),
            "PUT" => Ok(Method(Put)),
            "POST" => Ok(Method(Post)),
            "HEAD" => Ok(Method(Head)),
            "PATCH" => Ok(Method(Patch)),
            "TRACE" => Ok(Method(Trace)),
            "DELETE" => Ok(Method(Delete)),
            "OPTIONS" => Ok(Method(Options)),
            "CONNECT" => Ok(Method(Connect)),
            _ => Ok(Method(Extension(src))),
        }
    }

    pub fn as_str(&self) -> &str {
        match self.0 {
            Options => "OPTIONS",
            Get => "GET",
            Post => "POST",
            Put => "PUT",
            Delete => "DELETE",
            Head => "HEAD",
            Trace => "TRACE",
            Connect => "CONNECT",
            Patch => "PATCH",
            Extension(ext) => ext,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum Inner<'a> {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
    Extension(&'a str),
}

impl<'a> AsRef<str> for Method<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a, 'b> PartialEq<&'a Method<'b>> for Method<'b> {
    #[inline]
    fn eq(&self, other: &&'a Method<'b>) -> bool {
        self == *other
    }
}

impl<'a, 'b> PartialEq<Method<'b>> for &'a Method<'b> {
    #[inline]
    fn eq(&self, other: &Method<'b>) -> bool {
        *self == other
    }
}

impl<'a> PartialEq<str> for Method<'a> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl<'a> PartialEq<Method<'a>> for str {
    #[inline]
    fn eq(&self, other: &Method) -> bool {
        self == other.as_ref()
    }
}

impl<'a> PartialEq<&'a str> for Method<'a> {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref() == *other
    }
}

impl<'a> PartialEq<Method<'a>> for &'a str {
    #[inline]
    fn eq(&self, other: &Method) -> bool {
        *self == other.as_ref()
    }
}

impl<'a> fmt::Debug for Method<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl<'a> fmt::Display for Method<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.as_ref())
    }
}

impl<'a> Default for Method<'a> {
    #[inline]
    fn default() -> Method<'static> {
        Method::GET
    }
}

impl<'a, 'b> From<&'a Method<'b>> for Method<'b> {
    #[inline]
    fn from(t: &'a Method<'b>) -> Self {
        *t
    }
}

impl<'a> TryFrom<&'a str> for Method<'a> {
    type Error = InvalidMethod;

    #[inline]
    fn try_from(t: &'a str) -> Result<Self, Self::Error> {
        Self::new(t)
    }
}

/// A possible error value when converting `Method` from bytes.
pub struct InvalidMethod(());

impl InvalidMethod {
    fn new() -> Self {
        Self(())
    }
}

impl fmt::Debug for InvalidMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InvalidMethod").finish()
    }
}

impl fmt::Display for InvalidMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid HTTP method")
    }
}
