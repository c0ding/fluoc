// Name mangling
// Name mangling is done on all functions and function calls

// i.e.
// Number proceeding modifier (i.e. "N", "P", "R") is length
// N for "name"
// `List_my::get` -> N7List_my_N3get
// `List::my_get` -> N4List_N6my_get

// Types are encoded like
// P for "parameter type"
// A for "argument types"
// R for "return type"
// my_func (int, int) -> bool
// N7my_func_R6V4bool_A15P5V3int_P5V3int

// Tuples
// t for "tuple type"
// V for "type value"
// my_func (int, int) -> (bool, bool)
// N7my_func _ P5 V3int _ P5 V3int _ R14 t11 V3int_V3int

// Operator Overloading
// convert the tokens into characters:
//  + `add`  int, int -> int OR int -> int (infix operator)
//  - `sub`  int, int -> int OR int -> int (infix operator)
//  * `mul`  int, int -> int
//  / `div`  int, int -> float
//  % `mod`  int, int -> int
// %% `mdd`  (MoDulo Divisible)          int, int -> bool
//  > `grt`  (GReater Than)              int, int -> bool
//  < `lst`  (LeSs Than)                 int, int -> bool
//  >= `get` (Greater than or Equal To)  int, int -> bool
//  <= `let` (Less than or Equal To)     int, int -> bool
//  == `eqt` (EQual To)                  int, int -> bool
//
// O for "overloaded value"
// overload + add_ints(int, int) -> int
// O3add_N8add_ints_R5V3int_A15P5V3int_P5V3int

use crate::lexer::TokenType;
use crate::logger::ErrorOrVec;
use crate::parser::ast;

use std::rc::Rc;

#[derive(PartialEq, Debug)]
pub enum NodeChild {
    RawString(String),
    Children(Vec<Node>),
}

#[derive(PartialEq, Debug)]
pub struct Node {
    descriptor: char,
    child: NodeChild,
}

impl Node {
    pub fn demangle(input: String) -> Node {
        Node {
            descriptor: '$',
            child: NodeChild::demangle(input),
        }
    }

    fn from_type<'a>(
        namespace: &ast::Namespace,
        arguments: &ast_typecheck::ArgumentsTypeCheck<'a>,
        ret_type: &ast_typecheck::TypeCheckType<'a>,
    ) -> Node {
        let mut val = Vec::new();
        val.append(&mut NodeChild::from_namespace(namespace));
        Node {
            descriptor: '$',
            child: NodeChild::Children(val),
        }
    }
}

impl NodeChild {
    fn from_name<'a>(name: ast::NameID<'a>) -> NodeChild {
        NodeChild::RawString(name.value.to_string())
    }

    fn from_namespace<'a>(namespace: &ast::Namespace<'a>) -> Vec<Node> {
        let mut nodes = Vec::new();
        for name in &namespace.scopes {
            nodes.push(Node {
                descriptor: 'N',
                child: Self::from_name(*name),
            })
        }

        nodes
    }

    fn from_type<'a, 'b>(
        type_val: &ast_typecheck::TypeCheckTypeType<'a>,
        context: &'b context::TypeCheckSymbTab<'a>,
    ) -> Result<Node, ErrorOrVec<'a>> {
        Ok(match type_val {
            ast_typecheck::TypeCheckTypeType::SingleType(type_val) => Node {
                descriptor: 'V',
                child: NodeChild::RawString(type_val.mangle()),
            },
            ast_typecheck::TypeCheckTypeType::TupleType(types) => {
                let tuple_items = types
                    .types
                    .iter()
                    .map(|type_val| NodeChild::from_type(&type_val.value, context))
                    .collect::<Result<Vec<_>, _>>()?;
                Node {
                    descriptor: 't',
                    child: NodeChild::Children(tuple_items),
                }
            }
            ast_typecheck::TypeCheckTypeType::CustomType(namespace, _) => Self::from_type(
                &context
                    .get_type(Rc::clone(namespace))?
                    .unwrap_type_ref()
                    .0
                    .value,
                context,
            )?,
            _ => panic!("Mangling not implemented for this type yet!"),
        })
    }

    fn demangle(input: String) -> NodeChild {
        let mut characters = input.chars().peekable();
        let mut nodes = Vec::new();
        loop {
            let first = match characters.next() {
                Some(val) => val,
                None => break,
            };
            let mut next_char = *(match characters.peek() {
                Some(val) => val,
                None => break,
            });

            if '0' <= next_char && next_char <= '9' {
                let mut modifier_amount = String::new();
                let mut first_loop = true;
                while '0' <= next_char && next_char <= '9' {
                    if !first_loop {
                        characters.next();
                    } else {
                        first_loop = false;
                    }

                    modifier_amount.push(next_char);
                    next_char = *characters.peek().unwrap();
                }

                let length = modifier_amount[1..].parse::<usize>().unwrap();
                let mut descriptor_string = String::with_capacity(length);
                for _ in 0..length {
                    descriptor_string.push(characters.next().unwrap());
                }

                nodes.push(Node {
                    descriptor: first,
                    child: NodeChild::demangle(descriptor_string),
                })
            } else {
                return NodeChild::RawString(input);
            }

            characters.next();
        }

        NodeChild::Children(nodes)
    }
}

#[cfg(test)]
mod demangle_tests {
    use super::*;

    #[test]
    fn simple_test() {
        assert_eq!(
            Node::demangle("N5entry".to_string()),
            Node {
                descriptor: '$',
                child: NodeChild::Children(vec![Node {
                    descriptor: 'N',
                    child: NodeChild::RawString("entry".to_string()),
                }])
            }
        )
    }

    #[test]
    fn complex_test() {
        assert_eq!(
            Node::demangle("N7my_func_P5V3int_P5V3int_R14t11V3int_V3int".to_string()),
            Node {
                descriptor: '$',
                child: NodeChild::Children(vec![
                    Node {
                        descriptor: 'N',
                        child: NodeChild::RawString("my_func".to_string())
                    },
                    Node {
                        descriptor: 'P',
                        child: NodeChild::Children(vec![Node {
                            descriptor: 'V',
                            child: NodeChild::RawString("int".to_string())
                        }])
                    },
                    Node {
                        descriptor: 'P',
                        child: NodeChild::Children(vec![Node {
                            descriptor: 'V',
                            child: NodeChild::RawString("int".to_string())
                        }])
                    },
                    Node {
                        descriptor: 'R',
                        child: NodeChild::Children(vec![Node {
                            descriptor: 't',
                            child: NodeChild::Children(vec![
                                Node {
                                    descriptor: 'V',
                                    child: NodeChild::RawString("int".to_string())
                                },
                                Node {
                                    descriptor: 'V',
                                    child: NodeChild::RawString("int".to_string())
                                }
                            ])
                        }])
                    }
                ])
            }
        )
    }
}

pub(crate) fn gen_mangled_args<'a, 'b>(
    types: &[&ast_typecheck::TypeCheckType<'a>],
    context: &'b context::TypeCheckSymbTab<'a>,
) -> Result<String, ErrorOrVec<'a>> {
    let mangled_args = types
        .into_iter()
        .map(|arg_type| {
            let arg_mangled = arg_type.mangle(context)?;
            Ok(format!("P{}{}", arg_mangled.len(), arg_mangled))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("_");
    Ok(format!("_A{}{}", mangled_args.len(), mangled_args))
}

impl TokenType {
    pub(crate) fn mangle(&self) -> String {
        let mangled_op = match self {
            TokenType::SUB => "sub",
            TokenType::ADD => "add",
            TokenType::DIV => "div",
            TokenType::DMOD => "mdd",
            TokenType::MOD => "mod",
            TokenType::MUL => "mul",
            TokenType::GT => "grt",
            TokenType::LT => "lst",
            TokenType::GE => "get",
            TokenType::LE => "let",
            TokenType::EQ => "eqt",
            _ => panic!("{} cannot be mangled", self),
        };

        format!("O{}{}_", mangled_op.len(), mangled_op)
    }
}

impl<'a> ast::Namespace<'a> {
    /// Mangle function name without types
    /// Mangles name and returns as string
    pub(crate) fn mangle(&self) -> String {
        self.scopes
            .iter()
            .map(|name| {
                let name_mangled = name.mangle();
                format!("N{}{}", name_mangled.len(), name_mangled)
            })
            .collect::<Vec<_>>()
            .join("_")
    }

    /// Mangle operator overload function
    pub(crate) fn mangle_overload<'b>(
        &self,
        types: &[&ast_typecheck::TypeCheckType<'a>],
        return_type: &ast_typecheck::TypeCheckType<'a>,
        token: TokenType,
        context: &'b context::TypeCheckSymbTab<'a>,
    ) -> Result<String, ErrorOrVec<'a>> {
        let mut mangled = token.mangle();
        mangled += &self.mangle()[..];

        mangled += &gen_mangled_args(types, context)?[..];

        let ret_type_mangled = return_type.mangle(context)?;
        mangled += &format!("_R{}{}", ret_type_mangled.len(), ret_type_mangled)[..];

        Ok(mangled)
    }

    /// Mangle function name with types
    /// Mangles name and returns as string
    pub(crate) fn mangle_types<'b>(
        &self,
        types: &[&ast_typecheck::TypeCheckType<'a>],
        return_type: &ast_typecheck::TypeCheckType<'a>,
        context: &'b context::TypeCheckSymbTab<'a>,
    ) -> Result<String, ErrorOrVec<'a>> {
        let mut mangled = self.mangle();
        mangled += &gen_mangled_args(types, context)?[..];

        let ret_type_mangled = return_type.mangle(context)?;
        mangled += &format!("_R{}{}", ret_type_mangled.len(), ret_type_mangled)[..];
        Ok(mangled)
    }
}

impl<'a> ast_typecheck::TypeCheckType<'a> {
    /// Mangle typecheck type
    pub(crate) fn mangle<'b>(
        &self,
        context: &'b context::TypeCheckSymbTab<'a>,
    ) -> Result<String, ErrorOrVec<'a>> {
        Ok(match &self.value {
            ast_typecheck::TypeCheckTypeType::SingleType(val) => {
                let mangled = val.mangle();
                format!("V{}{}", mangled.len(), mangled)
            }
            ast_typecheck::TypeCheckTypeType::CustomType(val, _) => context
                .get_type(Rc::clone(val))?
                .unwrap_type_ref()
                .0
                .mangle(context)?,
            ast_typecheck::TypeCheckTypeType::TupleType(val) => {
                let tuple_items = val
                    .types
                    .iter()
                    .map(|type_val| {
                        let mangled = type_val.mangle(context)?;
                        Ok(format!("{}{}", mangled.len(), mangled))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .join("_");
                format!("t{}{}", tuple_items.len(), tuple_items)
            }
            _ => panic!("No name mangling for {}", self),
        })
    }
}

impl<'a> ast::NameID<'a> {
    /// Mangle function name + its types
    pub(crate) fn mangle(&self) -> &'a str {
        self.value
    }
}
