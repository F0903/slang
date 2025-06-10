mod as_object_ptr;
mod closure;
mod function;
mod native_function;
mod object;
mod object_ref;
mod string;
mod upvalue;

pub use as_object_ptr::AsObjectPtr;
pub use closure::Closure;
pub use function::Function;
pub use native_function::NativeFunction;
pub(crate) use object::ObjectUnion;
pub use object::{Object, ObjectType};
pub use object_ref::ObjectRef;
pub use string::{InternedString, StringInterner};
pub use upvalue::Upvalue;
