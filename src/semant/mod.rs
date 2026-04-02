mod env;
mod eq;

use crate::ast::{self, BinOp, Error, FunSignature, Result, Tag, Top, Type, _Expr, _Type};
use crate::ir::frame::Frame;
use crate::ir::{self, translate, Fragment};

use env::{Binding, Entry, Environment};

// Helper to insert a function signature into the environment.
fn declare_fun<T>(env: &mut Environment, sig: &FunSignature, span_tag: &Tag<T>) -> Result<()> {
    if env.lookup(sig.name.inner()).is_some() {
        return Err(Error::new(format!(
            "function '{}' is defined multiple times",
            sig.name.inner()
        ))
        .label("redefined here", sig.name.span()));
    }

    let formal_tys: Vec<_Type> = sig.params.inner().iter().map(|(_, t)| t.clone()).collect();
    let fun_ty = Type::Fun(sig.params.to(formal_tys), Box::new(sig.ret.clone()));
    let label = translate::get_label(sig.name.inner());

    env.insert_global(
        sig.name.inner().clone(),
        Entry {
            ty: span_tag.to(fun_ty),
            binding: Binding::Fun(label),
        },
    );

    Ok(())
}

fn validate_main(main_top: &Tag<Top>) -> Result<()> {
    let Top::FunDec(sig) = main_top.inner();

    if !matches!(sig.ret.inner(), Type::Unit) {
        return Err(Error::new("main function must return Unit")
            .label(format!("got {}", sig.ret.inner()), sig.ret.span()));
    }

    if !sig.params.inner().is_empty() {
        return Err(Error::new("main function cannot have arguments")
            .label("unexpected arguments", sig.params.span()));
    }

    Ok(())
}

fn collect_globals(program: &ast::Program, env: &mut Environment) -> Result<()> {
    let mut main_found: Option<&Tag<Top>> = None;

    for top in program {
        let Top::FunDec(sig) = top.inner();

        if sig.name.inner() == "main" {
            if main_found.is_some() {
                return Err(Error::new("multiple main functions"));
            }
            main_found = Some(top);
        }

        declare_fun(env, sig, top)?;
    }

    let Some(main_tag) = main_found else {
        return Err(Error::new("main function not found"));
    };

    validate_main(&main_tag)?;

    Ok(())
}

fn collect_locals(expr: &_Expr, env: &mut Environment) -> Result<()> {
    match expr.inner() {
        ast::Expr::FunDec(sig, rest) => {
            declare_fun(env, sig, expr)?;
            collect_locals(rest, env)
        }
        ast::Expr::Let(_, _, _, rest) => collect_locals(rest, env),
        ast::Expr::Seq(a, b) => {
            if let ast::Expr::FunDec(sig, _) = a.inner() {
                declare_fun(env, sig, a)?;
            }
            collect_locals(b, env)
        }
        _ => Ok(()),
    }
}

fn emit_fun(
    sig: &FunSignature,
    env: &Environment,
    fragments: &mut Vec<Fragment>,
) -> Result<Fragment> {
    let entry = env.lookup(sig.name.inner()).unwrap();
    let label = match &entry.binding {
        Binding::Fun(l) => l.clone(),
        _ => unreachable!(),
    };

    let param_names = sig
        .params
        .inner()
        .iter()
        .map(|(n, _)| n.inner().to_owned())
        .collect();
    let mut ctx = translate::FunContext::entry(label, param_names);
    let mut body_env = env.for_body();

    for (name, access) in &ctx.params {
        let (_, ty) = sig
            .params
            .inner()
            .iter()
            .find(|(p_id, _)| p_id.inner() == name)
            .unwrap();

        body_env.insert_local(
            name.to_owned(),
            Entry {
                ty: ty.clone(),
                binding: Binding::Var(access.clone()),
            },
        );
    }

    collect_locals(&sig.body, &mut body_env)?;

    let body_info = check_expr(&sig.body, &body_env, &mut ctx.frame, fragments)?;
    body_info.ty.assert_eq(&sig.ret)?;

    Ok(ctx.exit(body_info.ir))
}

pub fn check(program: &ast::Program) -> Result<ir::Program> {
    let mut env = Environment::new();
    collect_globals(program, &mut env)?;

    let mut fragments = Vec::new();

    for top in program {
        let Top::FunDec(sig) = top.inner();
        let fragment = emit_fun(sig, &env, &mut fragments)?;
        fragments.push(fragment);
    }

    Ok(ir::Program { fragments })
}

fn check_expr(
    expr: &_Expr,
    env: &Environment,
    frame: &mut Frame,
    fragments: &mut Vec<Fragment>,
) -> Result<TypedExpr> {
    match expr.inner() {
        ast::Expr::Unit => Ok(TypedExpr {
            ir: translate::unit(),
            ty: expr.to(Type::Unit),
        }),

        ast::Expr::Bool(b) => Ok(TypedExpr {
            ir: translate::int(*b as i64),
            ty: expr.to(Type::Bool),
        }),

        ast::Expr::Int(i) => Ok(TypedExpr {
            ir: translate::int(*i),
            ty: expr.to(Type::I64),
        }),

        ast::Expr::Ident(id) => match env.lookup(id.inner()) {
            Some(Entry {
                ty,
                binding: Binding::Var(acc),
            }) => Ok(TypedExpr {
                ir: translate::var(acc),
                ty: ty.at(id),
            }),
            Some(Entry {
                ty,
                binding: Binding::Fun(label),
            }) => Ok(TypedExpr {
                ir: translate::fun(label),
                ty: ty.at(id),
            }),
            None => Err(Error::new(format!("undefined identifier '{}'", id.inner()))
                .label("not found in scope", id.span())),
        },

        ast::Expr::BinOp(left, op, right) => {
            let l = check_expr(left, env, frame, fragments)?;
            let r = check_expr(right, env, frame, fragments)?;
            l.ty.assert_eq(&r.ty)?;

            let (ir, res_ty) = match op.inner() {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                    l.ty.assert_eq_raw(&Type::I64)?;
                    let ir = match op.inner() {
                        BinOp::Add => translate::op_add(l.ir, r.ir),
                        BinOp::Sub => translate::op_sub(l.ir, r.ir),
                        BinOp::Mul => translate::op_mul(l.ir, r.ir),
                        _ => translate::op_div(l.ir, r.ir),
                    };
                    (ir, Type::I64)
                }
                BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => {
                    l.ty.assert_eq_raw(&Type::I64)?;
                    let ir = match op.inner() {
                        BinOp::Gt => translate::op_gt(l.ir, r.ir),
                        BinOp::Gte => translate::op_ge(l.ir, r.ir),
                        BinOp::Lt => translate::op_lt(l.ir, r.ir),
                        _ => translate::op_le(l.ir, r.ir),
                    };
                    (ir, Type::Bool)
                }
                BinOp::Eq => (translate::op_eq(l.ir, r.ir), Type::Bool),
                BinOp::Ne => (translate::op_ne(l.ir, r.ir), Type::Bool),
            };

            Ok(TypedExpr {
                ir,
                ty: expr.to(res_ty),
            })
        }

        ast::Expr::Seq(first, second) => {
            let first_info = check_expr(first, env, frame, fragments)?;
            let second_info = check_expr(second, env, frame, fragments)?;

            Ok(TypedExpr {
                ir: translate::chain_expr(first_info.ir, second_info.ir),
                ty: second_info.ty,
            })
        }

        ast::Expr::If(cond, then_b, else_b) => {
            let cond_info = check_expr(cond, env, frame, fragments)?;
            cond_info.ty.assert_eq_raw(&Type::Bool)?;

            let then_info = check_expr(then_b, env, frame, fragments)?;
            let else_info = check_expr(else_b, env, frame, fragments)?;
            then_info.ty.assert_eq(&else_info.ty)?;

            Ok(TypedExpr {
                ir: translate::if_else(cond_info.ir, then_info.ir, else_info.ir),
                ty: then_info.ty.at(expr),
            })
        }

        ast::Expr::FunApp(fun, args) => {
            let fun_info = check_expr(fun, env, frame, fragments)?;

            if let Type::Fun(formals, ret) = fun_info.ty.inner() {
                eq::len_eq(&formals, args)?;

                let mut arg_irs = Vec::with_capacity(args.inner().len());
                for (actual_ast, formal_ty) in args.inner().iter().zip(formals.inner()) {
                    let arg_info = check_expr(actual_ast, env, frame, fragments)?;
                    arg_info.ty.assert_eq(formal_ty)?;
                    arg_irs.push(arg_info.ir);
                }

                Ok(TypedExpr {
                    ir: translate::fun_app(fun_info.ir, arg_irs),
                    ty: (**ret).clone().at(expr),
                })
            } else {
                Err(Error::new("expected function").label("not callable", fun_info.ty.span()))
            }
        }

        ast::Expr::Let(id, ann, rhs, rest) => {
            let rhs_info = check_expr(rhs, env, frame, fragments)?;
            if let Some(expected) = ann {
                rhs_info.ty.assert_eq(expected)?;
            }

            let (assign, access) = translate::declare_var(rhs_info.ir, frame);

            let local_env = env.with_local(
                id.inner().clone(),
                Entry {
                    ty: rhs_info.ty,
                    binding: Binding::Var(access),
                },
            );

            let rest_info = check_expr(rest, &local_env, frame, fragments)?;

            Ok(TypedExpr {
                ir: translate::chain_expr(assign, rest_info.ir),
                ty: rest_info.ty,
            })
        }

        ast::Expr::FunDec(sig, rest) => {
            let fragment = emit_fun(sig, env, fragments)?;
            fragments.push(fragment);

            check_expr(rest, env, frame, fragments)
        }

        ast::Expr::Print(e) => {
            let e_info = check_expr(e, env, frame, fragments)?;
            Ok(TypedExpr {
                ir: translate::print_int(e_info.ir),
                ty: expr.to(Type::Unit),
            })
        }
    }
}

/// Represents a typed expression, pairing its IR representation with its type.
pub struct TypedExpr {
    pub ir: translate::Expr,
    pub ty: Tag<Type>,
}
