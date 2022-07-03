use crate::code_reader::CodeReader;
use crate::expressions::Expression;
use crate::keyword::{Keyword, KEYWORDS};
use crate::types::{Argument, Parameter, ScriptFunction, Value, Variable};
use crate::vm::{Function, VirtualMachine, VmContext};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

//TODO: Refactor

pub(crate) enum ScopeParseResult {
    None,
    Break,
    Return(Value),
}

pub struct Parser {}

impl Parser {
    fn is_char_legal_literal(ch: impl std::borrow::Borrow<char>) -> bool {
        let ch = ch.borrow();
        ch.is_alphabetic()
            || ch.is_numeric()
            || *ch == '"'
            || *ch == '='
            || *ch == '>'
            || *ch == '<'
    }

    fn is_char_legal_identifier(ch: impl std::borrow::Borrow<char>) -> bool {
        let ch = ch.borrow();
        ch.is_alphabetic() || ch.is_numeric() || *ch == '_' || *ch == '-'
    }

    fn read_func_name(line: &mut impl Iterator<Item = char>) -> String {
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

    fn read_func_params(lines: &CodeReader, keyword_line: impl ToString) -> Result<Vec<Parameter>> {
        let line_iter = [keyword_line.to_string()].into_iter().chain(lines);

        let mut params = vec![];
        let mut param_idx = 0;
        let mut param_name_buf = String::default();
        'line_iter: for line in line_iter {
            for ch in line.chars() {
                if ch == ')' || ch == ',' {
                    params.push(Parameter {
                        index: param_idx,
                        name: param_name_buf.clone(),
                        value: Value::None,
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

    fn read_func_signature(
        lines: &CodeReader,
        keyword_line: impl AsRef<str>,
    ) -> Result<(String, Vec<Parameter>)> {
        let mut keyword_line = keyword_line.as_ref().chars();
        let name = Self::read_func_name(&mut keyword_line);
        let keyword_line = keyword_line.collect::<String>();

        let params = Self::read_func_params(lines, keyword_line)?;
        Ok((name, params))
    }

    fn read_func_body(lines: &mut CodeReader) -> Result<String> {
        // Assume that the function signature is not included
        let mut code = String::default();
        let mut current_end_index = 0;
        let mut end_index_target = 1;
        for line in lines {
            let line = line.trim();
            match Self::get_keyword(&line) {
                Some(Keyword::Function | Keyword::IfScope | Keyword::RepeatScope) => {
                    end_index_target += 1
                }
                Some(Keyword::ScopeEnd) => current_end_index += 1,
                _ => (),
            };
            if current_end_index == end_index_target {
                break;
            }
            code.push_str(&line);
            code.push('\n');
        }
        Ok(code)
    }

    fn read_func(lines: &mut CodeReader, keyword_line: impl AsRef<str>) -> Result<ScriptFunction> {
        let (name, params) = Self::read_func_signature(lines, keyword_line)?;
        let body = Self::read_func_body(lines)?;
        let func = ScriptFunction {
            name,
            params,
            code: body,
            ret_val: Value::None,
        };
        Ok(func)
    }

    fn read_var_name(line: impl IntoIterator<Item = char>) -> Result<String> {
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

        if KEYWORDS
            .iter()
            .all(|(_kw, kw_str)| kw_str.contains(&name_buf))
        {
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
        expr_context: &VmContext,
        vm: &VirtualMachine,
    ) -> Result<Value> {
        let mut expression_text = String::default();
        for ch in line {
            if ch == '=' {
                continue;
            }

            if ch == '?' {
                break;
            }

            expression_text.push(ch);
        }

        let val = if !expression_text.is_empty() {
            Expression::from_str(expression_text.trim(), expr_context, vm).evaluate()?
        } else {
            Value::None
        };
        Ok(val)
    }

    fn parse_var(
        line: impl AsRef<str>,
        expr_context: &VmContext,
        vm: &VirtualMachine,
    ) -> Result<Variable> {
        let mut chars = line.as_ref().chars();

        // Skip the included keyword. (forward the iterator until a space is hit)
        for ch in chars.by_ref() {
            if ch == ' ' {
                break;
            }
        }

        let name = Self::read_var_name(&mut chars)?;
        let value = Self::parse_var_value(&mut chars, expr_context, vm)?;

        let var = Variable { name, value };
        Ok(var)
    }

    fn get_keyword(line: impl AsRef<str>) -> Option<&'static Keyword> {
        let line = line.as_ref();
        for (keyword, keyword_str) in KEYWORDS.iter() {
            let keyword_line_iter = line.chars().zip(keyword_str.chars());
            let mut match_count = 0;
            for (ch, keyword_ch) in keyword_line_iter {
                if ch != keyword_ch {
                    break;
                }

                match_count += 1;
                if match_count == keyword_str.len() {
                    return Some(&keyword);
                }
            }
        }
        None
    }

    fn parse_line_statement(
        line: impl AsRef<str>,
        vm: &VirtualMachine,
        ctx: &VmContext,
    ) -> Result<Value> {
        let line = line.as_ref().trim();
        if line.is_empty() {
            return Ok(Value::None);
        }

        Expression::from_str(line, ctx, vm).evaluate()?;

        Ok(Value::None)
    }

    pub fn parse_repeat(
        lines: &mut CodeReader,
        _keyword_line: impl AsRef<str>, // maybe use this at a later point for "for-loop" functionality
        vm: &VirtualMachine,
        ctx: &VmContext,
    ) -> Result<()> {
        let repeat_ctx = ctx.clone();
        loop {
            let indx = lines.get_index();
            if let Ok(ScopeParseResult::Break) = Self::parse_scope(lines, vm, Some(&repeat_ctx)) {
                break;
            }
            lines.seek(indx);
        }
        Ok(())
    }

    fn skip_body(lines: &CodeReader) -> Result<()> {
        // Forward through the 'if' body and terminate on matching "end"
        let mut current_end_index = 0;
        let mut end_index_target = 1;
        for line in lines {
            let line = line.trim();
            match Self::get_keyword(&line) {
                Some(Keyword::Function | Keyword::IfScope | Keyword::RepeatScope) => {
                    end_index_target += 1
                }
                Some(Keyword::ScopeEnd) => current_end_index += 1,
                _ => (),
            };
            if current_end_index == end_index_target {
                return Ok(());
            }
        }
        Err("Could not find matching 'end' for current body!".into())
    }

    fn parse_if(
        lines: &mut CodeReader,
        keyword_line: impl AsRef<str>,
        vm: &VirtualMachine,
        ctx: &VmContext,
    ) -> Result<ScopeParseResult> {
        let mut keyword_line_chars = keyword_line.as_ref().chars();
        let mut encountered_letters = false;
        // Skip the 'if'.
        for ch in keyword_line_chars.by_ref() {
            if ch == ' ' {
                if encountered_letters {
                    break;
                }
                continue;
            }

            if ch.is_alphabetic() && !encountered_letters {
                encountered_letters = true;
            }
        }

        let mut expr_buf = String::default();
        let mut encountered_letters = false;
        for ch in keyword_line_chars {
            if ch == '?' {
                break;
            }

            if ch != ' ' && !Self::is_char_legal_identifier(ch) && !Self::is_char_legal_literal(ch)
            {
                continue;
            }

            if !encountered_letters {
                encountered_letters = true;
            }

            expr_buf.push(ch);
        }

        let if_ctx = ctx.clone();
        let expr = Expression::from_str(expr_buf.trim(), &if_ctx, vm);
        let expr_val = match expr.evaluate()? {
            Value::Boolean(x) => x,
            _ => return Err("Expression in 'if' must evaluate to a boolean!".into()),
        };

        if !expr_val {
            Self::skip_body(lines)?;
            return Ok(ScopeParseResult::None);
        }

        Self::parse_scope(lines, vm, Some(&if_ctx))
    }

    fn parse_return_value(
        keyword_line: impl AsRef<str>,
        vm: &VirtualMachine,
        ctx: &VmContext,
    ) -> Result<ScopeParseResult> {
        let keyword_line = keyword_line.as_ref();
        let first_space = keyword_line
            .find(' ')
            .ok_or("Could not parse return value!")?
            + 1;
        let expr_str = keyword_line.split_at(first_space).1;
        let expr = Expression::from_str(expr_str, ctx, vm);
        let value = expr.evaluate()?;
        Ok(ScopeParseResult::Return(value))
    }

    fn handle_keyword_instance(
        keyword: impl Borrow<Keyword>,
        keyword_line: impl AsRef<str>,
        lines: &mut CodeReader,
        vm: &VirtualMachine,
        ctx: &VmContext,
    ) -> Result<ScopeParseResult> {
        match keyword.borrow() {
            Keyword::Variable => ctx.push_var(Rc::new(RefCell::new(Self::parse_var(
                keyword_line,
                ctx,
                vm,
            )?))),
            Keyword::Function => {
                ctx.push_func(Function::Script(Self::read_func(lines, keyword_line)?))
            }
            Keyword::IfScope => return Self::parse_if(lines, keyword_line, vm, ctx),
            Keyword::RepeatScope => Self::parse_repeat(lines, keyword_line, vm, ctx)?,
            Keyword::ScopeEnd => return Ok(ScopeParseResult::None),
            Keyword::ScopeBreak => return Ok(ScopeParseResult::Break),
            Keyword::ScopeReturn => return Self::parse_return_value(keyword_line, vm, ctx),
        };
        Ok(ScopeParseResult::None)
    }

    fn parse_scope(
        lines: &mut CodeReader,
        vm: &VirtualMachine,
        ctx: Option<&VmContext>,
    ) -> Result<ScopeParseResult> {
        let ctx = match ctx {
            Some(x) => x,
            None => vm.get_context(),
        };

        let mut line_buf = String::default();
        loop {
            line_buf.clear();
            match lines.read_line(&mut line_buf) {
                Ok(0) => continue,
                Err(_) => break,
                _ => (),
            };
            let line = line_buf.trim_start();

            let keyword = match Self::get_keyword(line) {
                Some(Keyword::ScopeEnd) => break,
                Some(x) => x,
                None => {
                    Self::parse_line_statement(line, vm, ctx)?;
                    continue;
                }
            };

            let result = Self::handle_keyword_instance(keyword, line, lines, vm, ctx)?;
            match result {
                ScopeParseResult::Break | ScopeParseResult::Return(_) => return Ok(result),
                _ => (),
            }
        }
        Ok(ScopeParseResult::None)
    }

    pub(crate) fn parse_func_code(
        code: impl AsRef<str>,
        args: &[Argument],
        vm: &VirtualMachine,
    ) -> Result<ScopeParseResult> {
        let mut ctx = vm.get_context().clone();
        for arg in args {
            ctx.push_var(Rc::new(RefCell::new(arg.clone())));
        }
        let mut reader = CodeReader::from_str(code);
        Self::parse_scope(&mut reader, vm, Some(&mut ctx))
    }

    pub fn parse_buffer(mut input: CodeReader, vm: &VirtualMachine) -> Result<()> {
        Self::parse_scope(&mut input, vm, None)?;
        Ok(())
    }
}
