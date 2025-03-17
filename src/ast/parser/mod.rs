mod expression;

use std::iter::Peekable;
use std::mem;

use expression::ExpressionParser;

use super::error::ErrorKind;
use super::node::{DataItem, LValue, UnaryOperator};
use super::{BinaryOperator, Error, Expression, Program, Statement};
use crate::tokens::{Lexer, Token};


pub struct Parser<'a> {
    expr_parser: ExpressionParser<'a>, // Se incluye ExpressionParser en el struct
    current_token: Option<Token>, // self.current_token possible fix
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let first_token = lexer.next();
        Self {
            expr_parser: ExpressionParser::new(lexer),
            current_token: first_token,
        }
    }

    /*pub fn advance(&mut self) {
        self.current_token = self.expr_parser.lexer.next();
    }
    */

    pub fn current_token(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }

    pub fn parse(&mut self) -> (Program, Vec<Error>) {
        let mut errors = Vec::new();
        let mut program = Vec::new(); // Aquí almacenamos las líneas que estamos parseando
    
        while let Some(token) = self.expr_parser.lexer.peek() {
            match token {
                Token::Number(_) => {
                    // Es posible que el número sea el inicio de una línea, así que parseamos la sentencia
                    match self.line() {
                        Ok((_, stmt)) => program.push(stmt),
                        Err(e) => errors.push(e),
                    }
                }
                Token::Let | Token::Identifier(_) | Token::Print | Token::Pause | Token::Input | Token::Wait |
                Token::Goto | Token::For | Token::Next | Token::End | Token::Gosub | Token::If |
                Token::Return | Token::Data | Token::Read | Token::Restore | Token::Poke | Token::Call |
                Token::Dim | Token::Rem(_) => {
                    // Si encontramos una sentencia de tipo conocido, la procesamos
                    match self.statement() {
                        Ok(stmt) => program.push(stmt),
                        Err(e) => errors.push(e),
                    }
                }
                _ => {
                    // Si encontramos un token inesperado, lo registramos como error
                    errors.push(Error {
                        kind: ErrorKind::UnexpectedToken,
                        line: self.lexer.current_line(),
                    });
                    self.expr_parser.lexer.next();  // Avanzamos al siguiente token
                }
            }
        }
    
        // Cambiar `statements` por `lines`, ya que el error indica que `Program` tiene el campo `lines`
        (Program { lines: program }, errors)
    }
    
    

    fn let_(&mut self) -> Result<Statement, Error> { // TODO revisar la siguiente función
        println!("let");
    
        let variable; // Declaramos la variable fuera del match
    
        // Revisamos si el token actual es "let"
        match &mut self.current_token {
            Some(Token::Let) => {
                // Si el token es "let", avanzamos al siguiente token
                self.current_token = self.expr_parser.lexer.next();
    
                // Ahora esperamos que el siguiente token sea un identificador
                match &self.current_token {
                    Some(Token::Identifier(_)) => {
                        // Si es un identificador, analizamos la variable
                        variable = self.expr_parser.lvalue()?;
                        println!("identifier: {:?}", variable);
                    }
                    _ => {
                        // Si no es un identificador, retornamos un error
                        return Err(Error {
                            kind: ErrorKind::ExpectedIdentifier,
                            line: self.expr_parser.lexer.current_line(),
                        });
                    }
                }
            }
            Some(Token::Identifier(v)) => {
                // Si el token es un identificador sin "let", tomamos el valor
                self.current_token = Some(Token::Identifier(mem::take(v)));
                println!("identifier");
                // Llamamos a `lvalue()` para obtener la variable, envuelta en Ok
                variable = self.expr_parser.lvalue()?;
            }
            _ => {
                // Si el token no es "let" ni un identificador, es un error
                unreachable!("Expected 'let' or an identifier, found {:?}", self.current_token);
            }
        };
    
        // Después de la variable, esperamos el token "="
        if self.current_token != Some(Token::Equal) {
            println!("not equal {:?}", self.current_token);
            return Err(Error {
                kind: ErrorKind::UnexpectedToken,
                line: self.expr_parser.lexer.current_line(),
            });
        }
    
        println!("equal");
    
        // Avanzamos al siguiente token, que debe ser la expresión
        self.current_token = self.expr_parser.lexer.next();
    
        // Parseamos la expresión que debe estar después del "="
        let expression = self.expr_parser.parse()?;
    
        // Verificamos si la expresión fue válida
        let expression = if let Some(expression) = expression {
            println!("expression: {:?}", expression);
            expression
        } else {
            println!("no expression");
            return Err(Error {
                kind: ErrorKind::ExpectedExpression,
                line: self.expr_parser.lexer.current_line(),
            });
        };
    
        // Finalmente, retornamos el Statement de tipo Let
        Ok(Statement::Let {
            variable, // Ahora la variable está disponible en este contexto
            expression,
        })
    }
    
    
    

     fn pause(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let mut content = Vec::new();

         while let Some(expr) = self.expr_parser.parse()? {
             content.push(expr);

             if self.current_token == Some(Token::Semicolon) {
                 self.current_token = self.expr_parser.lexer.next();
             } else {
                 break;
             }
         }

         Ok(Statement::Pause { content })
     }

     fn print(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let mut content = Vec::new();

         while let Some(expr) = self.expr_parser.parse()? {
             content.push(expr);

             if self.current_token == Some(Token::Semicolon) {
                 self.current_token = self.expr_parser.lexer.next();
             } else {
                 break;
             }
         }

         Ok(Statement::Print { content })
     }

     fn input(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let prompt = self.expr_parser.parse()?;

         if self.current_token == Some(Token::Semicolon) {
             self.current_token = self.expr_parser.lexer.next();
         }

         let variable = match self.current_token {
             Some(Token::Identifier(_)) => self.expr_parser.lvalue()?,
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedIdentifier,
                     line: self.lexer.current_line(),
                 });
             }
         };

         self.current_token = self.expr_parser.lexer.next();

         Ok(Statement::Input { prompt, variable })
     }

     fn wait(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let time = self.expr_parser.parse()?; // could be self.parse()?

         Ok(Statement::Wait { time })
     }

     fn data(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let mut values = Vec::new();

         loop {
             match &mut self.current_token {
                 Some(Token::Number(n)) => {
                     values.push(DataItem::Number(*n));
                     self.current_token = self.expr_parser.lexer.next();
                 }
                 Some(Token::String(s)) => {
                     values.push(DataItem::String(std::mem::take(s)));
                     self.current_token = self.expr_parser.lexer.next();
                 }
                 _ => {
                     return Err(Error {
                         kind: ErrorKind::ExpectedDataItem,
                         line: self.lexer.current_line(),
                     });
                 }
             }

             if self.current_token == Some(Token::Comma) {
                 self.current_token = self.expr_parser.lexer.next();
             } else {
                 break;
             }
         }

         Ok(Statement::Data { values })
     }

     fn read(&mut self) -> Result<Statement, Error> { // TODO revisar esta función y el tipo Read en node.rs
        self.current_token = self.expr_parser.lexer.next();
        let mut variables = Vec::new();
    
        loop {
            match self.current_token {
                Some(Token::Identifier(_)) => {
                    // Usamos ? para propagar el error de `lvalue()`
                    let variable = self.expr_parser.lvalue()?;  // Esto devuelve un `LValue`, no un `Result`
                    variables.push(variable);
                    self.current_token = self.expr_parser.lexer.next();
                }
                _ => {
                    return Err(Error {
                        kind: ErrorKind::ExpectedIdentifier,
                        line: self.lexer.current_line(),
                    });
                }
            }
    
            // Avanzamos si encontramos una coma
            if self.current_token == Some(Token::Comma) {
                self.current_token = self.expr_parser.lexer.next();
            } else {
                break;
            }
        }
    
        Ok(Statement::Read { variables }) // TODO revisar! Esta función se ha modificado para devolver Vec<LValue> en vez de  Result<LValue, Error>
    }
    

     fn restore(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let line_number = match &self.current_token {
             Some(Token::Number(n)) => match u32::try_from(*n) {
                 Ok(n) => Some(n),
                 Err(_) => {
                     return Err(Error {
                         kind: ErrorKind::ExpectedUnsigned,
                         line: self.lexer.current_line(),
                     });
                 }
             },
             _ => None,
         };

         if line_number.is_some() {
             self.current_token = self.expr_parser.lexer.next();
         }

         Ok(Statement::Restore { line_number })
     }

     fn poke(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let address = match &self.current_token {
             Some(Token::Number(n)) => u32::try_from(*n).map_err(|_e| Error {
                 kind: ErrorKind::ExpectedUnsigned,
                 line: self.lexer.current_line(),
             })?,
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedUnsigned,
                     line: self.lexer.current_line(),
                 });
             }
         };

         self.current_token = self.expr_parser.lexer.next();
         if self.current_token != Some(Token::Comma) {
             return Err(Error {
                 kind: ErrorKind::UnexpectedToken,
                 line: self.lexer.current_line(),
             });
         }

         self.current_token = self.expr_parser.lexer.next();
         let mut values: Vec<u8> = Vec::new();

         loop {
             match &mut self.current_token {
                 Some(Token::Number(n)) => {
                     values.push(u8::try_from(*n).map_err(|_e| Error {
                         kind: ErrorKind::ExpectedUnsigned,
                         line: self.lexer.current_line(),
                     })?);
                     self.current_token = self.expr_parser.lexer.next();
                 }
                 _ => {
                     return Err(Error {
                         kind: ErrorKind::ExpectedUnsigned,
                         line: self.lexer.current_line(),
                     });
                 }
             }

             if self.current_token == Some(Token::Comma) {
                 self.current_token = self.expr_parser.lexer.next();
             } else {
                 break;
             }
         }

         Ok(Statement::Poke { address, values })
     }

     fn call(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let address = match &self.current_token {
             Some(Token::Number(n)) => u32::try_from(*n).map_err(|_e| Error {
                 kind: ErrorKind::ExpectedUnsigned,
                 line: self.lexer.current_line(),
             })?,
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedUnsigned,
                     line: self.lexer.current_line(),
                 });
             }
         };

         self.current_token = self.expr_parser.lexer.next();

         Ok(Statement::Call { address })
     }

     fn goto(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let line_number = match &self.current_token {
             Some(Token::Number(n)) => match u32::try_from(*n) {
                 Ok(n) => n,
                 Err(_) => {
                     return Err(Error {
                         kind: ErrorKind::ExpectedUnsigned,
                         line: self.lexer.current_line(),
                     });
                 }
             },
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedUnsigned,
                     line: self.lexer.current_line(),
                 });
             }
         };

         self.current_token = self.expr_parser.lexer.next();

         Ok(Statement::Goto { line_number })
     }

     fn gosub(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let line_number = match &self.current_token {
             Some(Token::Number(n)) => match u32::try_from(*n) {
                 Ok(n) => n,
                 Err(_) => {
                     return Err(Error {
                         kind: ErrorKind::ExpectedUnsigned,
                         line: self.lexer.current_line(),
                     });
                 }
             },
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedUnsigned,
                     line: self.lexer.current_line(),
                 });
             }
         };

         self.current_token = self.expr_parser.lexer.next();

         Ok(Statement::GoSub { line_number })
     }

     fn return_(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();

         Ok(Statement::Return)
     }

     fn if_(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let condition = match self.expr_parser.parse()? { //original self.expression()
             Some(expr) => expr,
             None => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedExpression,
                     line: self.lexer.current_line(),
                 });
             }
         };

         if self.current_token == Some(Token::Then) {
             self.current_token = self.expr_parser.lexer.next();
         }

         let then = Box::new(self.statement()?);

         let else_ = if self.current_token == Some(Token::Else) {
             self.current_token = self.expr_parser.lexer.next();
             let statement = self.statement()?;
             Some(Box::new(statement))
         } else {
             None
         };

         Ok(Statement::If {
             condition,
             then,
             else_,
         })
     }

     fn for_(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let variable = match &mut self.current_token {
             Some(Token::Identifier(v)) => mem::take(v),
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedIdentifier,
                     line: self.lexer.current_line(),
                 });
             }
         };

         self.current_token = self.expr_parser.lexer.next();
         if self.current_token != Some(Token::Equal) {
             return Err(Error {
                 kind: ErrorKind::UnexpectedToken,
                 line: self.lexer.current_line(),
             });
         }

         self.current_token = self.expr_parser.lexer.next();
         let from = match self.expr_parser.parse()? {
             Some(expr) => expr,
             None => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedExpression,
                     line: self.lexer.current_line(),
                 });
             }
         };

         if self.current_token != Some(Token::To) {
             return Err(Error {
                 kind: ErrorKind::UnexpectedToken,
                 line: self.lexer.current_line(),
             });
         }

         self.current_token = self.expr_parser.lexer.next();
         let to = match self.expr_parser.parse()? {
             Some(expr) => expr,
             None => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedExpression,
                     line: self.lexer.current_line(),
                 });
             }
         };

         let step = if self.current_token == Some(Token::Step) {
             self.current_token = self.expr_parser.lexer.next();
             match self.expr_parser.parse()? {
                 Some(expr) => Some(expr),
                 None => {
                     return Err(Error {
                         kind: ErrorKind::ExpectedExpression,
                         line: self.lexer.current_line(),
                     });
                 }
             }
         } else {
             None
         };

         Ok(Statement::For {
             variable,
             from,
             to,
             step,
         })
     }

     fn next(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let variable = match &mut self.current_token {
             Some(Token::Identifier(v)) => mem::take(v),
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedIdentifier,
                     line: self.lexer.current_line(),
                 });
             }
         };

         self.current_token = self.expr_parser.lexer.next();

         Ok(Statement::Next { variable })
     }

     fn end(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();

         Ok(Statement::End)
     }

     fn comment(&mut self) -> Result<Statement, Error> {
         match &mut self.current_token {
             Some(Token::Rem(s)) => {
                 let res = Ok(Statement::Rem {
                     content: mem::take(s),
                 });

                 self.current_token = self.expr_parser.lexer.next();

                 res
             }
             _ => {
                 unreachable!("We already checked for REM");
             }
         }
     }

     fn dim(&mut self) -> Result<Statement, Error> {
         self.current_token = self.expr_parser.lexer.next();
         let variable = match &mut self.current_token {
             Some(Token::Identifier(v)) => mem::take(v),
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedIdentifier,
                     line: self.lexer.current_line(),
                 });
             }
         };

         self.current_token = self.expr_parser.lexer.next();
         if self.current_token != Some(Token::LeftParen) {
             return Err(Error {
                 kind: ErrorKind::ExpectedLeftParen,
                 line: self.lexer.current_line(),
             });
         }

         self.current_token = self.expr_parser.lexer.next();
         let size = match &self.current_token {
             Some(Token::Number(n)) => match u32::try_from(*n) {
                 Ok(n) => n,
                 Err(_) => {
                     return Err(Error {
                         kind: ErrorKind::ExpectedUnsigned,
                         line: self.lexer.current_line(),
                     });
                 }
             },
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedUnsigned,
                     line: self.lexer.current_line(),
                 });
             }
         };

         if self.current_token != Some(Token::RightParen) {
             return Err(Error {
                 kind: ErrorKind::ExpectedRightParen,
                 line: self.lexer.current_line(),
             });
         }

         self.current_token = self.expr_parser.lexer.next();

         let length = if self.current_token == Some(Token::Star) {
             self.current_token = self.expr_parser.lexer.next();
             match &self.current_token {
                 Some(Token::Number(n)) => match u32::try_from(*n) {
                     Ok(n) => {
                         self.current_token = self.expr_parser.lexer.next();
                         Some(n)
                     }
                     Err(_) => {
                         return Err(Error {
                             kind: ErrorKind::ExpectedUnsigned,
                             line: self.lexer.current_line(),
                         });
                     }
                 },
                 _ => {
                     return Err(Error {
                         kind: ErrorKind::ExpectedUnsigned,
                         line: self.lexer.current_line(),
                     });
                 }
             }
         } else {
             None
         };

         Ok(Statement::Dim {
             variable,
             size,
             length,
         })
     }

     fn atomic_statement(&mut self) -> Result<Statement, Error> {
          println!("Atomic statement: {:?}", self.current_token);
         match self.current_token {
             Some(Token::Let | Token::Identifier(_)) => self.let_(),
             Some(Token::Print) => self.print(),
             Some(Token::Pause) => self.pause(),
             Some(Token::Input) => self.input(),
             Some(Token::Wait) => self.wait(),
             Some(Token::Goto) => self.goto(),
             Some(Token::For) => self.for_(),
             Some(Token::Next) => self.next(),
             Some(Token::End) => self.end(),
             Some(Token::Gosub) => self.gosub(),
             Some(Token::If) => self.if_(),
             Some(Token::Return) => self.return_(),
             Some(Token::Data) => self.data(),
             Some(Token::Read) => self.read(),
             Some(Token::Restore) => self.restore(),
             Some(Token::Poke) => self.poke(),
             Some(Token::Call) => self.call(),
             Some(Token::Dim) => self.dim(),
             Some(Token::Rem(_)) => self.comment(),
             _ => Err(Error {
                 kind: ErrorKind::ExpectedStatement,
                 line: self.lexer.current_line(),
             }),
         }
     }

     fn statement(&mut self) -> Result<Statement, Error> {
         //TODO: small vec optimization
         let mut statements = Vec::new();

         loop {
             let stmt = self.atomic_statement()?;

             statements.push(stmt);

             if self.current_token == Some(Token::Colon) {
                 self.current_token = self.expr_parser.lexer.next();
             } else {
                 break;
             }
         }

         Ok(if statements.len() == 1 {
             statements.remove(0)
         } else {
             Statement::Seq { statements }
         })
     }

     fn line(&mut self) -> Result<(u32, Statement), Error> {
         let line_number = match &self.current_token {
             Some(Token::Number(n)) => {
                 if let Ok(n) = u32::try_from(*n) {
                     n
                 } else {
                     return Err(Error {
                         kind: ErrorKind::ExpectedLineNumber,
                         line: self.lexer.current_line(),
                     });
                 }
             }
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedLineNumber,
                     line: self.lexer.current_line(),
                 })
             }
         };

         self.current_token = self.expr_parser.lexer.next();
         let statement = self.statement()?;

         match self.current_token {
             Some(Token::Newline) => {
                 self.current_token = self.expr_parser.lexer.next();
             }
             None => {}
             _ => {
                 return Err(Error {
                     kind: ErrorKind::ExpectedEndOfLine,
                     line: self.lexer.current_line(),
                 });
             }
         }

         Ok((line_number, statement))
     }

     fn program(&mut self) -> (Program, Vec<Error>) {
         let mut errors = Vec::new();
         let mut program = Program::new();

         self.current_token = self.expr_parser.lexer.next();

         while self.current_token.is_some() {
             match self.line() {
                 Ok((line_number, statement)) => {
                     program.add_line(line_number, statement);
                 }
                 Err(e) => {
                     errors.push(e);
                     self.current_token = self.expr_parser.lexer.next();

                     while self.current_token != Some(Token::Newline) {
                         self.current_token = self.expr_parser.lexer.next();
                     }
                 }
             }
         }

         (program, errors)
     }
}
