pub mod object;
mod value;
mod value_casts;
mod value_type;

use object::{Object, ObjectNode};
pub use value::Value;
use value_casts::*;
use value_type::*;
