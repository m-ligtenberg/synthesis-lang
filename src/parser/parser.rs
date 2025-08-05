use crate::parser::{ast::*, lexer::Token};
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
            _ => return Err(anyhow::anyhow!("Expected module name after import").into()),
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
            statements.push(self.parse_statement()?);
        }
        
        Ok(statements)
    }
    
    fn parse_statement(&mut self) -> crate::Result<Statement> {
        match self.current_token() {
            Some(Token::If) => self.parse_if_statement(),
            Some(Token::Match) => self.parse_match_statement(),
            Some(Token::Every) => self.parse_temporal_statement(),
            Some(Token::After) => self.parse_temporal_statement(),
            Some(Token::While) => self.parse_temporal_statement(),
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
            _ => return Err(anyhow::anyhow!("Expected identifier in assignment").into()),
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
            _ => Err(anyhow::anyhow!("Expected pattern").into()),
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
                self.consume_token(Token::LeftParen)?;
                let condition = self.parse_expression()?;
                self.consume_token(Token::RightParen)?;
                self.consume_token(Token::LeftBrace)?;
                let body = self.parse_statements()?;
                self.consume_token(Token::RightBrace)?;
                Ok(Statement::While { condition, body })
            }
            _ => Err(anyhow::anyhow!("Unexpected temporal token").into()),
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
        let mut expr = self.parse_term()?;
        
        while let Some(op) = self.match_comparison_op() {
            self.advance();
            let right = self.parse_term()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
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
                    return Err(anyhow::anyhow!("Invalid function call").into());
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
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(s)))
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
                        Err(anyhow::anyhow!("Invalid unit value: {}", unit_string).into())
                    }
                } else {
                    Err(anyhow::anyhow!("Invalid unit format: {}", unit_string).into())
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
                            Ok(Expression::Identifier(format!("{}.{}", name, func_name)))
                        }
                    } else {
                        Err(anyhow::anyhow!("Expected function name after dot").into())
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
            _ => Err(anyhow::anyhow!("Unexpected token in expression").into()),
        }
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
                        _ => return Err(anyhow::anyhow!("Expected identifier in named argument").into()),
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
            Err(anyhow::anyhow!("Expected {:?}, found {:?}", expected, self.current_token()).into())
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