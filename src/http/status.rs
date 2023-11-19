//! HTTP status codes
//!
//! Module temporarily copied from the `http` crate until there is `no_std` support.
//!
//! This module contains HTTP-status code related structs an errors. The main
//! type in this module is `StatusCode` which is not intended to be used through
//! this module but rather the `http::StatusCode` type.
//!
//! # Examples
//!
//! ```
//! use http::StatusCode;
//!
//! assert_eq!(StatusCode::from_u16(200).unwrap(), StatusCode::OK);
//! assert_eq!(StatusCode::NOT_FOUND, 404);
//! assert!(StatusCode::OK.is_success());
//! ```

use core::{fmt, num::NonZeroU16};

/// An HTTP status code (`status-code` in RFC 7230 et al.).
///
/// Constants are provided for known status codes, including those in the IANA
/// [HTTP Status Code Registry](
/// https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml).
///
/// Status code values in the range 100-999 (inclusive) are supported by this
/// type. Values in the range 100-599 are semantically classified by the most
/// significant digit. See [`StatusCode::is_success`], etc. Values above 599
/// are unclassified but allowed for legacy compatibility, though their use is
/// discouraged. Applications may interpret such values as protocol errors.
///
/// # Examples
///
/// ```
/// use http::StatusCode;
///
/// assert_eq!(StatusCode::from_u16(200).unwrap(), StatusCode::OK);
/// assert_eq!(StatusCode::NOT_FOUND.as_u16(), 404);
/// assert!(StatusCode::OK.is_success());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatusCode(NonZeroU16);

/// A possible error value when converting a `StatusCode` from a `u16` or `&str`
///
/// This error indicates that the supplied input was not a valid number, was less
/// than 100, or was greater than 999.
pub struct InvalidStatusCode {
    _priv: (),
}

impl StatusCode {
    /// Converts a u16 to a status code.
    ///
    /// The function validates the correctness of the supplied u16. It must be
    /// greater or equal to 100 and less than 1000.
    ///
    /// # Example
    ///
    /// ```
    /// use http::StatusCode;
    ///
    /// let ok = StatusCode::from_u16(200).unwrap();
    /// assert_eq!(ok, StatusCode::OK);
    ///
    /// let err = StatusCode::from_u16(99);
    /// assert!(err.is_err());
    /// ```
    #[inline]
    pub fn from_u16(src: u16) -> Result<StatusCode, InvalidStatusCode> {
        if !(100..1000).contains(&src) {
            return Err(InvalidStatusCode::new());
        }

        NonZeroU16::new(src)
            .map(StatusCode)
            .ok_or_else(InvalidStatusCode::new)
    }

    /// Converts a &[u8] to a status code
    pub fn from_bytes(src: &[u8]) -> Result<StatusCode, InvalidStatusCode> {
        if src.len() != 3 {
            return Err(InvalidStatusCode::new());
        }

        let a = src[0].wrapping_sub(b'0') as u16;
        let b = src[1].wrapping_sub(b'0') as u16;
        let c = src[2].wrapping_sub(b'0') as u16;

        if a == 0 || a > 9 || b > 9 || c > 9 {
            return Err(InvalidStatusCode::new());
        }

        let status = (a * 100) + (b * 10) + c;
        NonZeroU16::new(status)
            .map(StatusCode)
            .ok_or_else(InvalidStatusCode::new)
    }

    /// Returns the `u16` corresponding to this `StatusCode`.
    ///
    /// # Note
    ///
    /// This is the same as the `From<StatusCode>` implementation, but
    /// included as an inherent method because that implementation doesn't
    /// appear in rustdocs, as well as a way to force the type instead of
    /// relying on inference.
    ///
    /// # Example
    ///
    /// ```
    /// let status = http::StatusCode::OK;
    /// assert_eq!(status.as_u16(), 200);
    /// ```
    #[inline]
    pub fn as_u16(&self) -> u16 {
        (*self).into()
    }

    /// Get the standardised `reason-phrase` for this status code.
    ///
    /// This is mostly here for servers writing responses, but could potentially have application
    /// at other times.
    ///
    /// The reason phrase is defined as being exclusively for human readers. You should avoid
    /// deriving any meaning from it at all costs.
    ///
    /// Bear in mind also that in HTTP/2.0 and HTTP/3.0 the reason phrase is abolished from
    /// transmission, and so this canonical reason phrase really is the only reason phrase you’ll
    /// find.
    ///
    /// # Example
    ///
    /// ```
    /// let status = http::StatusCode::OK;
    /// assert_eq!(status.canonical_reason(), Some("OK"));
    /// ```
    pub fn canonical_reason(&self) -> Option<&'static str> {
        canonical_reason(self.0.get())
    }

    /// Check if status is within 100-199.
    #[inline]
    pub fn is_informational(&self) -> bool {
        200 > self.0.get() && self.0.get() >= 100
    }

    /// Check if status is within 200-299.
    #[inline]
    pub fn is_success(&self) -> bool {
        300 > self.0.get() && self.0.get() >= 200
    }

    /// Check if status is within 300-399.
    #[inline]
    pub fn is_redirection(&self) -> bool {
        400 > self.0.get() && self.0.get() >= 300
    }

    /// Check if status is within 400-499.
    #[inline]
    pub fn is_client_error(&self) -> bool {
        500 > self.0.get() && self.0.get() >= 400
    }

    /// Check if status is within 500-599.
    #[inline]
    pub fn is_server_error(&self) -> bool {
        600 > self.0.get() && self.0.get() >= 500
    }
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

/// Formats the status code, *including* the canonical reason.
///
/// # Example
///
/// ```
/// # use http::StatusCode;
/// assert_eq!(format!("{}", StatusCode::OK), "200 OK");
/// ```
impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            u16::from(*self),
            self.canonical_reason().unwrap_or("<unknown status code>")
        )
    }
}

impl Default for StatusCode {
    #[inline]
    fn default() -> StatusCode {
        StatusCode::OK
    }
}

impl PartialEq<u16> for StatusCode {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.as_u16() == *other
    }
}

impl PartialEq<StatusCode> for u16 {
    #[inline]
    fn eq(&self, other: &StatusCode) -> bool {
        *self == other.as_u16()
    }
}

impl From<StatusCode> for u16 {
    #[inline]
    fn from(status: StatusCode) -> u16 {
        status.0.get()
    }
}

impl core::str::FromStr for StatusCode {
    type Err = InvalidStatusCode;

    fn from_str(s: &str) -> Result<StatusCode, InvalidStatusCode> {
        StatusCode::from_bytes(s.as_ref())
    }
}

impl<'a> From<&'a StatusCode> for StatusCode {
    #[inline]
    fn from(t: &'a StatusCode) -> Self {
        *t
    }
}

impl<'a> TryFrom<&'a [u8]> for StatusCode {
    type Error = InvalidStatusCode;

    #[inline]
    fn try_from(t: &'a [u8]) -> Result<Self, Self::Error> {
        StatusCode::from_bytes(t)
    }
}

impl<'a> TryFrom<&'a str> for StatusCode {
    type Error = InvalidStatusCode;

    #[inline]
    fn try_from(t: &'a str) -> Result<Self, Self::Error> {
        t.parse()
    }
}

impl TryFrom<u16> for StatusCode {
    type Error = InvalidStatusCode;

    #[inline]
    fn try_from(t: u16) -> Result<Self, Self::Error> {
        StatusCode::from_u16(t)
    }
}

macro_rules! status_codes {
    (
        $(
            $(#[$docs:meta])*
            ($num:expr, $konst:ident, $phrase:expr);
        )+
    ) => {
        impl StatusCode {
        $(
            $(#[$docs])*
            pub const $konst: StatusCode = StatusCode(unsafe { NonZeroU16::new_unchecked($num) });
        )+

        }

        fn canonical_reason(num: u16) -> Option<&'static str> {
            match num {
                $(
                $num => Some($phrase),
                )+
                _ => None
            }
        }
    }
}

status_codes! {
    /// 100 Continue
    /// [[RFC7231, Section 6.2.1](https://tools.ietf.org/html/rfc7231#section-6.2.1)]
    (100, CONTINUE, "Continue");
    /// 101 Switching Protocols
    /// [[RFC7231, Section 6.2.2](https://tools.ietf.org/html/rfc7231#section-6.2.2)]
    (101, SWITCHING_PROTOCOLS, "Switching Protocols");
    /// 102 Processing
    /// [[RFC2518](https://tools.ietf.org/html/rfc2518)]
    (102, PROCESSING, "Processing");

    /// 200 OK
    /// [[RFC7231, Section 6.3.1](https://tools.ietf.org/html/rfc7231#section-6.3.1)]
    (200, OK, "OK");
    /// 201 Created
    /// [[RFC7231, Section 6.3.2](https://tools.ietf.org/html/rfc7231#section-6.3.2)]
    (201, CREATED, "Created");
    /// 202 Accepted
    /// [[RFC7231, Section 6.3.3](https://tools.ietf.org/html/rfc7231#section-6.3.3)]
    (202, ACCEPTED, "Accepted");
    /// 203 Non-Authoritative Information
    /// [[RFC7231, Section 6.3.4](https://tools.ietf.org/html/rfc7231#section-6.3.4)]
    (203, NON_AUTHORITATIVE_INFORMATION, "Non Authoritative Information");
    /// 204 No Content
    /// [[RFC7231, Section 6.3.5](https://tools.ietf.org/html/rfc7231#section-6.3.5)]
    (204, NO_CONTENT, "No Content");
    /// 205 Reset Content
    /// [[RFC7231, Section 6.3.6](https://tools.ietf.org/html/rfc7231#section-6.3.6)]
    (205, RESET_CONTENT, "Reset Content");
    /// 206 Partial Content
    /// [[RFC7233, Section 4.1](https://tools.ietf.org/html/rfc7233#section-4.1)]
    (206, PARTIAL_CONTENT, "Partial Content");
    /// 207 Multi-Status
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (207, MULTI_STATUS, "Multi-Status");
    /// 208 Already Reported
    /// [[RFC5842](https://tools.ietf.org/html/rfc5842)]
    (208, ALREADY_REPORTED, "Already Reported");

    /// 226 IM Used
    /// [[RFC3229](https://tools.ietf.org/html/rfc3229)]
    (226, IM_USED, "IM Used");

    /// 300 Multiple Choices
    /// [[RFC7231, Section 6.4.1](https://tools.ietf.org/html/rfc7231#section-6.4.1)]
    (300, MULTIPLE_CHOICES, "Multiple Choices");
    /// 301 Moved Permanently
    /// [[RFC7231, Section 6.4.2](https://tools.ietf.org/html/rfc7231#section-6.4.2)]
    (301, MOVED_PERMANENTLY, "Moved Permanently");
    /// 302 Found
    /// [[RFC7231, Section 6.4.3](https://tools.ietf.org/html/rfc7231#section-6.4.3)]
    (302, FOUND, "Found");
    /// 303 See Other
    /// [[RFC7231, Section 6.4.4](https://tools.ietf.org/html/rfc7231#section-6.4.4)]
    (303, SEE_OTHER, "See Other");
    /// 304 Not Modified
    /// [[RFC7232, Section 4.1](https://tools.ietf.org/html/rfc7232#section-4.1)]
    (304, NOT_MODIFIED, "Not Modified");
    /// 305 Use Proxy
    /// [[RFC7231, Section 6.4.5](https://tools.ietf.org/html/rfc7231#section-6.4.5)]
    (305, USE_PROXY, "Use Proxy");
    /// 307 Temporary Redirect
    /// [[RFC7231, Section 6.4.7](https://tools.ietf.org/html/rfc7231#section-6.4.7)]
    (307, TEMPORARY_REDIRECT, "Temporary Redirect");
    /// 308 Permanent Redirect
    /// [[RFC7238](https://tools.ietf.org/html/rfc7238)]
    (308, PERMANENT_REDIRECT, "Permanent Redirect");

    /// 400 Bad Request
    /// [[RFC7231, Section 6.5.1](https://tools.ietf.org/html/rfc7231#section-6.5.1)]
    (400, BAD_REQUEST, "Bad Request");
    /// 401 Unauthorized
    /// [[RFC7235, Section 3.1](https://tools.ietf.org/html/rfc7235#section-3.1)]
    (401, UNAUTHORIZED, "Unauthorized");
    /// 402 Payment Required
    /// [[RFC7231, Section 6.5.2](https://tools.ietf.org/html/rfc7231#section-6.5.2)]
    (402, PAYMENT_REQUIRED, "Payment Required");
    /// 403 Forbidden
    /// [[RFC7231, Section 6.5.3](https://tools.ietf.org/html/rfc7231#section-6.5.3)]
    (403, FORBIDDEN, "Forbidden");
    /// 404 Not Found
    /// [[RFC7231, Section 6.5.4](https://tools.ietf.org/html/rfc7231#section-6.5.4)]
    (404, NOT_FOUND, "Not Found");
    /// 405 Method Not Allowed
    /// [[RFC7231, Section 6.5.5](https://tools.ietf.org/html/rfc7231#section-6.5.5)]
    (405, METHOD_NOT_ALLOWED, "Method Not Allowed");
    /// 406 Not Acceptable
    /// [[RFC7231, Section 6.5.6](https://tools.ietf.org/html/rfc7231#section-6.5.6)]
    (406, NOT_ACCEPTABLE, "Not Acceptable");
    /// 407 Proxy Authentication Required
    /// [[RFC7235, Section 3.2](https://tools.ietf.org/html/rfc7235#section-3.2)]
    (407, PROXY_AUTHENTICATION_REQUIRED, "Proxy Authentication Required");
    /// 408 Request Timeout
    /// [[RFC7231, Section 6.5.7](https://tools.ietf.org/html/rfc7231#section-6.5.7)]
    (408, REQUEST_TIMEOUT, "Request Timeout");
    /// 409 Conflict
    /// [[RFC7231, Section 6.5.8](https://tools.ietf.org/html/rfc7231#section-6.5.8)]
    (409, CONFLICT, "Conflict");
    /// 410 Gone
    /// [[RFC7231, Section 6.5.9](https://tools.ietf.org/html/rfc7231#section-6.5.9)]
    (410, GONE, "Gone");
    /// 411 Length Required
    /// [[RFC7231, Section 6.5.10](https://tools.ietf.org/html/rfc7231#section-6.5.10)]
    (411, LENGTH_REQUIRED, "Length Required");
    /// 412 Precondition Failed
    /// [[RFC7232, Section 4.2](https://tools.ietf.org/html/rfc7232#section-4.2)]
    (412, PRECONDITION_FAILED, "Precondition Failed");
    /// 413 Payload Too Large
    /// [[RFC7231, Section 6.5.11](https://tools.ietf.org/html/rfc7231#section-6.5.11)]
    (413, PAYLOAD_TOO_LARGE, "Payload Too Large");
    /// 414 URI Too Long
    /// [[RFC7231, Section 6.5.12](https://tools.ietf.org/html/rfc7231#section-6.5.12)]
    (414, URI_TOO_LONG, "URI Too Long");
    /// 415 Unsupported Media Type
    /// [[RFC7231, Section 6.5.13](https://tools.ietf.org/html/rfc7231#section-6.5.13)]
    (415, UNSUPPORTED_MEDIA_TYPE, "Unsupported Media Type");
    /// 416 Range Not Satisfiable
    /// [[RFC7233, Section 4.4](https://tools.ietf.org/html/rfc7233#section-4.4)]
    (416, RANGE_NOT_SATISFIABLE, "Range Not Satisfiable");
    /// 417 Expectation Failed
    /// [[RFC7231, Section 6.5.14](https://tools.ietf.org/html/rfc7231#section-6.5.14)]
    (417, EXPECTATION_FAILED, "Expectation Failed");
    /// 418 I'm a teapot
    /// [curiously not registered by IANA but [RFC2324](https://tools.ietf.org/html/rfc2324)]
    (418, IM_A_TEAPOT, "I'm a teapot");

    /// 421 Misdirected Request
    /// [RFC7540, Section 9.1.2](http://tools.ietf.org/html/rfc7540#section-9.1.2)
    (421, MISDIRECTED_REQUEST, "Misdirected Request");
    /// 422 Unprocessable Entity
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (422, UNPROCESSABLE_ENTITY, "Unprocessable Entity");
    /// 423 Locked
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (423, LOCKED, "Locked");
    /// 424 Failed Dependency
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (424, FAILED_DEPENDENCY, "Failed Dependency");

    /// 426 Upgrade Required
    /// [[RFC7231, Section 6.5.15](https://tools.ietf.org/html/rfc7231#section-6.5.15)]
    (426, UPGRADE_REQUIRED, "Upgrade Required");

    /// 428 Precondition Required
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (428, PRECONDITION_REQUIRED, "Precondition Required");
    /// 429 Too Many Requests
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (429, TOO_MANY_REQUESTS, "Too Many Requests");

    /// 431 Request Header Fields Too Large
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (431, REQUEST_HEADER_FIELDS_TOO_LARGE, "Request Header Fields Too Large");

    /// 451 Unavailable For Legal Reasons
    /// [[RFC7725](http://tools.ietf.org/html/rfc7725)]
    (451, UNAVAILABLE_FOR_LEGAL_REASONS, "Unavailable For Legal Reasons");

    /// 500 Internal Server Error
    /// [[RFC7231, Section 6.6.1](https://tools.ietf.org/html/rfc7231#section-6.6.1)]
    (500, INTERNAL_SERVER_ERROR, "Internal Server Error");
    /// 501 Not Implemented
    /// [[RFC7231, Section 6.6.2](https://tools.ietf.org/html/rfc7231#section-6.6.2)]
    (501, NOT_IMPLEMENTED, "Not Implemented");
    /// 502 Bad Gateway
    /// [[RFC7231, Section 6.6.3](https://tools.ietf.org/html/rfc7231#section-6.6.3)]
    (502, BAD_GATEWAY, "Bad Gateway");
    /// 503 Service Unavailable
    /// [[RFC7231, Section 6.6.4](https://tools.ietf.org/html/rfc7231#section-6.6.4)]
    (503, SERVICE_UNAVAILABLE, "Service Unavailable");
    /// 504 Gateway Timeout
    /// [[RFC7231, Section 6.6.5](https://tools.ietf.org/html/rfc7231#section-6.6.5)]
    (504, GATEWAY_TIMEOUT, "Gateway Timeout");
    /// 505 HTTP Version Not Supported
    /// [[RFC7231, Section 6.6.6](https://tools.ietf.org/html/rfc7231#section-6.6.6)]
    (505, HTTP_VERSION_NOT_SUPPORTED, "HTTP Version Not Supported");
    /// 506 Variant Also Negotiates
    /// [[RFC2295](https://tools.ietf.org/html/rfc2295)]
    (506, VARIANT_ALSO_NEGOTIATES, "Variant Also Negotiates");
    /// 507 Insufficient Storage
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (507, INSUFFICIENT_STORAGE, "Insufficient Storage");
    /// 508 Loop Detected
    /// [[RFC5842](https://tools.ietf.org/html/rfc5842)]
    (508, LOOP_DETECTED, "Loop Detected");

    /// 510 Not Extended
    /// [[RFC2774](https://tools.ietf.org/html/rfc2774)]
    (510, NOT_EXTENDED, "Not Extended");
    /// 511 Network Authentication Required
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (511, NETWORK_AUTHENTICATION_REQUIRED, "Network Authentication Required");
}

impl InvalidStatusCode {
    fn new() -> InvalidStatusCode {
        InvalidStatusCode { _priv: () }
    }
}

impl fmt::Debug for InvalidStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InvalidStatusCode").finish()
    }
}

impl fmt::Display for InvalidStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid status code")
    }
}
