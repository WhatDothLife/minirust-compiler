use crate::ast;
use crate::parse::KEYWORDS;
use crate::ast::Tag;

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

        // Types
        rule _typ() -> ast::Type
            = "()" { ast::Type::Unit }
            / "i64"  { ast::Type::Int }


        rule typ() -> ast::_Type = precedence!{
            s:position!() t:@ e:position!() { Tag::new(t, (s, e)) }
            --
            s:position!() "()" e:position!() _ "->" _ c:@ { ast::Type::Fun(Tag::new(vec!(), (s, e)), c)}
            --
            t:_typ() { t }
        }

        // Atomic terms 
        #[cache_left_rec]
        rule atom() -> ast::Term
            = i:i64() { ast::Term::Int(i) }
            / "()" { ast::Term::Unit }
            / "(" __ t:term() __ ")" { t.into_inner() }
            / i:ident() { ast::Term::Var(i)   }

        // Terms involving operators or evaluation order
        #[cache_left_rec]
        pub rule term() -> ast::_Term = precedence! {
            s:position!() t:@ e:position!() { Tag::new(t, (s, e)) }
            --
            x:(@) __ s:position!() "==" e:position!() __ y:@ { ast::Term::BinOp(x, Tag::new(ast::BinOp::Eq, (s, e)), y) }
            x:(@) __ s:position!() "!=" e:position!() __ y:@ { ast::Term::BinOp(x, Tag::new(ast::BinOp::Neq, (s, e)), y) }
            --
            x:(@) __ s:position!() ">" e:position!() __ y:@ { ast::Term::BinOp(x, Tag::new(ast::BinOp::Gt, (s, e)), y) }
            x:(@) __ s:position!() ">=" e:position!() __ y:@  { ast::Term::BinOp(x, Tag::new(ast::BinOp::Gte, (s, e)), y) }
            x:(@) __ s:position!() "<" e:position!() __ y:@  { ast::Term::BinOp(x, Tag::new(ast::BinOp::Lt, (s, e)), y) }
            x:(@) __ s:position!() "<=" e:position!() __ y:@  { ast::Term::BinOp(x, Tag::new(ast::BinOp::Lte, (s, e)), y) }
            --
            x:(@) __ s:position!() "+" e:position!() __ y:@  { ast::Term::BinOp(x, Tag::new(ast::BinOp::Add, (s, e)), y) }
            x:(@) __ s:position!() "-" e:position!() __ y:@ { ast::Term::BinOp(x, Tag::new(ast::BinOp::Sub, (s, e)), y) }
            --
            x:(@) __ s:position!() "*" e:position!() __ y:@ { ast::Term::BinOp(x, Tag::new(ast::BinOp::Mul, (s, e)), y) }
            x:(@) __ s:position!() "/" e:position!() __ y:@ { ast::Term::BinOp(x, Tag::new(ast::BinOp::Div, (s, e)), y) }
            --
            l:(@) _ "(" __ ts:list(<term()>) __ ")" { ast::Term::FunApp(l, ts) }
            --
            t:atom() { t }
        }

        #[cache_left_rec]
        pub rule seq() -> ast::_Term = precedence! {
            s:position!() t:@ e:position!() { Tag::new(t, (s, e)) }
            --
            // let: 4 variants (type annotation yes/no, continuation yes/no).
            // Missing continuation defaults to Unit.
             "let" spa() __ i:ident() __ "=" __ t:term() _ ";" __  c:seq()
              { ast::Term::Let(i, None, t, c) }
             "let" spa() __ s:position!() i:ident() e:position!() __ "=" __ t:term() _ ";" __
              { ast::Term::Let(i, None, t, Tag::new(ast::Term::Unit, (s, e))) }
             "let" spa() __ i:ident() _ ":" _ ty:typ() __  "=" __ t:term() _ ";" __  c:seq()
              { ast::Term::Let(i, Some(ty), t, c) }
             "let" spa() __ s:position!() i:ident() e:position!()  _ ":" _ ty:typ() __ "=" __ t:term() _ ";" __
              { ast::Term::Let(i, Some(ty), t, Tag::new(ast::Term::Unit, (s, e))) }
            // fn: 4 variants (continuation yes/no, return type yes/no).
            // Missing return type defaults to Unit.
            // Missing continuation defaults to Unit.
             "fn" spa() i:ident() _ "(" _ ts:list(<tuple_colon(<ident()>, <typ()>)>) _ ")" _ "->" _ ty:typ() __ "{" __ t:seq() __ "}" _ "\n" __ c:seq()
              { ast::Term::FunDec(i, ts, ty, t, c) }
             "fn" spa() i:ident() _ "(" _ ts:list(<tuple_colon(<ident()>, <typ()>)>) _ ")" _ "->" _ ty:typ() __ "{" __ t:seq() __ "}"
              { ast::Term::FunDec(i.clone(), ts, ty, t, Tag::new(ast::Term::Unit, i.span)) }
             "fn" spa() i:ident() _ "(" _ ts:list(<tuple_colon(<ident()>, <typ()>)>) _ ")" _ "{" __ t:seq() __ "}" _ "\n" __ c:seq()
              { ast::Term::FunDec(i.clone(), ts, Tag::new(ast::Type::Unit, i.span), t, c) }
             "fn" spa() i:ident() _ "(" _ ts:list(<tuple_colon(<ident()>, <typ()>)>) _ ")" _ "{" __ t:seq() __ "}"
              { ast::Term::FunDec(i.clone(), ts, Tag::new(ast::Type::Unit, i.span), t, Tag::new(ast::Term::Unit, i.span)) }
            
            --
            l:(@) _ ";" __ r:@ { ast::Term::Seq(l, r) }
            l:(@) _ s:position!() ";" __ e:position!() { ast::Term::Seq(l.clone(), Tag::new(ast::Term::Unit, l.span)) }
            --
            e:term() { e.into_inner() }
        }


        // Program
        rule top() -> ast::_Top = precedence! {
            s:position!() t:@ e:position!() { Tag::new(t, (s, e)) }
            --
            "fn" spa() i:ident() _ "(" __ ts:list(<tuple_colon(<ident()>, <typ()>)>) __ ")" _ "->" _ ty:typ() __ "{" __ t:seq() __ "}"
            { ast::Top::FunDec(i, ts, ty, t) }

            "fn" spa() i:ident() _ "(" __ ts:list(<tuple_colon(<ident()>, <typ()>)>) __ ")" _ "{" __ t:seq() __ "}"
            { ast::Top::FunDec(i.clone(), ts, Tag::new(ast::Type::Unit, i.span), t) }
        }

        pub rule program() -> ast::Program =  
            __ ts:(top() ++ (sep() __)) __? { ts }
        
    }
}
