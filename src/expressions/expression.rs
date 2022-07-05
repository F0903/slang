use std::fmt::format;
use std::ops::Deref;

use super::sub_expression::SubExpression;
use crate::operators::{self, Operation};
use crate::types::{Argument, Identifiable, Value};
use crate::util::window_iter::IntoWindowIter;
use crate::vm::{ExecutionContext, VirtualMachine};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Expression<'a> {
    expr_string: String,
    context: &'a ExecutionContext,
    vm: &'a VirtualMachine,
}

impl<'a> Expression<'a> {
    pub fn from_str(
        expr: impl ToString,
        context: &'a ExecutionContext,
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
        let arg_list = &expr_str[brace_start + 1..brace_end];
        let args = arg_list.split(',');
        let mut arg_values = vec![];
        for arg in args {
            if arg.is_empty() {
                continue;
            }
            let val = self.get_value_from_expr(&arg)?;
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

    fn get_array_name_and_indexing(expression_str: &str) -> Result<Option<(&str, usize)>> {
        let mut index_start = 0;
        let mut index_end = 0;
        let mut index_start_found = false;
        let mut index_end_found = false;
        for (i, ch) in expression_str.chars().enumerate() {
            if ch == '[' && !index_start_found {
                index_start = i;
                index_start_found = true;
            }
            if ch == ']' && !index_end_found {
                index_end = i;
                index_end_found = true;
            }
            if index_start_found && index_end_found {
                break;
            }
        }

        if !index_start_found && !index_end_found {
            return Ok(None);
        }

        let var_name = &expression_str[0..index_start];
        let arr_index = expression_str[index_start + 1..index_end].parse::<usize>()?;
        Ok(Some((var_name, arr_index)))
    }

    fn get_value_from_var_or_literal(&self, expr_str: &str) -> Result<Value> {
        let mut is_array = false;
        let mut arr_name = "";
        let mut arr_indexing = 0;
        if let Some((name, indexing)) = Self::get_array_name_and_indexing(expr_str)? {
            is_array = true;
            arr_name = name;
            arr_indexing = indexing;
        }
        if is_array {
            let array = match self.context.get_var(arr_name) {
                Some(x) => x.deref().borrow().get_value(),
                None => {
                    return Err(
                        format!("Could not find array variable with name '{arr_name}'").into(),
                    )
                }
            };
            let arr_val_result = match array {
                Value::Array(raw_vec) => raw_vec
                    .get(arr_indexing)
                    .cloned()
                    .ok_or("Invalid index!".into()),
                _ => Err("Can only index into arrays!".into()),
            };
            return arr_val_result;
        }

        let var_val = match self.context.get_var(expr_str) {
            Some(x) => x.deref().borrow().get_value(),
            None => Value::from_string(expr_str)?,
        };
        return Ok(var_val);
    }

    fn get_value_from_expr(&self, expr_str: &str) -> Result<Value> {
        let brace_start = expr_str.find('(');
        let brace_end = expr_str.rfind(')');
        let is_callable = brace_start.is_some() && brace_end.is_some();
        if is_callable {
            self.resolve_func_value(
                unsafe { brace_start.unwrap_unchecked() },
                unsafe { brace_end.unwrap_unchecked() },
                expr_str,
            )
        } else {
            self.get_value_from_var_or_literal(expr_str)
        }
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
        let assignment_indx = expression_str
            .find('=')
            .ok_or("Could not find variable assignment operator!")?;

        let name = expression_str[0..assignment_indx].trim();
        let expr = expression_str[assignment_indx + 1..].trim();
        self.context
            .set_var(name, self.get_value_from_expr(expr)?)?;
        Ok(())
    }

    fn is_expr_assignment(expr_str: &str) -> bool {
        for ch in expr_str.chars() {
            match ch {
                '=' => return true,
                '"' => break,
                _ => continue,
            }
        }
        false
    }

    fn handle_array_initializer(&self, expression_str: &str) -> Result<Value> {
        // Remove the '[' and ']'
        let expression_str = &expression_str[1..expression_str.len() - 1];

        let mut vals = vec![];
        for val_expr in expression_str.split(',') {
            let val = self.get_value_from_expr(val_expr.trim())?;
            vals.push(val);
        }
        Ok(Value::Array(vals))
    }

    fn parse_expr(self) -> Result<Value> {
        let expression_str = self.expr_string.trim();

        if expression_str.starts_with('?') {
            return Ok(Value::None);
        }

        if expression_str.starts_with('"') && expression_str.ends_with('"') {
            return Value::from_string(expression_str);
        }

        if expression_str.starts_with('[') && expression_str.ends_with(']') {
            return self.handle_array_initializer(expression_str);
        }

        if Self::is_expr_assignment(expression_str) {
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
