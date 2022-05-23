use core::f32;
use std::panic;
use std::{
    fs::File, 
    io::Read, 
    iter::Peekable, 
    str::Chars
};

use llvm_sys::core::*;
use llvm_sys::execution_engine::LLVMExecutionEngineGetErrMsg;
use llvm_sys::prelude::*;

#[derive(Debug)]

pub enum TokenType {
    Error,
    Ascii(char),
    Ident(String),
    Number(f64),
    KeywordFunc,
    KeywordReturn,
    KeywordIf,
    KeywordElse,
    KeywordForeign,
}

impl TokenType {
    pub fn from_string(string : &str) -> TokenType {
        return match string {
            "if" => TokenType::KeywordIf,
            "func" => TokenType::KeywordFunc,
            "else" => TokenType::KeywordElse,
            "return" =>  TokenType::KeywordReturn,
            "foreign" =>  TokenType::KeywordForeign,
            _ if string.chars().nth(0).unwrap().is_numeric() => TokenType::Number(string.parse().unwrap()),
            _ if string.chars().nth(0).unwrap().is_alphabetic() => TokenType::Ident(string.to_string()),
            _ => TokenType::Error
        }
    }
}

#[derive(Debug)]
pub struct Token {
    line_nr : usize,
    token_type : TokenType,
}

pub struct Lexer {
    tokens : Vec<Token>,
    buffer : String,
    line_nr : usize,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            tokens : vec![],
            buffer : String::new(),
            line_nr : 0
        }
    }

    pub fn print_tokens(&self) {
        for token in &self.tokens {
            println!("{:?}", token);
        }
    }

    pub fn load_file(&mut self, buffer : &String) {
        self.buffer = buffer.clone();

        let mut iter = self.buffer.chars().peekable();

        while let Some(c) = iter.peek() {
            if c.is_alphanumeric() {
                let string = self.parse_alphanumeric(&mut iter);

                self.tokens.push(Token {
                    line_nr : self.line_nr,
                    token_type : TokenType::from_string(&string),
                });

            } else if !c.is_whitespace() {
                self.tokens.push(Token { 
                    line_nr: self.line_nr, 
                    token_type : TokenType::Ascii(*c)
                });

                iter.next();
            } else /* its whitespace */ {
                if *c == '\n' {
                    self.line_nr += 1;
                }
                iter.next();
            }
        }
    }
    
    fn parse_alphanumeric(&self, iter : &mut Peekable<Chars>) -> String {
        let mut ident = String::new();

        for (index, character) in iter.clone().enumerate() {
            if character.is_alphanumeric() {
                ident.push(character);
            } else {
                for _ in 0..index {
                    iter.next();
                }
                
                break;
            }
        }
        
        return ident;
    }
}

trait AstNode {
    fn codegen(&self) -> LLVMValueRef;
}

struct AstNumber<T> {
    value : T
}

impl AstNode for AstNumber<f64> {
    fn codegen(&self) -> LLVMValueRef {
        unsafe { LLVMConstReal(LLVMDoubleType(), self.value) }
    }
}

struct Context {
    module : LLVMModuleRef,
    builder : LLVMBuilderRef,
}

impl Context {
    fn new() -> Context {
        Context { 
            module : unsafe { LLVMModuleCreateWithName("RK".as_ptr() as *const i8) },
            builder : unsafe { LLVMCreateBuilder() }
        }
    }
}

pub struct Parser {
    context : Context
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            context : Context::new()
        }
    }

    pub fn parse(&mut self, lexer : &Lexer) {
        let mut iter = lexer.tokens.iter().peekable();

        while let Some(token) = iter.peek() {

            let node = match token.token_type {
                TokenType::Number(n) => Some(AstNumber {value : n}),
                _ => None
            };

            if node.is_some() {
                unsafe { LLVMDumpValue(node.unwrap().codegen()); }
            }

            iter.next();
        }

        unsafe { LLVMDumpModule(self.context.module); }
    }
}

