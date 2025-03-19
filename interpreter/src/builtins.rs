use crate::environment::StoredValue;
use crate::function::{params, Function};
use crate::runtime_error::{runtime_err, type_err};
use crate::stack::Stack;
use crate::value::Value;

pub fn setup_native_bindings(stack: &mut Stack) {
    setup_global_bindings(stack);
}

macro_rules! add_global {
    ($stack:ident, $builtin:expr) => {{
        let builtin = $builtin;

        $stack
            .define(builtin.0, StoredValue::Unique(builtin.1))
            .unwrap();
    }};
    ($stack:ident, $($builtin:expr),+) => {{
        $(add_global!($stack, $builtin);)+
    }};
}

macro_rules! builtin_assoc_array {
    ($assoc_array_name:literal, $($name:literal => $value:expr),+) => {
        (
            $assoc_array_name.to_string(),
            {
                use std::cell::RefCell;
                use std::rc::Rc;
                use crate::value::AssociativeArrayKey;

                let mut map = indexmap::IndexMap::new();

                $(
                    let key = AssociativeArrayKey::String($name.to_string());
                    map.insert(key, $value);
                )+

                Value::AssociativeArray(Rc::new(RefCell::new(map)))
            }
        )
    };
}

macro_rules! builtin_fn {
    ($args:expr, $body:expr) => {
        Value::Function(Function::new($args, None, None, $body))
    };
}

macro_rules! named_builtin_fn {
    ($name:literal, $args:expr, $body:expr) => {
        (
            $name.to_string(),
            Value::Function(Function::new($args, None, None, $body)),
        )
    };
}

macro_rules! args {
    ($args:expr, $index:expr) => {
        &$args[$index].1
    };
}

macro_rules! ensure {
    ($val:expr, $variant:ident($binding:pat) => $body:expr) => {{
        match $val {
            Value::$variant($binding) => $body,
            value => return type_err!($variant, value.kind()),
        }
    }};
}

fn setup_global_bindings(stack: &mut Stack) {
    add_global!(
        stack,
        named_builtin_fn!("exiba", params!["texto"], |args, _, _, _| {
            let text = match args!(args, 0) {
                Value::String(value) => value.to_string(),
                value => format!("{}", value),
            };

            println!("{}", text);

            Ok(Value::Nil)
        }),
        builtin_assoc_array!(
            "Lista",

            "tamanho" => builtin_fn!(params!["lista"], |args, _, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());

                Ok(Value::Number(list.len() as f64))
            }),
            "insira" => builtin_fn!(params!["lista", "valor"], |args, _, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());

                list.push(args!(args, 1).clone());

                Ok(Value::Nil)
            }),
            "remova" => builtin_fn!(params!["lista", "valor"], |args, _, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());
                let value = args!(args, 1);
                let index = list.iter().position(|v| v == value);

                if let Some(index) = index {
                    return Ok(list.remove(index));
                }

                Ok(Value::Nil)
            }),
            "remova_todos" => builtin_fn!(params!["lista", "valor"], |args, _, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());
                let value = args!(args, 1);

                list.retain(|v| v != value);

                Ok(Value::Nil)
            }),
            "remova_por_índice" => builtin_fn!(params!["lista", "índice"], |args, _, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());
                let index = ensure!(args!(args, 1), Number(value) => *value as usize);

                if list.len() <= index {
                    return runtime_err!(IndexOutOfBounds { index, len: list.len() });
                }

                let element = list.remove(index);

                Ok(element)
            }),
            "obtenha" => builtin_fn!(params!["lista", "índice"], |args, _, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let index = ensure!(args!(args, 1), Number(value) => *value as usize);

                if list.len() <= index {
                    return runtime_err!(IndexOutOfBounds { index, len: list.len() });
                }

                Ok(list[index].clone())
            }),
            "índice_de" => builtin_fn!(params!["lista", "valor"], |args, _, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let value = args!(args, 1);

                let index = list.iter().position(|v| v == value);

                Ok(Value::Number(index.map(|i| i as f64).unwrap_or(-1.0)))
            }),
            "contém" => builtin_fn!(params!["lista", "valor"], |args, _, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let value = args!(args, 1);

                Ok(Value::Boolean(list.contains(value)))
            }),
            "vazio" => builtin_fn!(params!["lista"], |args, _, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());

                Ok(Value::Boolean(list.is_empty()))
            }),
            "limpar" => builtin_fn!(params!["lista"], |args, _, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());

                list.clear();

                Ok(Value::Nil)
            }),
            "fatiar" => builtin_fn!(params!["lista", "início", "fim"], |args, _, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let start = ensure!(args!(args, 1), Number(value) => *value as usize);
                let end = ensure!(args!(args, 2), Number(value) => *value as usize);

                if start > end {
                    return runtime_err!(InvalidRangeBounds { bound: end as f64 });
                }

                if end >= list.len() {
                    return runtime_err!(IndexOutOfBounds { index: end, len: list.len() });
                }

                let extracted = list[start..=end].to_vec();

                Ok(Value::List(Rc::new(RefCell::new(extracted))))
            })
        )
    );
}
