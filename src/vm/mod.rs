mod context;
mod native_func;
mod vm;

use crate::types::NamedValue;
use std::{cell::RefCell, rc::Rc};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type NamedVal = Rc<RefCell<dyn NamedValue>>;

pub use context::{Contextable, ExecutionContext, VmContext};
pub use native_func::{Function, NativeFunction};
pub use vm::VirtualMachine;
