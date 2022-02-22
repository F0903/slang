use crate::defs::{Argument, Function, Variable};
use crate::expression::{Expression, ExpressionContext};
use crate::token::{get_tokens, Token};
use crate::value::Value;
use std::io::BufRead;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn is_char_var_delimiter(ch: char) -> bool {
    ch == ' ' || ch == '='
}

pub struct ParseResult {
    pub vars: Vec<Variable>,
    pub funcs: Vec<Function>,
}

pub struct Parser {
    vars: Vec<Variable>,
    funcs: Vec<Function>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            vars: vec![],
            funcs: vec![],
        }
    }

    fn parse_var_name(line_iter: &mut dyn Iterator<Item = char>) -> String {
        let mut name_buf = String::default();
        for ch in line_iter {
            if is_char_var_delimiter(ch) {
                break;
            }

            // Remove extra spaces at the start.
            if ch == ' ' {
                continue;
            }

            name_buf.push(ch);
        }
        name_buf
    }

    fn parse_var_value(&self, line_iter: &mut dyn Iterator<Item = char>) -> Result<Value> {
        let mut expression_text = None;
        for ch in line_iter {
            if ch != '=' && expression_text.is_none() {
                continue;
            } else if ch == '=' {
                expression_text = Some(String::default());
                continue;
            }

            if ch == ' ' {
                continue;
            }

            if let Some(expr) = &mut expression_text {
                expr.push(ch);
            }
        }

        match expression_text {
            None => Ok(Value::None),
            Some(expr) => Expression::from_str(expr).evaluate_statically(ExpressionContext {
                vars: self.vars.clone(),
                funcs: self.funcs.clone(),
            }),
        }
    }

    fn parse_var(&self, line_iter: &mut dyn Iterator<Item = char>) -> Result<Variable> {
        let name = Self::parse_var_name(line_iter);
        let value = self.parse_var_value(line_iter)?;
        Ok(Variable { name, value })
    }

    fn parse_func_name(line_iter: &mut dyn Iterator<Item = char>) -> String {
        let mut name_buf = String::default();
        let mut last_char = '0';
        for ch in line_iter {
            if last_char.is_alphabetic() && (ch == ' ' || ch == '(') {
                break;
            }

            // Remove extra spaces at the start.
            if ch == ' ' {
                continue;
            }

            name_buf.push(ch);
            last_char = ch;
        }
        name_buf
    }

    fn parse_func_args(line_iter: &mut dyn Iterator<Item = char>) -> Result<Vec<Argument>> {
        let mut args = vec![];
        let mut arg_name_buf = String::default();
        for ch in line_iter {
            if ch == ')' || ch == ',' {
                args.push(Argument {
                    name: arg_name_buf.clone(),
                    value: Value::Any,
                });
                arg_name_buf.clear();
                if ch == ',' {
                    continue;
                }
                break;
            }

            if ch == ' ' {
                continue;
            }

            if ch == '(' {
                continue;
            }

            arg_name_buf.push(ch);
        }
        Ok(args)
    }

    fn parse_func(line_iter: &mut dyn Iterator<Item = char>) -> Result<Function> {
        let name = Self::parse_func_name(line_iter);
        let args = Self::parse_func_args(line_iter)?;
        Ok(Function {
            name,
            args,
            ret_val: Value::Any,
        })
    }

    fn parse_line(&mut self, line: &str) -> Result<()> {
        // Exit early if line is just space.
        if line == "\r\n" || line == "\n" {
            return Ok(());
        }

        let tokens = get_tokens(line);

        for token in tokens {
            let mut line_enumerator = line.chars().skip(token.index);
            // Forward enumerator if chars are spaces.
            for ch in line_enumerator.by_ref() {
                if ch != ' ' {
                    continue;
                }
                // If comment, return.
                if ch == '?' {
                    return Ok(());
                }
                break;
            }

            match token.token {
                Token::Variable(_) => {
                    let var = self.parse_var(&mut line_enumerator)?;
                    println!("{:?}", var);
                    self.vars.push(var);
                }
                Token::Function(_) => {
                    let func = Self::parse_func(&mut line_enumerator)?;
                    println!("{:?}", func);
                    self.funcs.push(func);
                }
            };
        }
        Ok(())
    }

    pub fn parse(&mut self, input: impl BufRead) -> Result<ParseResult> {
        for line_result in input.lines() {
            let line = match line_result {
                Ok(x) => x,
                Err(_) => continue,
            };
            self.parse_line(&line)?;
        }
        let vars = self.vars.clone();
        self.vars.clear();
        let funcs = self.funcs.clone();
        self.funcs.clear();
        Ok(ParseResult { vars, funcs })
    }
}
