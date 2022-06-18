use crate::defs::{Function, FunctionBody, Parameter, Variable};
use crate::expressions::Expression;
use crate::identifiable::Identifiable;
use crate::keyword::{Keyword, KeywordInfo, KEYWORDS};
use crate::line_reader::LineReader;
use crate::operators::OPERATORS;
use crate::value::{Argument, Value};
use crate::vm::{Contextable, ExecutionContext, VirtualMachine, VmContext};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::io::BufReader;
use std::rc::Rc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

//TODO: Many lines are duplicated. Seperate into functions and simplify.
//TODO: Consider moving into seperate files.

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

    fn is_scope_end(st: impl AsRef<str>) -> bool {
        let st = st.as_ref();
        for (x, (y, z)) in st.chars().zip(st.chars().skip(1).zip(st.chars().skip(2))) {
            if x == 'e' && y == 'n' && z == 'd' {
                return true;
            }
        }
        false
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

    fn read_func_params(lines: &LineReader, keyword_line: impl ToString) -> Result<Vec<Parameter>> {
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

    fn read_func_signature(
        lines: &LineReader,
        keyword_line: impl AsRef<str>,
    ) -> Result<(String, Vec<Parameter>)> {
        let mut keyword_line = keyword_line.as_ref().chars();
        let name = Self::read_func_name(&mut keyword_line);
        let keyword_line = keyword_line.collect::<String>();

        let params = Self::read_func_params(lines, keyword_line)?;
        Ok((name, params))
    }

    fn read_func_body(lines: &LineReader) -> Result<FunctionBody> {
        // Assume that the function signature is not included
        let mut code = String::default();
        let mut line_buf = String::default();
        'line_loop: loop {
            if lines.read_line(&mut line_buf)? == 0 {
                break;
            }
            let mut encountered_letter = false;
            for ch in line_buf.chars() {
                if !encountered_letter && ch == ' ' {
                    continue;
                } else if ch.is_alphabetic() {
                    encountered_letter = true;
                }
                code.push(ch);
                if Self::is_scope_end(&mut code) {
                    code.pop();
                    code.pop();
                    code.pop();
                    break 'line_loop;
                }
            }
        }
        Ok(FunctionBody { code })
    }

    fn read_func(lines: &LineReader, keyword_line: impl AsRef<str>) -> Result<Function> {
        let (name, params) = Self::read_func_signature(lines, keyword_line)?;
        let body = Self::read_func_body(lines)?;
        let func = Function {
            name,
            params,
            body,
            ret_val: Value::Any,
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

        if KEYWORDS.iter().all(|x| x.get_keyword().contains(&name_buf)) {
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
            if ch == '=' {
                continue;
            }

            if ch == '?' {
                break;
            }

            if ch == '\n' || ch == '\r' {
                break;
            }

            expression_text.push(ch);
        }

        let val = if !expression_text.is_empty() {
            Expression::from_str(expression_text.trim(), expr_context).evaluate()?
        } else {
            Value::None
        };
        Ok(val)
    }

    fn parse_var(line: impl AsRef<str>, expr_context: &impl ExecutionContext) -> Result<Variable> {
        let mut chars = line.as_ref().chars();

        // Skip the included keyword. (forward the iterator until a space is hit)
        for ch in chars.by_ref() {
            if ch == ' ' {
                break;
            }
        }

        let name = Self::read_var_name(&mut chars)?;
        let value = Self::parse_var_value(&mut chars, expr_context)?;

        let var = Variable { name, value };
        Ok(var)
    }

    fn get_keyword(line: impl AsRef<str>) -> Option<&'static Keyword> {
        let line = line.as_ref();
        for keyword in KEYWORDS {
            let keyword_str = keyword.get_keyword();
            let keyword_line_iter = line.chars().zip(keyword_str.chars());
            let mut match_count = 0;
            for (ch, keyword_ch) in keyword_line_iter {
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

    fn parse_line_statement(line: impl AsRef<str>, vm: &mut VirtualMachine) -> Result<()> {
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

    pub fn parse_repeat(
        lines: &LineReader,
        keyword_line: impl AsRef<str>,
        vm: &mut VirtualMachine,
    ) -> Result<()> {
        //TODO: Implement
        panic!("Not implemented.");
    }

    pub fn parse_if(
        lines: &LineReader,
        keyword_line: impl AsRef<str>,
        vm: &mut VirtualMachine,
        ctx: &mut VmContext,
    ) -> Result<()> {
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
            if ch == '?' || ch == '\n' || ch == '\r' {
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

        let mut if_ctx = ctx.clone();
        let expr = Expression::from_str(expr_buf.trim(), &if_ctx);
        let expr_val = match expr.evaluate()? {
            Value::Boolean(x) => x,
            _ => return Err("Expression in 'if' must evaluate to a boolean!".into()),
        };

        if !expr_val {
            // Forward through the 'if' body and terminate on matching "end"
            let mut end_index = 0;
            let mut end_target_index = 0;
            for line in lines {
                let line = line.trim();
                if let Some(Keyword::IfScope(_) | Keyword::Function(_) | Keyword::RepeatScope(_)) =
                    Self::get_keyword(&line)
                {
                    end_target_index += 1;
                }

                if Self::is_scope_end(line) {
                    if end_index == end_target_index {
                        return Ok(());
                    }
                    end_index += 1;
                }
            }
            return Err("Could not find matching 'end' for current if statement!".into());
        }

        Self::parse_scope(lines, vm, Some(&mut if_ctx))?;

        Ok(())
    }

    fn handle_keyword_instance(
        keyword: impl Borrow<Keyword>,
        keyword_line: impl AsRef<str>,
        lines: &LineReader,
        vm: &mut VirtualMachine,
        ctx: &mut VmContext,
    ) -> Result<()> {
        match keyword.borrow() {
            Keyword::Variable(_) => {
                ctx.push_var(Rc::new(RefCell::new(Self::parse_var(keyword_line, ctx)?)))
            }
            Keyword::Function(_) => ctx.push_func(Self::read_func(lines, keyword_line)?),
            Keyword::IfScope(_) => Self::parse_if(lines, keyword_line, vm, ctx)?,
            Keyword::RepeatScope(_) => Self::parse_repeat(lines, keyword_line, vm)?,
            Keyword::ScopeEnd(_) => {
                return Err(
                    "Invalid structured program. Encountered end before scope start.".into(),
                )
            }
            Keyword::ScopeBreak(_) => panic!("Not implemented"),
        };
        Ok(())
    }

    fn parse_scope(
        lines: &LineReader,
        vm: &mut VirtualMachine,
        ctx: Option<&mut VmContext>,
    ) -> Result<()> {
        let mut ctx_val;
        let ctx = match ctx {
            Some(x) => x,
            None => {
                ctx_val = vm.get_context().clone();
                &mut ctx_val
            }
        };

        let mut line_buf = String::default();
        loop {
            line_buf.clear();
            if lines.read_line(&mut line_buf)? == 0 {
                break;
            }
            let line = line_buf.trim_start();

            if Self::is_scope_end(line) {
                return Ok(());
            }

            let keyword = match Self::get_keyword(line) {
                Some(x) => x,
                None => {
                    Self::parse_line_statement(line, vm)?;
                    continue;
                }
            };

            Self::handle_keyword_instance(keyword, line, lines, vm, ctx)?;
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
        let buf_reader = BufReader::new(code.as_bytes());
        let reader = LineReader::new(buf_reader);
        Self::parse_scope(&reader, vm, Some(&mut ctx))?;

        Ok(())
    }

    pub fn parse_buffer(input: LineReader, vm: &mut VirtualMachine) -> Result<()> {
        Self::parse_scope(&input, vm, None)
    }
}
