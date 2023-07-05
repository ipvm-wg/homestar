//! [wasmtime::component::Component] test-utilities.

use std::{fmt::Write, iter};
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_component_util::REALLOC_AND_FREE;

/// [Param] types.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Type {
    S8,
    U8,
    S16,
    U16,
    I32,
    I64,
    F32,
    F64,
}

impl Type {
    fn store(&self) -> &'static str {
        match self {
            Self::S8 | Self::U8 => "store8",
            Self::S16 | Self::U16 => "store16",
            Self::I32 | Self::F32 | Self::I64 | Self::F64 => "store",
        }
    }

    fn primitive(&self) -> &'static str {
        match self {
            Self::S8 | Self::U8 | Self::S16 | Self::U16 | Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }
}

/// Specialized-param for making test Wasm [Component]s.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Param(pub Type, pub Option<usize>);

/// Setup an `echo` Wasm component with a type and type size.
pub fn setup_component(wasm_type: String, type_size: u32) -> wasmtime::component::Type {
    make_component(wasm_type, type_size, None)
}

/// Setup an `echo` Wasm component with a type and [Type] params.
pub fn setup_component_with_param(wasm_type: String, param: &[Param]) -> wasmtime::component::Type {
    make_component(wasm_type, 0, Some(param))
}

fn make_component(
    wasm_type: String,
    type_size: u32,
    opt_param: Option<&[Param]>,
) -> wasmtime::component::Type {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config).unwrap();

    let mut store = Store::new(&engine, ());

    let make_component = if let Some(param) = opt_param {
        make_echo_component_with_params(&wasm_type, param)
    } else {
        make_echo_component(&wasm_type, type_size)
    };
    let component = Component::new(&engine, make_component).unwrap();
    let instance = Linker::new(&engine)
        .instantiate(&mut store, &component)
        .unwrap();
    let func = instance.get_func(&mut store, "echo").unwrap();
    let ty = &func.params(&store)[0];
    ty.clone()
}

fn make_echo_component(type_definition: &str, type_size: u32) -> String {
    let mut offset = 0;
    make_echo_component_with_params(
        type_definition,
        &iter::repeat(Type::I32)
            .map(|ty| {
                let param = Param(ty, Some(offset));
                offset += 4;
                param
            })
            .take(usize::try_from(type_size).unwrap() / 4)
            .collect::<Vec<_>>(),
    )
}

fn make_echo_component_with_params(type_definition: &str, params: &[Param]) -> String {
    let func = if params.is_empty() {
        "(func (export \"echo\"))".to_string()
    } else if params.len() == 1 || params.len() > 16 {
        let primitive = if params.len() == 1 {
            params[0].0.primitive()
        } else {
            "i32"
        };

        format!(
            r#"
            (func (export "echo") (param {primitive}) (result {primitive})
                 local.get 0
            )"#,
        )
    } else {
        let mut param_string = String::new();
        let mut store = String::new();
        let mut size = 8;

        for (index, Param(ty, offset)) in params.iter().enumerate() {
            let primitive = ty.primitive();

            write!(&mut param_string, " {primitive}").unwrap();
            if let Some(offset) = offset {
                write!(
                    &mut store,
                    "({primitive}.{} offset={offset} (local.get $base) (local.get {index}))",
                    ty.store(),
                )
                .unwrap();

                size = size.max(offset + 8);
            }
        }

        format!(
            r#"
            (func (export "echo") (param{param_string}) (result i32)
                (local $base i32)
                (local.set $base
                    (call $realloc
                        (i32.const 0)
                        (i32.const 0)
                        (i32.const 4)
                        (i32.const {size})))
                {store}
                local.get $base
            )"#
        )
    };

    let type_section = if type_definition.contains("(type ") {
        type_definition.to_string()
    } else {
        format!("(type $Foo' {type_definition})")
    };

    format!(
        r#"
        (component
            (core module $m
                {func}

                (memory (export "memory") 1)
                {REALLOC_AND_FREE}
            )

            (core instance $i (instantiate $m))

            {type_section}
            (export $Foo "foo" (type $Foo'))

            (func (export "echo") (param "a" $Foo) (result "b" $Foo)
                (canon lift
                    (core func $i "echo")
                    (memory $i "memory")
                    (realloc (func $i "realloc"))
                )
            )
        )"#
    )
}
