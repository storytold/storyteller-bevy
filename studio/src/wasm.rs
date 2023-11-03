use std::{cell::RefCell, sync::OnceLock};

use parking_lot::ReentrantMutex;

pub(crate) type WasmMessageBuffer<T> = OnceLock<ReentrantMutex<RefCell<Option<T>>>>;

macro_rules! get_buffer {
    ($buffer:ident) => {
        $buffer.get_or_init(|| {
            ::parking_lot::ReentrantMutex::new(::std::cell::RefCell::new(
                ::core::option::Option::None,
            ))
        })
    };
}

macro_rules! write_message {
    ($buffer:ident, $expr:expr) => {{
        let buffer = crate::wasm::get_buffer!($buffer);
        if let Some(lock) = buffer.try_lock() {
            *lock.borrow_mut() = Some($expr);
        }
    }};
}

macro_rules! read_buffer {
    ($buffer:ident, |$msg:ident| $stmt:stmt) => {{
        let buffer = crate::wasm::get_buffer!($buffer);
        if let Some(lock) = buffer.try_lock() {
            if let Some($msg) = lock.borrow_mut().take() {
                $stmt
            }
        }
    }};
}

pub(crate) use get_buffer;
pub(crate) use read_buffer;
pub(crate) use write_message;
