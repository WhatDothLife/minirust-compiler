mod env;
mod eq;

use crate::ast::{BinOp, Error, Expr, Program, Result, Tag, Top, Type, _Expr, _Type};
use crate::typing::eq::TypeEq;
use env::Environment;

pub fn type_check(program: &Program) -> Result<()> {
    let mut env = Environment::new();
    let mut main_found: Option<Tag<Top>> = None;

    for top in program {
        let Top::FunDec(id, params, ret, _) = top.inner();

        if id.inner() == "main" {
            if let Some(first) = main_found {
                return Err(Error::new("duplicated main function")
                    .label("first definition", first.span())
                    .label("second definition ", top.span()));
            }
            if !matches!(ret.inner(), Type::Unit) {
                return Err(Error::new("main function must return Unit")
                    .label(format!("got {}", ret.inner()), ret.span()));
            }
            if !params.inner().is_empty() {
                return Err(Error::new("main function cannot have arguments")
                    .label("unexpected arguments", params.span()));
            }
            main_found = Some(top.clone());
        }

        let formal_tys: Vec<_Type> = params.inner().iter().map(|(_, t)| t.clone()).collect();
        let fun_ty = Type::Fun(params.to(formal_tys), ret.clone());
        env.insert(id.inner().clone(), fun_ty);
    }

    if main_found.is_none() {
        return Err(Error::new("main function not found"));
    }

    for top in program {
        let Top::FunDec(_, params, ret, body) = top.inner();
        let mut local_env = env.clone();

        for (arg_name, arg_ty) in params.inner() {
            local_env.insert(arg_name.inner().clone(), arg_ty.inner().clone());
        }

        let body_ty = type_of_expr(body, &local_env)?;
        body_ty.assert_eq(ret)?;
    }

    Ok(())
}

fn type_of_expr(expr: &_Expr, env: &Environment) -> Result<_Type> {
    let ty = match expr.inner() {
        Expr::Unit => Type::Unit,
        Expr::Int(_) => Type::Int,

        Expr::Ident(id) => match env.lookup(id.inner()) {
            Some(t) => t.clone(),
            None => {
                return Err(Error::new(format!("undefined identifier '{}'", id.inner()))
                    .label("not found in scope", id.span()))
            }
        },

        Expr::BinOp(left, op, right) => {
            let l_ty = type_of_expr(left, env)?;
            let r_ty = type_of_expr(right, env)?;

            match op.inner() {
                BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::Gt
                | BinOp::Gte
                | BinOp::Lt
                | BinOp::Lte => {
                    l_ty.assert_eq(&expr.to(Type::Int))?;
                    r_ty.assert_eq(&expr.to(Type::Int))?;
                    if matches!(
                        op.inner(),
                        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div
                    ) {
                        Type::Int
                    } else {
                        Type::Bool
                    }
                }
                BinOp::Eq | BinOp::Neq => {
                    l_ty.assert_eq(&r_ty)?;
                    Type::Bool
                }
            }
        }

        Expr::Seq(first, second) => {
            type_of_expr(first, env)?;
            return type_of_expr(second, env);
        }

        Expr::If(cond, then_b, else_b) => {
            type_of_expr(cond, env)?.assert_eq(&expr.to(Type::Bool))?;

            let t_ty = type_of_expr(then_b, env)?;
            let e_ty = type_of_expr(else_b, env)?;

            t_ty.assert_eq(&e_ty)?;
            t_ty.into_inner()
        }

        Expr::FunApp(fun, args) => {
            let fun_ty = type_of_expr(fun, env)?;
            if let Type::Fun(formals, ret) = fun_ty.inner() {
                eq::len_eq(formals, args)?;

                for (actual, formal) in args.inner().iter().zip(formals.inner()) {
                    type_of_expr(actual, env)?.assert_eq(formal)?;
                }
                ret.inner().clone()
            } else {
                return Err(Error::new("expected function").label("not callable", fun.span()));
            }
        }

        Expr::Let(id, ann, val, rest) => {
            let val_ty = type_of_expr(val, env)?;
            if let Some(expected) = ann {
                val_ty.assert_eq(expected)?;
            }

            let mut local_env = env.clone();
            local_env.insert(id.inner().clone(), val_ty.into_inner());

            return type_of_expr(rest, &local_env);
        }

        Expr::FunDec(id, params, ret, body, rest) => {
            let mut env = env.clone();
            let formal_tys: Vec<_Type> = params.inner().iter().map(|(_, t)| t.clone()).collect();
            let fun_ty = Type::Fun(params.to(formal_tys), ret.clone());
            env.insert(id.inner().clone(), fun_ty);

            {
                let mut body_env = env.clone();
                for (p_id, p_ty) in params.inner() {
                    body_env.insert(p_id.inner().clone(), p_ty.inner().clone());
                }
                type_of_expr(body, &body_env)?.assert_eq(ret)?;
            }

            return type_of_expr(rest, &env);
        }

        Expr::Print(e) => {
            type_of_expr(e, env)?;
            Type::Unit
        }
    };

    Ok(expr.to(ty))
}
