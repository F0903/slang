use crate::defs::{Function, FunctionBody, Parameter, Variable};
use crate::expressions::Expression;
use crate::identifiable::Identifiable;
use crate::keyword::{Keyword, KeywordInfo, KEYWORDS};
use crate::operators::OPERATORS;
use crate::value::{Argument, Value};
use crate::vm::{Contextable, ExecutionContext, VirtualMachine};
use std::cell::RefCell;
use std::io::BufRead;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

//TODO: Refactor and seperate into files?

pub struct Parser {}

impl Parser {
    fn is_char_legal_literal(ch: impl std::borrow::Borrow<char>) -> bool {
        let ch = ch.borrow();
        ch.is_alphabetic() || ch.is_numeric() || *ch == '"'
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

    fn parse_func_params(
        mut input: impl BufRead,
        keyword_line: impl ToString,
    ) -> Result<Vec<Parameter>> {
        let line_iter = [Ok(keyword_line.to_string())]
            .into_iter()
            .chain(input.by_ref().lines());

        let mut params = vec![];
        let mut param_idx = 0;
        let mut param_name_buf = String::default();
        'line_iter: for line in line_iter {
            let line = match line {
                Ok(x) => x,
                Err(x) => return Err(x.into()),
            };
            for ch in line.chars() {
                if ch == ')' || ch == ',' {
                    params.push(Parameter {
                        index: param_idx,
                        name: param_name_buf.clone(),
                        value: Value::Any,
                    });
                    param_name_buf.clear();
                    if ch == ',' {
                        param_idx += 1;
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

                param_name_buf.push(ch);
            }
        }

        Ok(params)
    }

    fn parse_func_signature(
        input: impl BufRead,
        keyword_line: impl AsRef<str>,
    ) -> Result<(String, Vec<Parameter>)> {
        let mut keyword_line = keyword_line.as_ref().chars();
        let name = Self::parse_func_name(&mut keyword_line);
        let keyword_line = keyword_line.collect::<String>();

        let params = Self::parse_func_params(input, keyword_line)?;
        Ok((name, params))
    }

    fn parse_func_body(mut input: impl BufRead) -> Result<FunctionBody> {
        // Assume that the function signature is not included
        let mut code = String::default();
        'line_loop: loop {
            let mut line = String::default();
            input.read_line(&mut line)?;
            let mut encountered_letter = false;
            for ch in line.chars() {
                if !encountered_letter && ch == ' ' {
                    continue;
                } else if ch.is_alphabetic() {
                    encountered_letter = true;
                }

                code.push(ch);
                if code.len() >= 3 && &code[code.len() - 3..code.len()] == "end" {
                    code.pop();
                    code.pop();
                    code.pop();
                    break 'line_loop;
                }
            }
        }
        Ok(FunctionBody { code })
    }

    // Find alternative instead of the keyword line arg?
    fn parse_func(mut input: impl BufRead, keyword_line: impl AsRef<str>) -> Result<Function> {
        let (name, params) = Self::parse_func_signature(input.by_ref(), keyword_line)?;
        let body = Self::parse_func_body(input)?;
        let func = Function {
            name,
            params,
            body,
            ret_val: Value::Any,
        };
        Ok(func)
    }

    fn parse_var_name(line: impl IntoIterator<Item = char>) -> Result<String> {
        let mut name_buf = String::default();
        let mut encountered_name = false;
        for ch in line {
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

    fn parse_var_value(
        line: impl IntoIterator<Item = char>,
        expr_context: &impl ExecutionContext,
    ) -> Result<Value> {
        let mut expression_text = String::default();
        for ch in line {
            if ch == '?' {
                break;
            }

            if ch == '\n' || ch == '\r' {
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
            Expression::from_str(expression_text, expr_context).evaluate()?
        } else {
            Value::None
        };
        Ok(val)
    }

    fn parse_var(line: impl AsRef<str>, expr_context: &impl ExecutionContext) -> Result<Variable> {
        let mut chars = line.as_ref().chars();

        // Skip the included keyword.
        for ch in chars.by_ref() {
            if ch == ' ' {
                break;
            }
        }

        let name = Self::parse_var_name(&mut chars)?;
        let value = Self::parse_var_value(&mut chars, expr_context)?;

        let var = Variable { name, value };
        Ok(var)
    }

    fn get_keyword(line: impl AsRef<str>) -> Option<&'static Keyword> {
        for keyword in KEYWORDS {
            let keyword_str = keyword.get_keyword();
            let keyword_line_iter = line.as_ref().chars().zip(keyword_str.chars());
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

    fn check_line_statement(line: impl AsRef<str>, vm: &mut VirtualMachine) -> Result<()> {
        let line = line.as_ref();
        if line.is_empty()
            || (!line.is_empty() && &line[0..1] == "\n")
            || (line.len() >= 2 && &line[0..2] == "\r\n")
        {
            return Ok(());
        }

        let ctx = vm.get_context();
        let mut name_buf = String::default();
        let mut vals_str_buf = vec![];
        let mut val_buf = String::default();
        let mut value_encountered = false;
        for ch in line.chars() {
            if value_encountered {
                if ch == ')' {
                    vals_str_buf.push(val_buf.clone());
                    break;
                }
                if ch == ',' {
                    vals_str_buf.push(val_buf.clone());
                    val_buf.clear();
                    continue;
                }
                if !Self::is_char_legal_literal(ch) {
                    continue;
                }
                val_buf.push(ch);
                continue;
            }

            if ch == '?' || ch == '\n' || ch == '\r' {
                break;
            }

            if ch == '=' || ch == '(' {
                value_encountered = true;
                continue;
            }

            if !Self::is_char_legal_identifier(ch) {
                continue;
            }
            name_buf.push(ch);
        }

        if ctx.contains_var(&name_buf) {
            ctx.set_var(&name_buf, Expression::from_str(val_buf, ctx).evaluate()?)?;
        } else if ctx.contains_func(&name_buf) {
            let mut vals_buf = vec![];
            for (i, val_str) in vals_str_buf.iter().enumerate() {
                let expr = Expression::from_str(val_str, ctx);
                let expr_value = expr.evaluate()?;
                vals_buf.push(Argument {
                    matched_name: None,
                    index: i,
                    value: expr_value,
                });
            }
            vm.call_func(name_buf, &mut vals_buf)?;
        }
        Ok(())
    }

    pub fn parse_func_code(
        code: impl AsRef<str>,
        args: &[Argument],
        vm: &mut VirtualMachine,
    ) -> Result<()> {
        let code = code.as_ref();
        let mut ctx = vm.get_context().clone();
        for arg in args {
            ctx.push_var(Rc::new(RefCell::new(arg.clone())));
        }
        for line in code.lines() {
            let keyword = match Self::get_keyword(line) {
                Some(x) => x,
                None => {
                    Self::check_line_statement(line, vm)?;
                    continue;
                }
            };

            match keyword {
                Keyword::Variable(_) => ctx.push_var(Rc::new(RefCell::new(Self::parse_var(line, &ctx)?))),
                Keyword::Function(_) => ctx.push_func(Self::parse_func(code.as_bytes(), line)?), // UNTESTED AND NOT EXPECTED OR SUPPOSED TO WORK
                Keyword::ScopeEnd(_) => return Err("Invalid structured program! Cannot encounter a scope end before a scope is declared.".into()),
            }
        }

        Ok(())
    }

    pub fn parse(mut input: impl BufRead, vm: &mut VirtualMachine) -> Result<()> {
        loop {
            let mut line = String::default();
            let count = input.by_ref().read_line(&mut line)?;
            if count == 0 {
                break;
            }
            if line.is_empty() {
                continue;
            }

            let keyword = match Self::get_keyword(&line) {
                Some(x) => x,
                None => {
                    Self::check_line_statement(line, vm)?;
                    continue;
                }
            };

            let ctx = vm.get_context();
            match keyword {
                Keyword::Variable(_) => ctx.push_var(Rc::new(RefCell::new(Self::parse_var(line, ctx)?))),
                Keyword::Function(_) => ctx.push_func(Self::parse_func(input.by_ref(), line)?),
                Keyword::ScopeEnd(_) => return Err("Invalid structured program! Cannot encounter a scope end before a scope is declared.".into()),
            }
        }

        Ok(())
    }
}
