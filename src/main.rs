use std::io::Read;
use std::vec;
use std::fs::File;


#[derive(Debug, Clone)]
enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    NumberLiteral(f64),
    StringLiteral(String)
}

#[derive(Debug)]
enum TokenType {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    NumberLiteral,
    StringLiteral
}

struct Lexer {
    tokens: Vec<Token>,
    source: Vec<char>
}

impl Lexer {

    fn new(source: String) -> Self {

        return Self{
            tokens: vec![],
            source: source.chars().collect()
        };
    }

    fn lex(&mut self) {
        let mut index: usize = 0;

        while index < self.source.len() {
            match self.source[index] {
                ' ' | '\n' | '\t' | '\r' => {},
                '{' => self.tokens.push(Token::LeftBrace),
                '}' => self.tokens.push(Token::RightBrace),
                '[' => self.tokens.push(Token::LeftBracket),
                ']' => self.tokens.push(Token::RightBracket),
                ',' => self.tokens.push(Token::Comma),
                ':' => self.tokens.push(Token::Colon),
                '"' => {
                    index += 1;
                    let start: usize = index;
                    while index < self.source.len() && self.source[index] != '"' {
                        if self.source[index] == '\\' {
                            index += 1;
                        }

                        index += 1;
                    }

                    let s: String = self.source[start..index].iter().collect();
                    self.tokens.push(Token::StringLiteral(s));
                },
                _ => {
                    let start: usize = index;
                    while index < self.source.len() && !is_delim(self.source[index]) {
                        index += 1;
                    }

                    let s: String = self.source[start..index].iter().collect();
                    self.tokens.push(Token::NumberLiteral(s.parse::<f64>().unwrap()));
                    index -= 1;
                },
            }

            index += 1;
        }
    }
}

fn is_delim(c: char) -> bool {
    return c == ',' || c == '{' || c == '}' || c == '[' || c == ']' || c == ':' || c == '\n' || c == '\t' || c == '\r';
}

#[derive(Debug)]
enum JsonExpression {
    Number(f64),
    String(String),
    Array(Vec<Box<JsonExpression>>),
    Object(Vec<(String, Box<JsonExpression>)>)
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        return Self { tokens, current: 0};
    }

    fn parse(&mut self) -> Result<JsonExpression, String> {
        match self.tokens[self.current] {
            Token::LeftBrace => return self.parse_object(),
            Token::LeftBracket => return self.parse_array(),
            _ => return Err(String::from("")) 
        }
    }

    fn parse_expression(&mut self) -> Result<JsonExpression, String> {
        let token = self.tokens[self.current].clone();
        match token {
            Token::LeftBracket => return self.parse_array(),
            Token::LeftBrace => return self.parse_object(),
            Token::NumberLiteral(n) => {
                self.current += 1;
                return Ok(JsonExpression::Number(n))
            },
            Token::StringLiteral(s) => {
                self.current += 1;
                return Ok(JsonExpression::String(s))
            },
            _ => return Err(String::from("Unexpected token at start of expression..."))
        } 
    }

    fn parse_array(&mut self) -> Result<JsonExpression, String> {
        
        match self.tokens[self.current] {
            Token::LeftBracket => self.current += 1,
            _ => return Err(String::from("Expected left bracket..."))
        }

        let mut elements: Vec<Box<JsonExpression>> = Vec::new();

        // dont parse array as it is empty
        if let Token::RightBracket = self.tokens[self.current] {
            self.current += 1;
            return Ok(JsonExpression::Array(elements)) 
        }

        loop {
            let element = self.parse_expression();
            match element {
                Ok(e) => elements.push(Box::new(e)),
                Err(e) => return Err(e),
            }

            match self.tokens[self.current] {
                Token::Comma => self.current += 1,
                _ => break 
            } 
        }

        match self.tokens[self.current] {
            Token::RightBracket => self.current += 1,
            _ => return Err(String::from("Expected right bracket..."))
        }
        
        return Ok(JsonExpression::Array(elements))
    }

    fn parse_object(&mut self) -> Result<JsonExpression, String> {
        match self.tokens[self.current] {
            Token::LeftBrace => self.current += 1,
            _ => return Err(String::from("Expected left brace..."))
        }

        let mut key_values_pairs: Vec<(String, Box<JsonExpression>)> = Vec::new();

        // dont parse object as it is empty
        if let Token::RightBrace = self.tokens[self.current] {
            self.current += 1;
            return Ok(JsonExpression::Object(key_values_pairs)) 
        }

        loop {
            let key = match self.tokens[self.current].clone() {
                Token::StringLiteral(s) => {
                    self.current += 1;
                    s
                },
                _ => return Err(String::from("Expected string literal..."))
            };

            match self.tokens[self.current] {
                Token::Colon => self.current += 1,
                _ => return Err(String::from("Expected colon...")) 
            } 

            let value = match self.parse_expression() {
                Ok(e) => e,
                Err(err) => return Err(err),
            };

            key_values_pairs.push((key, Box::new(value)));

            match self.tokens[self.current] {
                Token::Comma => self.current += 1,
                _ => break 
            } 
        }

        match self.tokens[self.current] {
            Token::RightBrace => self.current += 1,
            _ => return Err(String::from("Expected right brace..."))
        }
        
        return Ok(JsonExpression::Object(key_values_pairs))
    }
}

fn json(source: String) -> Result<JsonExpression, String> {
    let mut lexer = Lexer::new(source);
    lexer.lex();

    let mut parser = Parser::new(lexer.tokens);
    return parser.parse();
}

#[cfg(test)]
mod tests {
    use crate::{json, JsonExpression};

    #[test]
    fn empty_object() {
        let expr = json(String::from("{}")).unwrap();
        assert!(matches!(JsonExpression::Object(vec![]), expr)); 
    }

    #[test]
    fn empty_array() {
        let expr = json(String::from("[]")).unwrap();
        assert!(matches!(JsonExpression::Array(vec![]), expr)); 
    }

    #[test]
    fn object() {
        let expr = json(String::from("
        {
            \"one\": 1,
            \"two\": 2.0
        }
        ")).unwrap();
        assert!(matches!(
            JsonExpression::Object(vec![
                (String::from("one"), Box::new(JsonExpression::Number(1.0))),
                (String::from("two"), Box::new(JsonExpression::Number(2.0))),
            ]), 
            expr
        )); 
    }
}

fn main() {
}
