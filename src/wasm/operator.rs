use wasmer::wasmparser::Operator;

pub fn to_cost(op: &Operator) -> u64 {
    match op {
        Operator::LocalGet { .. } | Operator::I32Const { .. } => 1,
        Operator::I32Add { .. } => 2,
        _ => 0,
    }
}
