use crate::defs::{Function, Variable};
use crate::identifiable::Identifiable;
use crate::operators::{self, OpPriority, Operation};
use crate::value::Value;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug)]
struct SubExpression {
    value: Value,
    op: Operation,
    next: Option<Box<Self>>,
}

impl SubExpression {
    fn remove_next_from_chain(&mut self) {
        let next: SubExpression;
        {
            let next_temp = match &self.next {
                None => return,
                Some(x) => x,
            };
            next = (**next_temp).clone();
        }

        let new_next = match next.next {
            None => {
                self.next = None;
                return;
            }
            Some(x) => x,
        };
        self.next = Some(new_next);
    }

    pub fn evaluate(&mut self) -> Result<Value> {
        let next = match &mut self.next {
            None => return Ok(self.value.clone()),
            Some(x) => x,
        };

        let my_priority = self.op.get_op_priority();
        let next_priority = next.op.get_op_priority();

        let mut set_plus_op = false;
        let next_value;
        if my_priority < next_priority {
            next_value = next.evaluate()?;
        } else {
            next_value = next.value.clone();
            if matches!(next.op, Operation::Minus(_)) && matches!(self.op, Operation::Plus(_)) {
                self.op = operators::MINUS;
            }
            set_plus_op = true;
        }

        self.value = self.value.perform_op(&self.op, &next_value)?;
        self.remove_next_from_chain();

        if set_plus_op {
            self.op = operators::PLUS
        }

        self.evaluate()
    }
}

pub struct ExpressionContext {
    pub vars: Vec<Variable>,
    pub funcs: Vec<Function>,
}

impl From<&dyn crate::vm::VmContext> for ExpressionContext {
    fn from(from: &dyn crate::vm::VmContext) -> Self {
        ExpressionContext {
            vars: from.get_vars(),
            funcs: from.get_funcs(),
        }
    }
}

pub struct Expression {
    expr_string: String,
    context: ExpressionContext,
}

impl Expression {
    pub fn from_str(expr: impl ToString, context: impl Into<ExpressionContext>) -> Expression {
        Expression {
            expr_string: expr.to_string(),
            context: context.into(),
        }
    }

    fn get_value_from_expr_token(&self, expr_token: &str) -> Result<Value> {
        let vars = &self.context.vars;
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
        for op in operators::OPERATORS {
            if op.get_identifier().chars().all(|x| x == ch) {
                return Some(op);
            }
        }
        None
    }

    /// NOTE: Expects no spacing in input
    fn parse_expr(self) -> Result<Value> {
        let expression_str = &self.expr_string;
        let mut token_values = vec![];
        let mut ops = vec![];
        let mut token_buf = String::default();

        for ch in expression_str.chars() {
            if let Some(op) = Self::is_char_operator(ch) {
                let value = self.get_value_from_expr_token(&token_buf)?;
                token_values.push(value);
                token_buf.clear();
                ops.push(Some(op));
                continue;
            }
            token_buf.push(ch);
        }

        if !token_buf.is_empty() {
            let value = self.get_value_from_expr_token(&token_buf)?;
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
            None => return Err("No start node found!".into()),
            Some(x) => x,
        };
        let val = start_node.evaluate()?;

        Ok(val)
    }

    pub fn evaluate(self) -> Result<Value> {
        self.parse_expr()
    }
}
