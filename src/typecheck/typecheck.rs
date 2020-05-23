use crate::core::generate_symbtab;
use crate::helpers;
use crate::logger::logger::{Error, Logger};
use crate::parser::parser;
use crate::parser::parser::Parser;
use crate::typecheck::ast_typecheck;

use std::cell::RefCell;
use std::path;
use std::rc::Rc;

/// Typecheck object
pub struct TypeCheckModule<'a> {
    pub parser: Parser<'a>,
    pub symtab: ast_typecheck::TypeCheckSymbTab<'a>,
}

impl<'a> TypeCheckModule<'a> {
    /// Return new module object.
    ///
    /// Arguments
    /// * `filename` - the filename of the file to read
    pub fn new(
        filename: &'a path::Path,
        file_contents: &'a str,
        logger: Rc<RefCell<Logger<'a>>>,
        first: bool,
    ) -> Result<TypeCheckModule<'a>, Vec<Error<'a>>> {
        let new_ref = Rc::clone(&logger);
        let mut p = parser::Parser::new(filename, file_contents, logger);
        p.initialize_expr();
        Ok(TypeCheckModule {
            parser: p,
            symtab: if first {
                generate_symbtab(new_ref)?
            } else {
                ast_typecheck::TypeCheckSymbTab::new()
            },
        })
    }

    pub fn type_check(&mut self) -> Result<(), Vec<Error<'a>>> {
        self.parser.parse()?;

        // Do type checking
        match (self.parser.ast.as_mut().unwrap() as &mut dyn ast_typecheck::TypeCheck)
            .type_check(None, &mut self.symtab)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(helpers::get_high_priority(e.as_vec())),
        }
    }

    pub fn get_symbols(mut self) -> Result<ast_typecheck::TypeCheckSymbTab<'a>, Vec<Error<'a>>> {
        self.parser.parse()?;

        // Do type checking
        match (self.parser.ast.as_mut().unwrap() as &mut dyn ast_typecheck::TypeCheck)
            .type_check(None, &mut self.symtab)
        {
            Ok(_) => Ok(self.symtab), // Return symbol table here
            Err(e) => Err(helpers::get_high_priority(e.as_vec())),
        }
    }
}
