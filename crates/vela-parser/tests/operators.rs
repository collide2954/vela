use vela_parser::{BinOp, Expr, Lit, PostOp, UnOp, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn bin(op: BinOp, a: Expr, b: Expr) -> Expr {
    Expr::BinOp(op, Box::new(a), Box::new(b))
}

#[test]
fn simple_addition() {
    assert_eq!(p("1 + 2"), bin(BinOp::Add, lit(1), lit(2)));
}

#[test]
fn left_assoc_subtraction() {
    assert_eq!(
        p("1 - 2 - 3"),
        bin(BinOp::Sub, bin(BinOp::Sub, lit(1), lit(2)), lit(3)),
    );
}

#[test]
fn mul_binds_tighter_than_add() {
    assert_eq!(
        p("1 + 2 * 3"),
        bin(BinOp::Add, lit(1), bin(BinOp::Mul, lit(2), lit(3))),
    );
    assert_eq!(
        p("1 * 2 + 3"),
        bin(BinOp::Add, bin(BinOp::Mul, lit(1), lit(2)), lit(3)),
    );
}

#[test]
fn pow_is_right_assoc() {
    assert_eq!(
        p("2 ^ 3 ^ 2"),
        bin(BinOp::Pow, lit(2), bin(BinOp::Pow, lit(3), lit(2))),
    );
}

#[test]
fn pow_binds_tighter_than_mul() {
    assert_eq!(
        p("2 * 3 ^ 2"),
        bin(BinOp::Mul, lit(2), bin(BinOp::Pow, lit(3), lit(2))),
    );
}

#[test]
fn concat_is_lower_than_arith() {
    assert_eq!(
        p("1 + 2 ++ 3 + 4"),
        bin(
            BinOp::Concat,
            bin(BinOp::Add, lit(1), lit(2)),
            bin(BinOp::Add, lit(3), lit(4)),
        ),
    );
}

#[test]
fn comparison_below_concat() {
    assert_eq!(
        p("1 ++ 2 == 3 ++ 4"),
        bin(
            BinOp::Eq,
            bin(BinOp::Concat, lit(1), lit(2)),
            bin(BinOp::Concat, lit(3), lit(4)),
        ),
    );
}

#[test]
fn and_above_or() {
    let a = var("a");
    let b = var("b");
    let c = var("c");
    assert_eq!(p("a or b and c"), bin(BinOp::Or, a, bin(BinOp::And, b, c)),);
}

#[test]
fn pipe_is_lowest() {
    assert_eq!(
        p("1 + 2 |> f"),
        bin(BinOp::Pipe, bin(BinOp::Add, lit(1), lit(2)), var("f")),
    );
}

#[test]
fn pipe_is_left_associative() {
    assert_eq!(
        p("1 |> f |> g"),
        bin(BinOp::Pipe, bin(BinOp::Pipe, lit(1), var("f")), var("g")),
    );
}

#[test]
fn unary_minus() {
    assert_eq!(p("-x"), Expr::UnaryOp(UnOp::Neg, Box::new(var("x"))),);
}

#[test]
fn unary_not() {
    assert_eq!(p("not p"), Expr::UnaryOp(UnOp::Not, Box::new(var("p"))),);
}

#[test]
fn postfix_question() {
    assert_eq!(p("x?"), Expr::Postfix(PostOp::Question, Box::new(var("x"))),);
}

#[test]
fn parens_override_precedence() {
    assert_eq!(
        p("(1 + 2) * 3"),
        bin(BinOp::Mul, bin(BinOp::Add, lit(1), lit(2)), lit(3)),
    );
}
