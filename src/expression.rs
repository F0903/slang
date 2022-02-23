use crate::defs::{Function, Variable};
use crate::identifiable::Identifiable;
use crate::operators::{Operation, OPERATORS};
use crate::value::Value;
use crate::vm::VmContext;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct ExpressionContext {
    pub vars: Vec<Variable>,
    pub funcs: Vec<Function>,
}

impl From<&dyn VmContext> for ExpressionContext {
    fn from(ctx: &dyn VmContext) -> Self {
        ExpressionContext {
            vars: ctx.get_vars(),
            funcs: ctx.get_funcs(),
        }
    }
}

pub struct Expression {
    expr_string: String,
}

impl Expression {
    pub fn from_str(expr: impl ToString) -> Expression {
        Expression {
            expr_string: expr.to_string(),
        }
    }

    fn get_value_from_expr_token(ctx: &ExpressionContext, expr_token: &str) -> Result<Value> {
        let vars = &ctx.vars;
        let mut token_chars = expr_token.chars();
        let value = if token_chars.all(|ch| ch.is_numeric()) || token_chars.any(|ch| ch == '"') {
            // If here then first token is a constant.
            Value::from_string(expr_token)?
        } else {
            // If here then first token is a variable.
            vars.iter()
                .find(|x| x.name == expr_token)
                .ok_or("Could not find varible.")?
                .value
                .clone()
        };
        Ok(value)
    }

    fn is_char_operator<'a>(ch: char) -> Option<&'a Operation> {
        for op in OPERATORS {
            if op.get_identifier().chars().all(|x| x == ch) {
                return Some(op);
            }
        }
        None
    }

    /// NOTE: Expects no spacing in input
    fn parse_expr(self, ctx: ExpressionContext) -> Result<Value> {
        let expression_str = self.expr_string;
        let mut token_values = vec![];
        let mut ops = vec![];
        let mut token_buf = String::default();

        for ch in expression_str.chars() {
            if let Some(op) = Self::is_char_operator(ch) {
                let value = Self::get_value_from_expr_token(&ctx, &token_buf)?;
                token_values.push(value);
                token_buf.clear();
                ops.push(op);
                continue;
            }
            token_buf.push(ch);
        }

        if !token_buf.is_empty() {
            let value = Self::get_value_from_expr_token(&ctx, &token_buf)?;
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

        let mut ops_iter = ops.iter();
        let mut token_iter = token_values.iter();

        let first_token = token_iter.next().unwrap();
        let first_op = ops_iter.next().unwrap();
        let second_token = token_iter.next().unwrap();

        let token_ops = token_iter.zip(ops_iter);

        //TODO: Respect math order of operations.
        let mut sum: Value = first_token.perform_op(first_op, second_token)?;
        for (value, op) in token_ops {
            sum = sum.perform_op(op, value)?;
        }

        Ok(sum)
    }

    pub fn evaluate(self, ctx: ExpressionContext) -> Result<Value> {
        self.parse_expr(ctx)
    }
}
