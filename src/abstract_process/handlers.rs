use std::any::{type_name, TypeId};
use std::marker::PhantomData;

use super::messages::RequestMessage;
use super::{AbstractProcess, DeferredRequestHandler, MessageHandler, RequestHandler};
use crate::serializer::Serializer;
use crate::Tag;

pub struct Message<T>(PhantomData<T>);
pub struct Request<T>(PhantomData<T>);
pub struct DeferredRequest<T>(PhantomData<T>);

pub trait Handler<AP: AbstractProcess> {
    fn handle(response_tag: Tag, state: &mut AP::State);
}

impl<AP, T> Handler<AP> for Message<T>
where
    AP: MessageHandler<T>,
    AP::Serializer: Serializer<T>,
{
    fn handle(_: Tag, state: &mut <AP as AbstractProcess>::State) {
        let state = super::State { state };
        let message = AP::Serializer::decode().unwrap();
        AP::handle(state, message);
    }
}

impl<AP, T> Handler<AP> for Request<T>
where
    AP: RequestHandler<T>,
    AP::Serializer: Serializer<T>,
    AP::Serializer: Serializer<AP::Response>,
    AP::Serializer: Serializer<RequestMessage<T, AP::Response, AP::Serializer>>,
{
    fn handle(response_tag: Tag, state: &mut <AP as AbstractProcess>::State) {
        let state = super::State { state };
        let request: RequestMessage<T, AP::Response, AP::Serializer> =
            AP::Serializer::decode().unwrap();
        let response = AP::handle(state, request.0);
        request.1.send_response(response, response_tag);
    }
}

impl<AP, T> Handler<AP> for DeferredRequest<T>
where
    AP: DeferredRequestHandler<T>,
    AP::Serializer: Serializer<T>,
    AP::Serializer: Serializer<AP::Response>,
    AP::Serializer: Serializer<RequestMessage<T, AP::Response, AP::Serializer>>,
{
    fn handle(response_tag: Tag, state: &mut <AP as AbstractProcess>::State) {
        let state = super::State { state };
        let request: RequestMessage<T, AP::Response, AP::Serializer> =
            AP::Serializer::decode().unwrap();
        AP::handle(
            state,
            request.0,
            super::DeferredResponse {
                tag: response_tag,
                return_address: request.1,
            },
        );
    }
}

pub trait Handlers<AP: AbstractProcess> {
    fn handler_id<Handler: 'static>() -> u8;
    fn handle(response_tag: Tag, id: u8, state: &mut AP::State);
}

// Implement `Handlers` for tuple containing up to 16 handlers.
macros::impl_handlers!();
macros::impl_handlers!(T1 1);
macros::impl_handlers!(T1 1, T2 2);
macros::impl_handlers!(T1 1, T2 2, T3 3);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10, T11 11);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10, T11 11, T12 12);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10, T11 11, T12 12, T13 13);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10, T11 11, T12 12, T13 13, T14 14);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10, T11 11, T12 12, T13 13, T14 14, T15 15);
macros::impl_handlers!(T1 1, T2 2, T3 3, T4 4, T5 5, T6 6, T7 7, T8 8, T9 9, T10 10, T11 11, T12 12, T13 13, T14 14, T15 15, T16 16);

mod macros {
    macro_rules! impl_handlers {
        ($($args:ident $i:tt),*) => {
            impl<AP, $($args: 'static),*> Handlers<AP> for ($($args,)*)
            where
                AP: AbstractProcess,
                $($args: Handler<AP>),*
            {
                #[track_caller]
                fn handler_id<Handler: 'static>() -> u8 {
                    match TypeId::of::<Handler>() {
                        $(id if id == TypeId::of::<$args>() => $i,)*
                        _ => panic!(
                            "Called `send/request()` on type '{}' that doesn't match any handler defined in '<{} as AbstractProcess>::Handlers'",
                            type_name::<Handler>(),
                            type_name::<AP>()
                        ),
                    }
                }

                #[allow(unused_variables)]
                fn handle(response_tag: Tag, id: u8, state: &mut <AP as AbstractProcess>::State) {
                    match id {
                        // Handlers start with a value of 1. Zero indicates that this is a response from another
                        // process where the call timed out, and we don't care about the result.
                        0 => (),
                        $($i => $args::handle(response_tag, state),)*
                        _ => unreachable!(
                            "AbstractProcess `{}` received message with unknown message ID: {}.",
                            type_name::<AP>(),
                            id
                        ),
                    }
                }
            }
        };
    }

    pub(crate) use impl_handlers;
}
