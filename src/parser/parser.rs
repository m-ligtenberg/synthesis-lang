use crate::parser::{ast::*, lexer::Token};
use crate::errors::{SynthesisError, ErrorKind, SourceLocation};
use std::collections::HashMap;

pub struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, position: 0 }
    }
    
    pub fn parse(&mut self) -> crate::Result<Program> {
        let items = self.parse_items()?;
        Ok(Program { items })
    }
    
    fn parse_items(&mut self) -> crate::Result<Vec<Item>> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            if let Some(item) = self.parse_item()? {
                items.push(item);
            }
        }
        
        Ok(items)
    }
    
    fn parse_item(&mut self) -> crate::Result<Option<Item>> {
        if self.is_at_end() {
            return Ok(None);
        }
        
        match self.current_token() {
            Some(Token::Import) => {
                let import = self.parse_import()?;
                Ok(Some(Item::Import(import)))
            }
            Some(Token::Loop) => {
                let loop_block = self.parse_loop()?;
                Ok(Some(Item::Loop(loop_block)))
            }
            _ => {
                let stmt = self.parse_statement()?;
                Ok(Some(Item::Statement(stmt)))
            }
        }
    }
    
    fn parse_import(&mut self) -> crate::Result<ImportItem> {
        self.consume_token(Token::Import)?;
        
        let module = match self.current_token() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(SynthesisError::new(
                ErrorKind::SyntaxError,
                "Expected module name after 'import'"
            )
            .with_suggestion("Add a module name like: import Audio")
            .with_suggestion("Available modules: Audio, Graphics, GUI, Hardware, Math, Time")
            .with_docs("https://synthesis-lang.org/docs/modules")),
        };
        
        let items = if self.match_token(&Token::Dot) {
            self.advance();
            if self.match_token(&Token::LeftBrace) {
                self.advance();
                let items = self.parse_import_list()?;
                self.consume_token(Token::RightBrace)?;
                Some(items)
            } else {
                None
            }
        } else {
            None
        };
        
        Ok(ImportItem { module, items })
    }
    
    fn parse_import_list(&mut self) -> crate::Result<Vec<String>> {
        let mut items = Vec::new();
        
        while !self.match_token(&Token::RightBrace) && !self.is_at_end() {
            if let Some(Token::Identifier(name)) = self.current_token() {
                items.push(name.clone());
                self.advance();
                
                if self.match_token(&Token::Comma) {
                    self.advance();
                }
            } else {
                break;
            }
        }
        
        Ok(items)
    }
    
    fn parse_loop(&mut self) -> crate::Result<LoopBlock> {
        self.consume_token(Token::Loop)?;
        self.consume_token(Token::LeftBrace)?;
        
        let body = self.parse_statements()?;
        
        self.consume_token(Token::RightBrace)?;
        
        Ok(LoopBlock { body })
    }
    
    fn parse_statements(&mut self) -> crate::Result<Vec<Statement>> {
        let mut statements = Vec::new();
        
        while !self.match_token(&Token::RightBrace) && !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    // Error recovery: skip to next likely statement start or block end
                    self.synchronize_after_error();
                    // Re-throw the error with recovery context
                    return Err(err.with_suggestion("Parser recovered and continued after this error"));
                }
            }
        }
        
        Ok(statements)
    }

    /// Skip tokens until we find a likely place to resume parsing
    fn synchronize_after_error(&mut self) {
        self.advance(); // Skip the problematic token
        
        while !self.is_at_end() {
            // Stop at statement boundaries
            match self.current_token() {
                Some(Token::Let) | Some(Token::If) | Some(Token::While) | 
                Some(Token::For) | Some(Token::Match) | Some(Token::Every) | 
                Some(Token::After) | Some(Token::RightBrace) => return,
                _ => {}
            }
            
            self.advance();
        }
    }
    
    fn parse_statement(&mut self) -> crate::Result<Statement> {
        match self.current_token() {
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::Match) => self.parse_match_statement(),
            Some(Token::Every) => self.parse_temporal_statement(),
            Some(Token::After) => self.parse_temporal_statement(),
            Some(Token::While) => self.parse_temporal_statement(),
            Some(Token::For) => self.parse_for_statement(),
            Some(Token::Let) => self.parse_let_statement(),
            Some(Token::Identifier(_)) if self.peek_token(1) == Some(&Token::Assignment) => {
                self.parse_assignment()
            }
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
            }
        }
    }
    
    fn parse_assignment(&mut self) -> crate::Result<Statement> {
        let name = match self.current_token() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(SynthesisError::new(
                ErrorKind::SyntaxError,
                "Expected variable name in assignment"
            )
            .with_suggestion("Variable names should start with a letter")
            .with_suggestion("Example: my_variable = Audio.mic_input()")),
        };
        
        self.consume_token(Token::Assignment)?;
        let value = self.parse_expression()?;
        
        Ok(Statement::Assignment { name, value })
    }
    
    fn parse_if_statement(&mut self) -> crate::Result<Statement> {
        self.consume_token(Token::If)?;
        
        let condition = self.parse_expression()?;
        
        self.consume_token(Token::LeftBrace)?;
        let then_branch = self.parse_statements()?;
        self.consume_token(Token::RightBrace)?;
        
        let else_branch = if self.match_token(&Token::Else) {
            self.advance();
            self.consume_token(Token::LeftBrace)?;
            let else_statements = self.parse_statements()?;
            self.consume_token(Token::RightBrace)?;
            Some(else_statements)
        } else {
            None
        };
        
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }
    
    fn parse_match_statement(&mut self) -> crate::Result<Statement> {
        self.consume_token(Token::Match)?;
        let expression = self.parse_expression()?;
        self.consume_token(Token::LeftBrace)?;
        
        let mut arms = Vec::new();
        while !self.match_token(&Token::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.consume_token(Token::Arrow)?;
            self.consume_token(Token::LeftBrace)?;
            let body = self.parse_statements()?;
            self.consume_token(Token::RightBrace)?;
            
            arms.push(MatchArm { pattern, body });
            
            if self.match_token(&Token::Comma) {
                self.advance();
            }
        }
        
        self.consume_token(Token::RightBrace)?;
        Ok(Statement::Match { expression, arms })
    }
    
    fn parse_pattern(&mut self) -> crate::Result<Pattern> {
        match self.current_token() {
            Some(Token::Underscore) => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                
                if self.match_token(&Token::LeftParen) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.match_token(&Token::RightParen) && !self.is_at_end() {
                        fields.push(self.parse_pattern()?);
                        if self.match_token(&Token::Comma) {
                            self.advance();
                        }
                    }
                    self.consume_token(Token::RightParen)?;
                    Ok(Pattern::Enum { name, fields: Some(fields) })
                } else {
                    Ok(Pattern::Identifier(name))
                }
            }
            Some(Token::Integer(val)) => {
                let val = *val;
                self.advance();
                Ok(Pattern::Literal(Literal::Integer(val)))
            }
            Some(Token::Float(val)) => {
                let val = *val;
                self.advance();
                Ok(Pattern::Literal(Literal::Float(val)))
            }
            Some(Token::Percentage(val)) => {
                let val = *val;
                self.advance();
                Ok(Pattern::Literal(Literal::Percentage(val)))
            }
            Some(Token::String(val)) => {
                let val = val.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(val)))
            }
            Some(Token::Boolean(val)) => {
                let val = *val;
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(val)))
            }
            _ => {
                let found_desc = self.current_token()
                    .map(token_description)
                    .unwrap_or("end of file".to_string());
                
                Err(SynthesisError::new(
                    ErrorKind::InvalidExpression,
                    format!("Invalid pattern: {}", found_desc)
                )
                .with_suggestion("Pattern matching supports numbers, strings, and wildcards")
                .with_suggestion("Use _ for catch-all patterns")
                .with_docs("https://synthesis-lang.org/docs/pattern-matching"))
            }
        }
    }
    
    fn parse_temporal_statement(&mut self) -> crate::Result<Statement> {
        let token = self.current_token().unwrap().clone();
        self.advance();
        
        match token {
            Token::Every => {
                self.consume_token(Token::LeftParen)?;
                let duration = self.parse_expression()?;
                self.consume_token(Token::RightParen)?;
                self.consume_token(Token::LeftBrace)?;
                let body = self.parse_statements()?;
                self.consume_token(Token::RightBrace)?;
                Ok(Statement::Every { duration, body })
            }
            Token::After => {
                self.consume_token(Token::LeftParen)?;
                let duration = self.parse_expression()?;
                self.consume_token(Token::RightParen)?;
                self.consume_token(Token::LeftBrace)?;
                let body = self.parse_statements()?;
                self.consume_token(Token::RightBrace)?;
                Ok(Statement::After { duration, body })
            }
            Token::While => {
                let condition = self.parse_expression()?;
                self.consume_token(Token::LeftBrace)?;
                let body = self.parse_statements()?;
                self.consume_token(Token::RightBrace)?;
                Ok(Statement::While { condition, body })
            }
            _ => Err(SynthesisError::new(
                ErrorKind::UnexpectedToken,
                "Invalid temporal statement"
            )
            .with_suggestion("Use 'every', 'after', or 'while' for time-based logic")
            .with_suggestion("Example: every(1.seconds) { ... }")
            .with_docs("https://synthesis-lang.org/docs/time")),
        }
    }

    fn parse_for_statement(&mut self) -> crate::Result<Statement> {
        self.consume_token(Token::For)?;
        
        let variable = match self.current_token() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(SynthesisError::new(
                ErrorKind::SyntaxError,
                "Expected variable name in for loop"
            )
            .with_suggestion("for loops need a variable: for item in list { ... }")
            .with_suggestion("Example: for i in 0..10 { ... }")),
        };
        
        self.consume_token(Token::In)?;
        let iterable = self.parse_expression()?;
        
        self.consume_token(Token::LeftBrace)?;
        let body = self.parse_statements()?;
        self.consume_token(Token::RightBrace)?;
        
        Ok(Statement::For {
            variable,
            iterable,
            body,
        })
    }

    fn parse_let_statement(&mut self) -> crate::Result<Statement> {
        self.consume_token(Token::Let)?;
        
        let name = match self.current_token() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(SynthesisError::new(
                ErrorKind::SyntaxError,
                "Expected variable name after 'let'"
            )
            .with_suggestion("let statements need a variable name")
            .with_suggestion("Example: let frequency = 440.0")),
        };
        
        let type_annotation = if self.match_token(&Token::Colon) {
            self.advance();
            Some(self.parse_type_annotation()?)
        } else {
            None
        };
        
        let value = if self.match_token(&Token::Assignment) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        Ok(Statement::Let {
            name,
            type_annotation,
            value,
        })
    }

    fn parse_type_annotation(&mut self) -> crate::Result<TypeAnnotation> {
        match self.current_token() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Ok(TypeAnnotation::Simple(name))
            }
            _ => Err(SynthesisError::new(
                ErrorKind::SyntaxError,
                "Expected type name in type annotation"
            )
            .with_suggestion("Common types: Audio, Graphics, Number, Text, Stream")
            .with_docs("https://synthesis-lang.org/docs/types")),
        }
    }
    
    fn parse_expression(&mut self) -> crate::Result<Expression> {
        self.parse_pipe()
    }
    
    fn parse_pipe(&mut self) -> crate::Result<Expression> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&Token::Pipe) || self.match_token(&Token::BiDirectionalPipe) {
            if self.match_token(&Token::Pipe) {
                self.advance();
                let right = self.parse_equality()?;
                expr = Expression::Pipe {
                    left: Box::new(expr),
                    right: Box::new(right),
                };
            } else if self.match_token(&Token::BiDirectionalPipe) {
                self.advance();
                let right = self.parse_equality()?;
                expr = Expression::BiDirectionalPipe {
                    left: Box::new(expr),
                    right: Box::new(right),
                };
            }
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> crate::Result<Expression> {
        let mut expr = self.parse_comparison()?;
        
        while let Some(op) = self.match_equality_op() {
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> crate::Result<Expression> {
        let mut expr = self.parse_range()?;
        
        while let Some(op) = self.match_comparison_op() {
            self.advance();
            let right = self.parse_range()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    fn parse_range(&mut self) -> crate::Result<Expression> {
        let mut expr = self.parse_term()?;
        
        if self.match_token(&Token::Range) || self.match_token(&Token::RangeInclusive) {
            let inclusive = self.match_token(&Token::RangeInclusive);
            self.advance();
            let end = self.parse_term()?;
            expr = Expression::Range {
                start: Box::new(expr),
                end: Box::new(end),
                inclusive,
            };
        }
        
        Ok(expr)
    }
    
    fn parse_term(&mut self) -> crate::Result<Expression> {
        let mut expr = self.parse_factor()?;
        
        while let Some(op) = self.match_term_op() {
            self.advance();
            let right = self.parse_factor()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_factor(&mut self) -> crate::Result<Expression> {
        let mut expr = self.parse_unary()?;
        
        while let Some(op) = self.match_factor_op() {
            self.advance();
            let right = self.parse_unary()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> crate::Result<Expression> {
        self.parse_call()
    }
    
    fn parse_call(&mut self) -> crate::Result<Expression> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&Token::LeftParen) {
                self.advance();
                let args = self.parse_arguments()?;
                self.consume_token(Token::RightParen)?;
                
                if let Expression::Identifier(name) = expr {
                    expr = Expression::FunctionCall {
                        module: None,
                        name,
                        args,
                        named_args: HashMap::new(),
                    };
                } else {
                    return Err(SynthesisError::new(
                    ErrorKind::SyntaxError,
                    "Invalid function call syntax"
                )
                .with_suggestion("Function calls need parentheses: function_name()")
                .with_suggestion("Module functions: Module.function_name()"));
                }
            } else if self.match_token(&Token::LeftBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.consume_token(Token::RightBracket)?;
                
                expr = Expression::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> crate::Result<Expression> {
        match self.current_token() {
            Some(Token::Integer(n)) => {
                let n = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Integer(n)))
            }
            Some(Token::Float(f)) => {
                let f = *f;
                self.advance();
                Ok(Expression::Literal(Literal::Float(f)))
            }
            Some(Token::Percentage(p)) => {
                let p = *p;
                self.advance();
                Ok(Expression::Literal(Literal::Percentage(p)))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(s)))
            }
            Some(Token::InterpolatedString(parts)) => {
                let parts = parts.clone();
                self.advance();
                Ok(Expression::InterpolatedString(parts))
            }
            Some(Token::Boolean(b)) => {
                let b = *b;
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(b)))
            }
            Some(Token::Unit(unit_string)) => {
                let unit_string = unit_string.clone();
                self.advance();
                
                // Parse "value.unit" format
                let parts: Vec<&str> = unit_string.split('.').collect();
                if parts.len() == 2 {
                    let value_str = parts[0];
                    let unit = parts[1].to_string();
                    
                    if let Ok(int_val) = value_str.parse::<i64>() {
                        Ok(Expression::UnitValue {
                            value: Box::new(Expression::Literal(Literal::Integer(int_val))),
                            unit,
                        })
                    } else if let Ok(float_val) = value_str.parse::<f64>() {
                        Ok(Expression::UnitValue {
                            value: Box::new(Expression::Literal(Literal::Float(float_val))),
                            unit,
                        })
                    } else {
                        Err(SynthesisError::new(
                            ErrorKind::InvalidExpression,
                            format!("Unit value '{}' is not valid", unit_string)
                        )
                        .with_suggestion("Unit values should be like: 3.5.seconds, 440.hz, 0.5.volume")
                        .with_docs("https://synthesis-lang.org/docs/units"))
                    }
                } else {
                    Err(SynthesisError::new(
                        ErrorKind::InvalidExpression,
                        format!("Unit format '{}' is invalid", unit_string)
                    )
                    .with_suggestion("Use format: number.unit (like 3.seconds or 440.hz)")
                    .with_docs("https://synthesis-lang.org/docs/units"))
                }
            }
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                
                if self.match_token(&Token::Dot) {
                    self.advance();
                    if let Some(Token::Identifier(func_name)) = self.current_token() {
                        let func_name = func_name.clone();
                        self.advance();
                        
                        if self.match_token(&Token::LeftParen) {
                            self.advance();
                            let (args, named_args) = self.parse_function_arguments()?;
                            self.consume_token(Token::RightParen)?;
                            
                            Ok(Expression::FunctionCall {
                                module: Some(name),
                                name: func_name,
                                args,
                                named_args,
                            })
                        } else {
                            // This is a method call without parentheses (property access)
                            Ok(Expression::MethodCall {
                                object: Box::new(Expression::Identifier(name)),
                                method: func_name,
                                args: Vec::new(),
                                named_args: HashMap::new(),
                            })
                        }
                    } else {
                        Err(SynthesisError::new(
                            ErrorKind::SyntaxError,
                            "Expected function name after '.'"
                        )
                        .with_suggestion("Module functions: Module.function_name()")
                        .with_suggestion("Example: Audio.mic_input(), Graphics.clear()"))
                    }
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume_token(Token::RightParen)?;
                Ok(expr)
            }
            Some(Token::LeftBracket) => {
                self.parse_array_literal()
            }
            Some(Token::LeftBrace) => {
                self.parse_block()
            }
            Some(Token::Branch(count)) => {
                let count = *count;
                self.advance();
                self.consume_token(Token::LeftParen)?;
                let stream = self.parse_expression()?;
                self.consume_token(Token::RightParen)?;
                Ok(Expression::StreamBranch {
                    stream: Box::new(stream),
                    count,
                })
            }
            _ => {
                let found_desc = self.current_token()
                    .map(token_description)
                    .unwrap_or("end of file".to_string());
                
                Err(SynthesisError::new(
                    ErrorKind::UnexpectedToken,
                    format!("Unexpected {} in expression", found_desc)
                )
                .with_suggestion("Check the syntax around this area")
                .with_suggestion("Look for missing punctuation or operators")
                .with_docs("https://synthesis-lang.org/docs/syntax"))
            }
        }
    }
    
    fn parse_array_literal(&mut self) -> crate::Result<Expression> {
        self.consume_token(Token::LeftBracket)?;
        
        let mut elements = Vec::new();
        
        while !self.match_token(&Token::RightBracket) && !self.is_at_end() {
            elements.push(self.parse_expression()?);
            
            if self.match_token(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        
        self.consume_token(Token::RightBracket)?;
        Ok(Expression::ArrayLiteral(elements))
    }

    fn parse_block(&mut self) -> crate::Result<Expression> {
        self.consume_token(Token::LeftBrace)?;
        
        let mut fields = HashMap::new();
        
        while !self.match_token(&Token::RightBrace) && !self.is_at_end() {
            if let Some(Token::Identifier(key)) = self.current_token() {
                let key = key.clone();
                self.advance();
                self.consume_token(Token::Colon)?;
                let value = self.parse_expression()?;
                fields.insert(key, value);
                
                if self.match_token(&Token::Comma) {
                    self.advance();
                }
            } else {
                break;
            }
        }
        
        self.consume_token(Token::RightBrace)?;
        
        Ok(Expression::Block { fields })
    }
    
    fn parse_function_arguments(&mut self) -> crate::Result<(Vec<Expression>, HashMap<String, Expression>)> {
        let mut args = Vec::new();
        let mut named_args = HashMap::new();
        
        while !self.match_token(&Token::RightParen) && !self.is_at_end() {
            // Check for named argument (identifier: expression)
            if let Some(Token::Identifier(_)) = self.current_token() {
                if self.peek_token(1) == Some(&Token::Colon) {
                    // This is a named argument
                    let name = match self.current_token() {
                        Some(Token::Identifier(name)) => {
                            let name = name.clone();
                            self.advance();
                            name
                        }
                        _ => return Err(SynthesisError::new(
                            ErrorKind::SyntaxError,
                            "Expected parameter name in function call"
                        )
                        .with_suggestion("Named parameters: function(name: value)")
                        .with_suggestion("Example: Audio.apply_reverb(room_size: 0.8)")),
                    };
                    
                    self.consume_token(Token::Colon)?;
                    let value = self.parse_expression()?;
                    named_args.insert(name, value);
                } else {
                    args.push(self.parse_expression()?);
                }
            } else {
                args.push(self.parse_expression()?);
            }
            
            if self.match_token(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        
        Ok((args, named_args))
    }

    fn parse_arguments(&mut self) -> crate::Result<Vec<Expression>> {
        let (args, _) = self.parse_function_arguments()?;
        Ok(args)
    }
    
    // Helper methods
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    
    fn peek_token(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position + offset)
    }
    
    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.tokens.get(self.position - 1)
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
    
    fn match_token(&self, token: &Token) -> bool {
        std::mem::discriminant(self.current_token().unwrap_or(&Token::Eof)) 
            == std::mem::discriminant(token)
    }
    
    fn consume_token(&mut self, expected: Token) -> crate::Result<()> {
        if self.match_token(&expected) {
            self.advance();
            Ok(())
        } else {
            let expected_desc = token_description(&expected);
            let found_desc = self.current_token()
                .map(token_description)
                .unwrap_or("end of file".to_string());
            
            Err(SynthesisError::new(
                ErrorKind::UnexpectedToken,
                format!("Expected {} but found {}", expected_desc, found_desc)
            )
            .with_suggestion("Check your syntax for missing punctuation")
            .with_suggestion("Make sure all blocks are properly closed with }")
            .with_docs("https://synthesis-lang.org/docs/syntax"))
        }
    }
    
    /// Create a source location for error reporting
    fn current_location(&self, filename: &str) -> SourceLocation {
        SourceLocation {
            line: 1, // TODO: Track line numbers in lexer
            column: self.position,
            filename: filename.to_string(),
        }
    }

    fn match_equality_op(&self) -> Option<BinaryOperator> {
        match self.current_token() {
            Some(Token::Equals) => Some(BinaryOperator::Equal),
            Some(Token::NotEqual) => Some(BinaryOperator::NotEqual),
            _ => None,
        }
    }
    
    fn match_comparison_op(&self) -> Option<BinaryOperator> {
        match self.current_token() {
            Some(Token::LessThan) => Some(BinaryOperator::LessThan),
            Some(Token::LessThanOrEqual) => Some(BinaryOperator::LessThanOrEqual),
            Some(Token::GreaterThan) => Some(BinaryOperator::GreaterThan),
            Some(Token::GreaterThanOrEqual) => Some(BinaryOperator::GreaterThanOrEqual),
            _ => None,
        }
    }
    
    fn match_term_op(&self) -> Option<BinaryOperator> {
        match self.current_token() {
            Some(Token::Plus) => Some(BinaryOperator::Add),
            Some(Token::Minus) => Some(BinaryOperator::Subtract),
            _ => None,
        }
    }
    
    fn match_factor_op(&self) -> Option<BinaryOperator> {
        match self.current_token() {
            Some(Token::Multiply) => Some(BinaryOperator::Multiply),
            Some(Token::Divide) => Some(BinaryOperator::Divide),
            _ => None,
        }
    }
}

/// Convert tokens to user-friendly descriptions
fn token_description(token: &Token) -> String {
    match token {
        Token::LeftBrace => "{".to_string(),
        Token::RightBrace => "}".to_string(),
        Token::LeftParen => "(".to_string(),
        Token::RightParen => ")".to_string(),
        Token::LeftBracket => "[".to_string(),
        Token::RightBracket => "]".to_string(),
        Token::Comma => ",".to_string(),
        Token::Colon => ":".to_string(),
        Token::Dot => ".".to_string(),
        Token::Assignment => "=".to_string(),
        Token::Arrow => "->".to_string(),
        Token::Import => "import".to_string(),
        Token::Loop => "loop".to_string(),
        Token::If => "if".to_string(),
        Token::Else => "else".to_string(),
        Token::Match => "match".to_string(),
        Token::Every => "every".to_string(),
        Token::After => "after".to_string(),
        Token::While => "while".to_string(),
        Token::For => "for".to_string(),
        Token::In => "in".to_string(),
        Token::Func => "func".to_string(),
        Token::Let => "let".to_string(),
        Token::Return => "return".to_string(),
        Token::Plus => "+".to_string(),
        Token::Minus => "-".to_string(),
        Token::Multiply => "*".to_string(),
        Token::Divide => "/".to_string(),
        Token::Equals => "==".to_string(),
        Token::NotEqual => "!=".to_string(),
        Token::LessThan => "<".to_string(),
        Token::LessThanOrEqual => "<=".to_string(),
        Token::GreaterThan => ">".to_string(),
        Token::GreaterThanOrEqual => ">=".to_string(),
        Token::Identifier(name) => format!("identifier '{}'", name),
        Token::Integer(num) => format!("number {}", num),
        Token::Float(num) => format!("number {}", num),
        Token::Percentage(num) => format!("percentage {}%", num * 100.0),
        Token::String(s) => format!("string \"{}\"", s),
        Token::Boolean(b) => format!("boolean {}", b),
        Token::Unit(u) => format!("unit value {}", u),
        Token::Newline => "newline".to_string(),
        Token::Eof => "end of file".to_string(),
        _ => format!("{:?}", token).to_lowercase(),
    }
}