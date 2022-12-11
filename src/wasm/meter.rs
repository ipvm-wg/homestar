//use std::sync::Arc;
//use wasmer::{
//    imports, wasmparser::Operator, wat2wasm, CompilerConfig, EngineBuilder, Instance, Module,
//    Store, TypedFunction,
//};
//use wasmer_compiler_cranelift::Cranelift;
//use wasmer_middlewares::{
//    metering::{get_remaining_points, set_remaining_points, MeteringPoints},
//    Metering,
//};
//
//pub fn cost(op: &Operator) -> u64 {
//    match op {
//        Operator::LocalGet { .. } => 1,
//        Operator::I32Const { .. } => 1,
//        Operator::I32Add { .. } => 2,
//        _ => 0,
//    }
//}

// pub fn setup() {
//     let metering = Arc::new(Metering::new(10, cost_function));
//     let mut compiler_config = Cranelift::default();
//     compiler_config.push_middleware(metering);
//
//     let mut store = Store::new(EngineBuilder::new(compiler_config));
//
//     println!("Compiling module...");
//     let module = Module::new(&store, wasm_bytes)?;
//     let import_object = imports! {};
//
//     println!("Instantiating module...");
//     let instance = Instance::new(&mut store, &module, &import_object)?;
//
//     let add_one: TypedFunction<i32, i32> = instance
//         .exports
//         .get_function("add_one")?
//         .typed(&mut store)?;
//
//     println!("Calling `add_one` function once...");
//     add_one.call(&mut store, 1)?;
//
//     let remaining_points_after_first_call = get_remaining_points(&mut store, &instance);
// }
