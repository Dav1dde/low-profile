use crate::macros::{composite_rejection, define_rejection};

define_rejection! {
    #[status = INTERNAL_SERVER_ERROR]
    #[body = "Failed to buffer the request body"]
    pub struct UnknownBodyError;
}

define_rejection! {
    #[status = PAYLOAD_TOO_LARGE]
    #[body = "Failed to buffer the request body"]
    /// Encountered some other error when buffering the body.
    pub struct BodyTooLarge;
}

define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Failed to read request body, invalid UTF-8"]
    pub struct InvalidUtf8;
}

define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Invalid JSON"]
    pub struct JsonError;
}

composite_rejection! {
    pub enum VecRejection {
        UnknownBodyError,
        BodyTooLarge,
    }
}

composite_rejection! {
    pub enum StringRejection {
        VecRejection,
        InvalidUtf8,
    }
}

composite_rejection! {
    pub enum JsonRejection {
        VecRejection,
        JsonError,
    }
}
