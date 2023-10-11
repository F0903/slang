use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::{Debug, Display},
    rc::Rc,
};

mod callable;
mod class;
mod function;
mod instance;

pub use {
    callable::{Callable, CallableResult},
    class::Class,
    function::*,
    instance::Instance,
};

#[derive(Debug, Clone)]
pub struct SharedPtr<T: ?Sized> {
    ptr: Rc<RefCell<T>>,
}

impl<T> SharedPtr<T> {
    pub fn new(val: T) -> Self {
        Self {
            ptr: Rc::new(RefCell::new(val)),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.ptr.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.ptr.borrow_mut()
    }
}

impl<'a, T> Callable<'a> for SharedPtr<T>
where
    T: Callable<'a>,
{
    fn call(
        &mut self,
        interpreter: &mut crate::interpreter::Interpreter,
        args: Vec<Value>,
    ) -> CallableResult {
        self.ptr.borrow_mut().call(interpreter, args)
    }

    fn get_arity(&self) -> usize {
        self.ptr.borrow().get_arity()
    }

    fn get_name(&self) -> String {
        self.ptr.borrow().get_name()
    }
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    NativeFunction(SharedPtr<NativeFunction>),
    Function(SharedPtr<Function>),
    Class(Class),
    Instance(SharedPtr<Instance>),
    None,
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::String(x) => Self::String(x.clone()),
            Self::Number(x) => Self::Number(x.clone()),
            Self::Boolean(x) => Self::Boolean(x.clone()),
            Self::Function(x) => Self::Function(x.clone()),
            Self::NativeFunction(x) => Self::NativeFunction(x.clone()),
            Self::Class(x) => Self::Class(x.clone()),
            Self::Instance(x) => Self::Instance(x.clone()),
            Self::None => Self::None,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(x) => f.write_fmt(format_args!("{x}")),
            Value::Number(x) => f.write_fmt(format_args!("{x}")),
            Value::Boolean(x) => f.write_fmt(format_args!("{x}")),
            Value::NativeFunction(x) => {
                f.write_fmt(format_args!("native function {}", x.borrow().get_name()))
            }
            Value::Function(x) => f.write_fmt(format_args!("function {}", x.borrow().get_name())),
            Value::Class(x) => Display::fmt(x, f),
            Value::Instance(x) => Display::fmt(&x.borrow().clone(), f),
            Value::None => f.write_str("none"),
        }
    }
}
