use crate::ast;
use crate::ast::Tag;
use crate::parse::KEYWORDS;

peg::parser! {
    pub grammar lang() for str {
        // Comments
        rule block_comment() = quiet!{"/*" (!"*/" [_])* "*/"}
        rule line_comment() = "//" (!"\n" [_])* ("\n" / ![_])

        // Optional space
        rule _  = quiet!{([' ' | '\t' | '\r'] / block_comment() / line_comment())*}
        rule __ = quiet!{([' ' | '\n' | '\t' | '\r'] / block_comment() / line_comment())*}

        // Required space
        rule spa() = quiet!{[' ' | '\t' | '\r']+} // required space without \n
        rule sep() = quiet!{[' ' | '\t' | '\r']*} ['\n' | ';'] quiet!{[' ' | '\t' | '\r' | '\n']*} // optional space with required \n or ;

        rule str() -> String
            = cs:$(quiet!{['_' | 'a' ..= 'z' | 'A' ..= 'Z']['a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_' ]*}) {?
                if KEYWORDS.contains(&cs) {
                    Err("non-keyword")
                } else {
                    Ok(cs.into())
                }
            }
        rule i64() -> i64
            = i:$(quiet!{(['1'..='9'])?['0'..='9']*}) {? i.parse::<i64>().or(Err("0..9")) }

        rule tag<T>(x: rule<T>) -> Tag<T>
        = s:position!() x:x() e:position!() { Tag::new(x, (s, e)) }

        rule tuple_colon<T, V>(x: rule<T>, y: rule<V>) -> (T, V)
            = x:x() _ ":" _ y:y() { (x, y) }

        rule list<T>(x: rule<T>) -> ast::_Vec<T>
            = s:position!() v:(x() ** (_ "," __)) (_ ",")? e:position!() {
                Tag::new(v, (s, e))
            }

        // Primitives
        rule ident() -> ast::_Ident
            = i:tag(<str()>) { i }

        rule _typ() -> ast::Type
            = "()" { ast::Type::Unit }
            / "i64" { ast::Type::I64 }
            / "bool" { ast::Type::Bool }
            / "fn" _ "(" _ args:list(<typ()>) _ ")" _ "->" _ ret:typ() {
                ast::Type::Fun(args, Box::new(ret))
            }
            / "fn" _ "(" _ args:list(<typ()>) _ ")" {
                let (_, e) = args.span();
                ast::Type::Fun(args, Box::new(Tag::new(ast::Type::Unit, (e, e))))
            }

        rule typ() -> ast::_Type = precedence!{
            s:position!() t:@ e:position!() { Tag::new(t, (s, e)) }
            --
            t:_typ() { t }
        }

        // Primary expressions
        #[cache_left_rec]
        rule primary() -> ast::Expr
            = i:i64() { ast::Expr::Int(i) }
            / "()" { ast::Expr::Unit }
            / "true" { ast::Expr::Bool(true) }
            / "false" { ast::Expr::Bool(false) }
            / "(" __ e:expr() __ ")" { e.into_inner() }
            / i:ident() { ast::Expr::Ident(i)   }

        // Expressions involving operators or evaluation order
        #[cache_left_rec]
        pub rule expr() -> ast::_Expr = precedence! {
            s:position!() t:@ e:position!() { Tag::boxed(t, (s, e)) }
            --
            "if" __ t:expr() __ "{" __ b:body() __ "}" __ "else" __ "{" __ e:body() __ "}"
            { ast::Expr::If(t, b, e) }
            "if" __ t:expr() __ "{" __ b:body() __ "}"
            { ast::Expr::If(t, b.clone(), Tag::boxed(ast::Expr::Unit, b.span())) }
            "println!(\"{}\"," __ e:expr() __ ")"
            { ast::Expr::Print(e) }
            --
            x:(@) __ s:position!() "==" e:position!() __ y:@ { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Eq, (s, e)), y) }
            x:(@) __ s:position!() "!=" e:position!() __ y:@ { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Ne, (s, e)), y) }
            --
            x:(@) __ s:position!() ">" e:position!() __ y:@ { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Gt, (s, e)), y) }
            x:(@) __ s:position!() ">=" e:position!() __ y:@  { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Gte, (s, e)), y) }
            x:(@) __ s:position!() "<" e:position!() __ y:@  { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Lt, (s, e)), y) }
            x:(@) __ s:position!() "<=" e:position!() __ y:@  { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Lte, (s, e)), y) }
            --
            x:(@) __ s:position!() "+" e:position!() __ y:@  { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Add, (s, e)), y) }
            x:(@) __ s:position!() "-" e:position!() __ y:@ { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Sub, (s, e)), y) }
            --
            x:(@) __ s:position!() "*" e:position!() __ y:@ { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Mul, (s, e)), y) }
            x:(@) __ s:position!() "/" e:position!() __ y:@ { ast::Expr::BinOp(x, Tag::new(ast::BinOp::Div, (s, e)), y) }
            --
            l:(@) _ "(" __ ts:list(<expr()>) __ ")" { ast::Expr::FunApp(l, ts) }
            --
            t:primary() { t }
        }


        #[cache_left_rec]
        pub rule body() -> ast::_Expr = precedence! {
            s:position!() t:@ e:position!() { Tag::boxed(t, (s, e)) }
            --
            // let: 4 variants (type annotation yes/no, continuation yes/no).
            // Missing continuation defaults to Unit.
             "let" spa() __ i:ident() __ "=" __ t:expr() _ ";" __  c:body()
              { ast::Expr::Let(i, None, t, c) }
             "let" spa() __ s:position!() i:ident() e:position!() __ "=" __ t:expr() _ ";" __
              { ast::Expr::Let(i, None, t, Tag::boxed(ast::Expr::Unit, (s, e))) }
             "let" spa() __ i:ident() _ ":" _ ty:typ() __  "=" __ t:expr() _ ";" __  c:body()
              { ast::Expr::Let(i, Some(ty), t, c) }
             "let" spa() __ s:position!() i:ident() e:position!()  _ ":" _ ty:typ() __ "=" __ t:expr() _ ";" __
              { ast::Expr::Let(i, Some(ty), t, Tag::boxed(ast::Expr::Unit, (s, e))) }
            // fn: 4 variants (continuation yes/no, return type yes/no).
            // Missing return type defaults to Unit.
            // Missing continuation defaults to Unit.
            "fn" spa() i:ident() _ "(" _ ts:list(<tuple_colon(<ident()>, <typ()>)>) _ ")" _ "->" _ ty:typ() __ "{" __ t:body() __ "}" _ "\n" __ c:body()
            { ast::Expr::FunDec(ast::FunSignature { name: i, params: ts, ret: ty, body: t }, c) }
            "fn" spa() i:ident() _ "(" _ ts:list(<tuple_colon(<ident()>, <typ()>)>) _ ")" _ "->" _ ty:typ() __ "{" __ t:body() __ "}"
            { ast::Expr::FunDec(ast::FunSignature { name: i.clone(), params: ts, ret: ty, body: t }, Tag::boxed(ast::Expr::Unit, i.span())) }
            "fn" spa() i:ident() _ "(" _ ts:list(<tuple_colon(<ident()>, <typ()>)>) _ ")" p:position!() _ "{" __ t:body() __ "}" _ "\n" __ c:body()
            { ast::Expr::FunDec(ast::FunSignature { name: i.clone(), params: ts.clone(), ret: Tag::new(ast::Type::Unit, (p, p + 1)), body: t }, c)}
            "fn" spa() i:ident() _ "(" _ ts:list(<tuple_colon(<ident()>, <typ()>)>) _ ")" p:position!() _ "{" __ t:body() __ "}"
            { ast::Expr::FunDec(ast::FunSignature { name: i.clone(), params: ts.clone(), ret: Tag::new(ast::Type::Unit, (p, p + 1)), body: t }, Tag::boxed(ast::Expr::Unit, i.span())) }
            --
            l:body() __ ";" __ r:@ { ast::Expr::Seq(l, r) }
            l:body() __ s:position!() ";" __ e:position!() { ast::Expr::Seq(l.clone(), Tag::boxed(ast::Expr::Unit, l.span())) }
            --
            e:expr() { e.into_inner() }
            s:position!() { ast::Expr::Unit } // Empty function bodies
        }


        // Program
        rule top() -> ast::_Top = precedence! {
            s:position!() t:@ e:position!() { Tag::new(t, (s, e)) }
            --
           "fn" spa() i:ident() _ "(" __ ts:list(<tuple_colon(<ident()>, <typ()>)>) __ ")" _ "->" _ ty:typ() __ "{" __ t:body() __ "}"
            { ast::Top::FunDec(ast::FunSignature { name: i, params: ts, ret: ty, body: t }) }
           "fn" spa() i:ident() _ "(" __ ts:list(<tuple_colon(<ident()>, <typ()>)>) __ ")" _ "{" __ t:body() __ "}"
            { ast::Top::FunDec(ast::FunSignature { name: i.clone(), params: ts, ret: Tag::new(ast::Type::Unit, i.span()), body: t }) }
        }

        pub rule program() -> ast::Program =
            __ ts:(top() ++ (sep() __)) __? { ts }

    }
}
