use std::ops::Deref;

use super::sub_expression::SubExpression;
use crate::operators::{self, Operation};
use crate::types::{Argument, Identifiable, Value};
use crate::util::window_iter::IntoWindowIter;
use crate::vm::{VirtualMachine, VmContext};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Expression<'a> {
    expr_string: String,
    context: &'a VmContext,
    vm: &'a VirtualMachine,
}

impl<'a> Expression<'a> {
    pub fn from_str(
        expr: impl ToString,
        context: &'a VmContext,
        vm: &'a VirtualMachine,
    ) -> Expression<'a> {
        Expression {
            expr_string: expr.to_string(),
            context,
            vm,
        }
    }

    fn resolve_func_value(
        &self,
        brace_start: usize,
        brace_end: usize,
        expr_str: &str,
    ) -> Result<Value> {
        let func_name = &expr_str[0..brace_start];
        let arg_list = &expr_str[brace_start..brace_end];
        let args = arg_list.split(',');
        let mut arg_values = vec![];
        for arg in args {
            let mut arg = arg.to_owned();
            arg.remove_matches(&['(', ')']);
            if arg.is_empty() {
                continue;
            }
            let val = Expression::from_str(arg, self.context, self.vm).evaluate()?;
            arg_values.push(val);
        }
        self.vm.call_func(
            func_name,
            arg_values
                .into_iter()
                .enumerate()
                .map(|(i, x)| Argument::new(i, x))
                .collect::<Vec<Argument>>()
                .as_mut_slice(),
        )
    }

    fn get_value_from_expr(&self, expr_str: &str) -> Result<Value> {
        let brace_start = expr_str.rfind('(');
        let brace_end = expr_str.rfind(')');
        let is_callable = brace_start.is_some() && brace_end.is_some();
        let val = if is_callable {
            self.resolve_func_value(
                unsafe { brace_start.unwrap_unchecked() },
                unsafe { brace_end.unwrap_unchecked() },
                expr_str,
            )?
        } else {
            match self.context.get_var(expr_str) {
                Some(x) => x.deref().borrow().get_value(),
                None => Value::from_string(expr_str)?,
            }
        };
        Ok(val)
    }

    fn get_op(op_str: &str) -> Option<&'a Operation> {
        for op in operators::OPERATORS {
            if op.get_identifier() == op_str {
                return Some(op);
            }
        }
        None
    }

    fn handle_var_assignment(&self, expression_str: &str) -> Result<()> {
        //offering a = 5
        let keyword_name_spacer = expression_str
            .find(' ')
            .ok_or("Could not find variable name start!")?;
        let assignment = expression_str
            .find('=')
            .ok_or("Could not find variable assignment operator!")?;

        let name = expression_str[keyword_name_spacer..assignment].trim();
        let expr = expression_str[assignment..].trim();
        self.context.set_var(
            name,
            Expression::from_str(expr, self.context, self.vm).evaluate()?,
        )?;
        Ok(())
    }

    fn parse_expr(self) -> Result<Value> {
        let expression_str = self.expr_string.trim();

        if expression_str.starts_with('?') {
            return Ok(Value::None);
        }

        if expression_str.starts_with('"') && expression_str.ends_with('"') {
            return Value::from_string(expression_str);
        }

        if expression_str.starts_with("offering") {
            self.handle_var_assignment(expression_str)?;
            return Ok(Value::None);
        }

        let mut ops = vec![];
        let tokens = expression_str.split(' ');
        let mut token_values = vec![];

        for token_pair in tokens.into_window_iter() {
            match token_pair {
                (Some(first), Some(second)) => {
                    let value = self.get_value_from_expr(first)?;
                    token_values.push(value);

                    let op = Self::get_op(second);
                    ops.push(op);
                }
                (Some(first), None) => {
                    let value = self.get_value_from_expr(first)?;
                    token_values.push(value);
                }
                _ => return Err("Expression has wrong format.".into()),
            }
        }

        if ops.is_empty() {
            if token_values.len() > 1 {
                return Err(
                    "No operators was found in expression, but more than one token was found!"
                        .into(),
                );
            }
            return Ok(token_values[0].clone());
        }

        ops.push(None);
        ops.reverse();
        let ops_iter = ops.iter();

        token_values.reverse();
        let value_iter = token_values.iter();

        let value_op_iter = value_iter.zip(ops_iter);

        let mut current_sub_expr;
        let mut next_sub_expr: Option<SubExpression> = None;
        for (value, op) in value_op_iter {
            current_sub_expr = SubExpression {
                value: value.clone(),
                op: match op {
                    None => operators::NOOP,
                    Some(x) => (*x).clone(),
                },
                next: next_sub_expr.map(Box::new),
            };
            next_sub_expr = Some(current_sub_expr);
        }

        let mut start_node = match next_sub_expr {
            None => return Err("No sub-expression start node found!".into()),
            Some(x) => x,
        };
        let val = start_node.evaluate()?;

        Ok(val)
    }

    pub fn evaluate(self) -> Result<Value> {
        self.parse_expr()
    }
}
