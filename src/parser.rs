use crate::defs::{Argument, Function, FunctionBody, Variable};
use crate::expression::{Expression, ExpressionContext};
use crate::token::{Token, TokenInfo, TokenInstance, TOKENS};
use crate::value::Value;
use std::io::BufRead;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

    fn parse_var_name(line_iter: &mut dyn Iterator<Item = char>) -> Result<String> {
        let mut name_buf = String::default();
        let mut encountered_name = false;
        for ch in line_iter {
            println!("{}", ch);

            if ch == ' ' {
                if encountered_name {
                    break;
                }
                continue;
            }

            if ch == '=' {
                break;
            }

            if ch == '?' {
                break;
            }

            encountered_name = true;
            name_buf.push(ch);
        }

        if crate::token::TOKENS.iter().any(|t| {
            let beg = t.get_start_token();
            let end = t.get_end_token();
            beg.contains(&name_buf) || end.contains(&name_buf)
        }) {
            return Err(format!(
                "Variable identifier '{}' is illegal. Identifiers cannot contain keywords!",
                name_buf
            )
            .into());
        }

        if !Self::is_legal_identifier(name_buf.chars()) {
            return Err(format!("Variable identifier '{}' is illegal.", name_buf).into());
        }
        Ok(name_buf)
    }

    fn parse_var_value(&self, line_iter: &mut dyn Iterator<Item = char>) -> Result<Value> {
        let mut expression_text = String::default();
        for ch in line_iter {
            println!("{}", ch);
            if ch == ' ' || ch == '=' {
                continue;
            }

            if ch == '?' {
                break;
            }

            expression_text.push(ch);
        }

        let val = if !expression_text.is_empty() {
            Expression::from_str(
                expression_text,
                ExpressionContext {
                    vars: self.vars.clone(),
                    funcs: self.funcs.clone(),
                },
            )
            .evaluate()?
        } else {
            Value::None
        };
        Ok(val)
    }

    fn parse_var(&self, line_iter: &mut dyn Iterator<Item = char>) -> Result<Variable> {
        let name = Self::parse_var_name(line_iter)?;
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

    fn parse_func_signature(
        line_iter: &mut impl Iterator<Item = char>,
    ) -> Result<(String, Vec<Argument>)> {
        let name = Self::parse_func_name(line_iter);
        let args = Self::parse_func_args(line_iter)?;
        Ok((name, args))
    }

    fn parse_local_scope(line_iter: &mut impl Iterator<Item = char>) -> Result<Vec<Variable>> {
        let vars = vec![];
        Ok(vars)
    }

    fn parse_func_body(line_iter: &mut impl Iterator<Item = char>) -> Result<FunctionBody> {
        let local_vars = Self::parse_local_scope(line_iter)?;
        Ok(FunctionBody { vars: local_vars })
    }

    fn parse_func(line_iter: &mut impl Iterator<Item = char>) -> Result<Function> {
        let (name, args) = Self::parse_func_signature(line_iter)?;
        let body = Self::parse_func_body(line_iter)?;
        Ok(Function {
            name,
            args,
            body,
            ret_val: Value::Any,
        })
    }

    fn is_legal_identifier(
        chars: impl IntoIterator<Item = impl std::borrow::Borrow<char>>,
    ) -> bool {
        chars.into_iter().all(|ch| {
            let ch = ch.borrow();
            ch.is_alphabetic() || ch.is_numeric() || *ch == '_' || *ch == '-'
        })
    }

    fn parse_tokens(line: &str) -> Vec<TokenInstance> {
        let mut token_buf = vec![];
        'token_loop: for token in TOKENS {
            let beg = token.get_start_token();
            let mut first_match_index = 0;
            let mut match_count = 0;
            for (i, (ch, beg_ch)) in line.chars().zip(beg.chars()).enumerate() {
                if ch == '?' {
                    break 'token_loop;
                }

                if ch == ' ' {
                    continue;
                }

                if ch != beg_ch {
                    break;
                }

                if match_count == 0 {
                    first_match_index = i;
                }
                match_count += 1;

                if match_count == beg.len() {
                    token_buf.push(TokenInstance {
                        start_index: first_match_index,
                        end_index: i + 1,
                        token,
                    });
                    break 'token_loop;
                }
            }
        }
        token_buf
    }

    fn parse_global_space(&mut self, input: impl BufRead) -> Result<()> {
        for line in input.lines() {
            let line = match line {
                Ok(x) => x,
                Err(x) => return Err(x.into()),
            };
            let tokens = Self::parse_tokens(&line);

            for token in tokens {
                let TokenInstance {
                    token,
                    start_index,
                    end_index,
                } = token;

                let line_chars = &mut line.chars().skip(start_index + end_index);

                match token {
                    Token::Variable(_) => self.vars.push(self.parse_var(line_chars)?),
                    Token::Function(_) => self.funcs.push(Self::parse_func(line_chars)?),
                };
            }
        }
        Ok(())
    }

    pub fn parse(&mut self, input: impl BufRead) -> Result<ParseResult> {
        self.parse_global_space(input)?;

        let vars = self.vars.clone();
        self.vars.clear();
        let funcs = self.funcs.clone();
        self.funcs.clear();

        Ok(ParseResult { vars, funcs })
    }
}
