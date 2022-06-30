use super::sub_expression::SubExpression;
use crate::operators::{self, Operation};
use crate::types::{Identifiable, Value};
use crate::util::window_iter::IntoWindowIter;
use crate::vm::ExecutionContext;
use std::ops::Deref;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Expression<'a> {
    expr_string: String,
    context: &'a dyn ExecutionContext,
}

impl<'a> Expression<'a> {
    pub fn from_str(expr: impl ToString, context: &'a dyn ExecutionContext) -> Expression {
        Expression {
            expr_string: expr.to_string(),
            context,
        }
    }

    fn get_value_from_expr_or_var(&self, expr_token: &str) -> Result<Value> {
        let var = self.context.get_var(expr_token);
        let value = match var {
            Some(x) => x.deref().borrow().get_value(),
            None => Value::from_string(expr_token)?,
        };
        Ok(value)
    }

    fn get_op(op_str: &str) -> Option<&'a Operation> {
        for op in operators::OPERATORS {
            if op.get_identifier() == op_str {
                return Some(op);
            }
        }
        None
    }

    fn parse_expr(self) -> Result<Value> {
        let expression_str = &self.expr_string.trim();
        if expression_str.starts_with('"') && expression_str.ends_with('"') {
            return Value::from_string(expression_str);
        }

        let mut ops = vec![];
        let tokens = expression_str.split(' ');
        let mut token_values = vec![];

        for token_pair in tokens.into_window_iter() {
            match token_pair {
                (Some(first), Some(second)) => {
                    let value = self.get_value_from_expr_or_var(first)?;
                    token_values.push(value);

                    let op = Self::get_op(second);
                    ops.push(op);
                }
                (Some(first), None) => {
                    let value = self.get_value_from_expr_or_var(first)?;
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
