use vela_bytecode::{Const, Function, Module, Op};

#[test]
fn empty_module_has_no_entry() {
    let m = Module::default();
    assert_eq!(m.functions.len(), 0);
    assert!(m.entry.is_none());
}

#[test]
fn function_holds_instructions_and_constants() {
    let f = Function {
        name: "main".into(),
        arity: 0,
        n_regs: 2,
        n_upvals: 0,
        upvals: Vec::new(),
        consts: vec![Const::Int(1), Const::Int(2)],
        code: vec![
            Op::LoadConst { dst: 0, k: 0 },
            Op::LoadConst { dst: 1, k: 1 },
            Op::Add { dst: 0, a: 0, b: 1 },
            Op::Return { src: 0 },
        ],
        source_path: "<test>".into(),
        source_spans: Vec::new(),
    };
    let module = Module {
        functions: vec![f],
        entry: Some(0),
    };
    assert_eq!(module.function(0).instruction_count(), 4);
    assert_eq!(module.entry, Some(0));
}
