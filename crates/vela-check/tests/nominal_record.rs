use vela_check::{Type, check_program};

#[test]
fn bare_record_literal_becomes_nominal_in_typed_context() {
    let src = "type Point = { x : Float, y : Float }\nlet p : Point = { x = 1.0, y = 2.0 }\np";
    assert_eq!(check_program(src).expect("ok"), Type::Named("Point".into(), vec![]));
}

#[test]
fn untyped_record_literal_stays_structural() {
    let src = "type Point = { x : Float, y : Float }\nlet q = { x = 1.0, y = 2.0 }\nq.x";
    assert_eq!(check_program(src).expect("ok"), Type::Float);
}

#[test]
fn field_access_on_nominal_record() {
    let src =
        "type Point = { x : Float, y : Float }\nlet p : Point = { x = 1.0, y = 2.0 }\np.x";
    assert_eq!(check_program(src).expect("ok"), Type::Float);
}

#[test]
fn missing_field_rejected_for_nominal() {
    let src = "type Point = { x : Float, y : Float }\nlet p : Point = { x = 1.0 }\np";
    assert!(check_program(src).is_err());
}

#[test]
fn extra_field_rejected_for_nominal() {
    let src =
        "type Point = { x : Float, y : Float }\nlet p : Point = { x = 1.0, y = 2.0, z = 3.0 }\np";
    assert!(check_program(src).is_err());
}
