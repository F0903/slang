use super::sub_expression::SubExpression;
use crate::identifiable::Identifiable;
use crate::operators::{self, Operation};
use crate::value::Value;
use crate::vm::ExecutionContext;

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

    fn get_value_from_expr_token(&self, expr_token: &str) -> Result<Value> {
        let var = self.context.get_var(expr_token);
        let value = match var {
            Some(x) => x.borrow().get_value(),
            None => Value::from_string(expr_token)?,
        };
        Ok(value)
    }

    fn get_op(token_buf: &mut String) -> Option<&'a Operation> {
        if token_buf.is_empty() {
            return None;
        }
        let token_buf_copy = token_buf.clone();
        let mut best_op = None;
        let mut best_match_count = 0;
        let mut best_start_index = 0;
        let mut best_end_index = 0;
        for op in operators::OPERATORS {
            let op_id = op.get_identifier();
            let mut match_count = 0;
            let mut match_index = 0;
            for op_ch in op_id.chars() {
                for (i, tkn_ch) in token_buf_copy.chars().enumerate() {
                    if tkn_ch != op_ch {
                        continue;
                    }

                    if i > match_index {
                        match_count += 1;
                        match_index = i;
                    }

                    if i < best_start_index {
                        best_start_index = i;
                    }
                    if i > best_end_index {
                        best_end_index = i;
                    }

                    if match_count == op_id.len() && match_count > best_match_count {
                        best_op = Some(op);
                        best_match_count = match_count;
                    }
                }
            }
        }
        if let Some(op) = best_op {
            for _ in 0..op.get_identifier().len() {
                token_buf.pop();
            }
        }
        best_op
    }

    fn parse_expr(self) -> Result<Value> {
        let expression_str = &self.expr_string;

        let mut token_values = vec![];
        let mut ops = vec![];
        let mut token_buf = String::default();
        for ch in expression_str.chars() {
            if !ch.is_alphabetic() && !ch.is_numeric() {
                continue;
            }

            if let Some(op) = Self::get_op(&mut token_buf) {
                let value = self.get_value_from_expr_token(token_buf.trim_end())?;
                token_values.push(value);
                token_buf.clear();
                ops.push(Some(op));
            }

            token_buf.push(ch);
        }

        if !token_buf.is_empty() {
            let value = self.get_value_from_expr_token(token_buf.trim_start())?;
            token_values.push(value);
            token_buf.clear();
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
