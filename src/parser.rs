use crate::defs::{Argument, Function, FunctionBody, Variable};
use crate::expression::{Expression, ExpressionContext};
use crate::identifiable::Identifiable;
use crate::keyword::{Keyword, KeywordInfo, KEYWORDS};
use crate::operators::OPERATORS;
use crate::util::LINE_ENDING;
use crate::value::Value;
use crate::vm::Vm;
use std::io::BufRead;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct ParseResult {
    pub vars: Vec<Variable>,
    pub funcs: Vec<Function>,
}

pub struct Parser<R: BufRead> {
    input: R,
}

impl<'a, R: BufRead> Parser<R> {
    pub fn new(input: R) -> Self {
        Self { input }
    }

    fn is_char_legal_identifier(ch: impl std::borrow::Borrow<char>) -> bool {
        let ch = ch.borrow();
        ch.is_alphabetic() || ch.is_numeric() || *ch == '_' || *ch == '-'
    }

    fn parse_func_name(line: &mut impl Iterator<Item = char>) -> String {
        //Skip the included keyword
        for ch in line.by_ref() {
            if ch == ' ' {
                break;
            }
        }

        let mut name_buf = String::default();
        let mut last_char = '0';
        for ch in line.by_ref() {
            if last_char.is_alphabetic() && (ch == ' ' || ch == '(') {
                break;
            }

            // Remove extra spaces at the start.
            if ch == ' ' {
                continue;
            }

            if !Self::is_char_legal_identifier(ch) {
                continue;
            }

            name_buf.push(ch);
            last_char = ch;
        }

        name_buf
    }

    fn parse_func_args(&mut self, keyword_line: impl ToString) -> Result<Vec<Argument>> {
        let line_iter = [Ok(keyword_line.to_string())]
            .into_iter()
            .chain(self.input.by_ref().lines());

        let mut args = vec![];
        let mut arg_name_buf = String::default();
        'line_iter: for line in line_iter {
            let line = match line {
                Ok(x) => x,
                Err(x) => return Err(x.into()),
            };
            for ch in line.chars() {
                if ch == ')' || ch == ',' {
                    args.push(Argument {
                        name: arg_name_buf.clone(),
                        value: Value::Any,
                    });
                    arg_name_buf.clear();
                    if ch == ',' {
                        continue;
                    }
                    if ch == ')' {
                        break 'line_iter;
                    }
                    break;
                }

                if ch == ' ' {
                    continue;
                }

                if ch == '(' {
                    continue;
                }

                if !Self::is_char_legal_identifier(ch) {
                    continue;
                }

                arg_name_buf.push(ch);
            }
        }

        Ok(args)
    }

    fn parse_func_signature(&mut self, keyword_line: &str) -> Result<(String, Vec<Argument>)> {
        let mut keyword_line = keyword_line.chars();
        let name = Self::parse_func_name(&mut keyword_line);
        let keyword_line = keyword_line.collect::<String>();

        let args = self.parse_func_args(keyword_line)?;
        Ok((name, args))
    }

    fn parse_local_scope(&mut self) -> Result<Vec<Variable>> {
        let vars = vec![];
        Ok(vars)
    }

    fn parse_func_body(&mut self) -> Result<FunctionBody> {
        let local_vars = self.parse_local_scope()?;
        Ok(FunctionBody { vars: local_vars })
    }

    // Find alternative instead of the keyword line arg?
    fn parse_func(&mut self, keyword_line: &str, vm: &'a mut dyn Vm) -> Result<()> {
        let (name, args) = self.parse_func_signature(keyword_line)?;
        let body = self.parse_func_body()?;
        let func = Function {
            name,
            args,
            body,
            ret_val: Value::Any,
        };
        vm.register_func(func);
        Ok(())
    }

    fn parse_var_name(line: impl IntoIterator<Item = char>) -> Result<String> {
        let mut name_buf = String::default();
        let mut encountered_name = false;
        for ch in line {
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

            if !Self::is_char_legal_identifier(ch) {
                continue;
            }

            encountered_name = true;
            name_buf.push(ch);
        }

        if KEYWORDS.iter().any(|x| x.get_keyword().contains(&name_buf)) {
            return Err(format!(
                "Variable identifier '{}' is illegal. Identifiers cannot contain keywords!",
                name_buf
            )
            .into());
        }

        Ok(name_buf)
    }

    fn parse_var_value(line: impl IntoIterator<Item = char>, vm: &'a dyn Vm) -> Result<Value> {
        let mut expression_text = String::default();
        for ch in line {
            println!("{}", ch);
            if ch == '?' {
                break;
            }

            if LINE_ENDING.chars().any(|le| le == ch) {
                break;
            }

            if ch == '=' || ch == ' ' {
                continue;
            }

            if ch != '"'
                && ch != '_'
                && !ch.is_alphabetic()
                && !ch.is_numeric()
                && !OPERATORS
                    .iter()
                    .any(|op| op.get_identifier().chars().next().unwrap() == ch)
            {
                continue;
            }

            expression_text.push(ch);
        }

        let val = if !expression_text.is_empty() {
            Expression::from_str(expression_text, vm.get_context()).evaluate()?
        } else {
            Value::None
        };
        Ok(val)
    }

    fn parse_var(line: &str, vm: &'a mut dyn Vm) -> Result<()> {
        let mut chars = line.chars();

        // Skip the included keyword.
        for ch in chars.by_ref() {
            if ch == ' ' {
                break;
            }
        }

        let name = Self::parse_var_name(&mut chars)?;
        let value = Self::parse_var_value(&mut chars, vm)?;

        let var = Variable { name, value };
        vm.register_var(var);
        Ok(())
    }

    fn get_keyword(line: &str) -> Option<&Keyword> {
        for keyword in KEYWORDS {
            let keyword_str = keyword.get_keyword();
            let keyword_line_iter = line.chars().zip(keyword_str.chars());
            let mut match_count = 0;
            for (ch, keyword_ch) in keyword_line_iter {
                if ch == ' ' {
                    continue;
                }

                if ch == '?' {
                    break;
                }

                if ch != keyword_ch {
                    break;
                }

                match_count += 1;
                if match_count == keyword_str.len() {
                    return Some(keyword);
                }
            }
        }
        None
    }

    pub fn parse(&mut self, vm: &'a mut dyn Vm) -> Result<()> {
        loop {
            let mut line = String::default();
            let count = self.input.read_line(&mut line)?;
            if count == 0 {
                break;
            }
            if line.is_empty() {
                continue;
            }

            let keyword = match Self::get_keyword(&line) {
                Some(x) => x,
                None => continue,
            };

            match keyword {
                Keyword::Variable(_) => Self::parse_var(&line, vm)?,
                Keyword::Function(_) => self.parse_func(&line, vm)?,
                Keyword::ScopeEnd(_) => return Err("Invalid structured program! Cannot encounter a scope end before a scope is declared.".into()),
            }
        }

        Ok(())
    }
}
