use super::annotation::*;
use super::unifier::Substitutions;

use crate::helpers::Pos;
use crate::logger::{ErrorAnnotation, ErrorDisplayType, ErrorType, ErrorValue};
use crate::parser::ast::LiteralType;

use std::rc::Rc;

impl AnnotationType {
    fn sub(&mut self, solved_constraints: &Substitutions) -> Result<(), ErrorValue> {
        match self {
            AnnotationType::Type(_, _) | AnnotationType::Never(_) => {}
            AnnotationType::Tuple(tys, _) => {
                for ty in Rc::make_mut(tys).iter_mut() {
                    ty.sub(solved_constraints)?;
                }
            }
            AnnotationType::Function(args_ty, ret_ty, _) => {
                for ty in Rc::make_mut(args_ty).iter_mut() {
                    ty.sub(solved_constraints)?;
                }
                Rc::make_mut(ret_ty).sub(solved_constraints)?;
            }
            // Important part!
            AnnotationType::Infer(infer_num, pos) => {
                match solved_constraints.subs.get(&infer_num) {
                    Some(ty) => {
                        *self = ty.clone();
                    }
                    None => {
                        // We cannot infer this type, error
                        return Err(cannot_infer_err(*pos));
                    }
                }
            }
        }

        Ok(())
    }
}

impl TypedBinder {
    fn substitute(&mut self, solved_constraints: &Substitutions) -> Result<(), ErrorValue> {
        self.ty.sub(solved_constraints)?;
        Ok(())
    }
}

impl TypedBlock {
    fn substitute(&mut self, solved_constraints: &Substitutions) -> Result<(), ErrorValue> {
        self.ty.sub(solved_constraints)?;
        for stmt in self.stmts.iter_mut() {
            stmt.substitute(solved_constraints)?;
        }
        Ok(())
    }
}

impl TypedExpr {
    fn substitute(&mut self, solved_constraints: &Substitutions) -> Result<(), ErrorValue> {
        match &mut self.expr {
            TypedExprEnum::Tuple(tup) => {
                tup.ty.sub(solved_constraints)?;
                for ty in tup.exprs.iter_mut() {
                    ty.substitute(solved_constraints)?;
                }
            }
            TypedExprEnum::Block(block) => {
                block.substitute(solved_constraints)?;
            }
            TypedExprEnum::Is(is) => {
                is.ty.sub(solved_constraints)?;
                is.expr.substitute(solved_constraints)?;
            }
            TypedExprEnum::RefID(ref_id) => {
                ref_id.ty.sub(solved_constraints)?;
            }
            TypedExprEnum::Yield(yie) => {
                yie.ty.sub(solved_constraints)?;
                yie.expr.substitute(solved_constraints)?;
            }
            TypedExprEnum::Return(ret) => {
                ret.ty.sub(solved_constraints)?;
                ret.expr.substitute(solved_constraints)?;
            }
            TypedExprEnum::Literal(literal) => {
                literal.ty.sub(solved_constraints)?;
                match literal.ty.is_primitive() {
                    Some(prim) => match (prim, literal.value.literal_type) {
                        (Prim::Bool, LiteralType::Bool) => {}
                        (Prim::I16, LiteralType::Number) => {}
                        (Prim::I32, LiteralType::Number) => {}
                        (Prim::I64, LiteralType::Number) => {}

                        (_, _) => return Err(bad_literal(&literal.ty, self.pos)),
                    },
                    None => return Err(bad_literal(&literal.ty, self.pos)),
                }
            }
            TypedExprEnum::Function(func) => {
                func.ty.sub(solved_constraints)?;
                func.block.substitute(solved_constraints)?;
            }
            TypedExprEnum::FunctionCall(call) => {
                call.ty.sub(solved_constraints)?;
                call.func_ty.sub(solved_constraints)?;
                for ty in call.arguments.iter_mut() {
                    ty.substitute(solved_constraints)?;
                }
            }
            TypedExprEnum::VariableAssign(var) => {
                var.binder.substitute(solved_constraints)?;
                var.expr.substitute(solved_constraints)?;
            }
            TypedExprEnum::VariableAssignDeclaration(var) => {
                var.binder.substitute(solved_constraints)?;
                var.expr.substitute(solved_constraints)?;
            }
        }

        Ok(())
    }
}

impl TypedStmt {
    fn substitute(&mut self, solved_constraints: &Substitutions) -> Result<(), ErrorValue> {
        match &mut self.stmt {
            TypedStmtEnum::Tag(_) => {}
            TypedStmtEnum::Expression(expr) => expr.substitute(solved_constraints)?,
            TypedStmtEnum::VariableDeclaration(_) => unimplemented!(),
        }

        Ok(())
    }
}

pub fn substitute(
    stmts: &mut [TypedStmt],
    solved_constraints: Substitutions,
) -> Result<(), Vec<ErrorValue>> {
    let mut errors = Vec::new();

    for stmt in stmts.iter_mut() {
        match stmt.substitute(&solved_constraints) {
            Ok(_) => {}
            Err(err) => errors.push(err),
        }
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}

fn bad_literal(ty: &AnnotationType, pos: Pos) -> ErrorValue {
    ErrorValue::new(
        "invalid literal type".to_string(),
        ErrorType::TypeMismatch,
        pos,
        ErrorDisplayType::Error,
        vec![
            ErrorAnnotation::new(
                Some(format!("invalid type `{}`", ty)),
                ty.pos(),
                ErrorDisplayType::Error,
            ),
            ErrorAnnotation::new(
                Some("for this literal".to_string()),
                pos,
                ErrorDisplayType::Info,
            ),
        ],
    )
}

fn cannot_infer_err(pos: Pos) -> ErrorValue {
    ErrorValue::new(
        "cannot infer type".to_string(),
        ErrorType::TypeMismatch,
        pos,
        ErrorDisplayType::Error,
        vec![
            ErrorAnnotation::new(
                Some("type found here".to_string()),
                pos,
                ErrorDisplayType::Error,
            ),
            ErrorAnnotation::new(
                Some("help: use `is` operator to annotate type".to_string()),
                pos,
                ErrorDisplayType::Info,
            ),
        ],
    )
}
