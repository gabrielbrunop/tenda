use crate::environment::StoredValue;
use crate::function::{params, Function};
use crate::runtime_error::{runtime_err, type_err, Result};
use crate::stack::Stack;
use crate::value::Value;

macro_rules! global {
    ($stack:ident, $builtin:expr) => {{
        let builtin = $builtin;

        $stack
            .define(builtin.0, StoredValue::Unique(builtin.1))
            .unwrap();
    }};
    ($stack:ident, $($builtin:expr),+) => {{
        $(global!($stack, $builtin);)+
    }};
}

macro_rules! builtin_assoc_array {
    ($($name:literal => $value:expr),+ $(,)?) => {{
        use std::cell::RefCell;
        use std::rc::Rc;
        use crate::associative_array::AssociativeArrayKey;

        let mut map = indexmap::IndexMap::new();

        $(
            let key = AssociativeArrayKey::String($name.to_string());
            map.insert(key, $value);
        )+

        Value::AssociativeArray(Rc::new(RefCell::new(map)))
    }};
}

macro_rules! define_assoc_array {
    ($assoc_array_name:literal, { $($name:literal => $value:expr),+ $(,)? }) => {
        (
            $assoc_array_name.to_string(),
            builtin_assoc_array!($($name => $value),+)
        )
    };
}

macro_rules! builtin_fn {
    ([$($param:expr),*], $body:expr) => {
        Value::Function(Function::new_builtin(
            params![$($param),*],
            $body,
        ))
    };
}

macro_rules! define_fn {
    ($name:literal, [$($param:expr),*], $body:expr) => {
        ($name.to_string(), builtin_fn!([$($param),*], $body))
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

pub fn setup_native_bindings(stack: &mut Stack) {
    setup_list_global_bindings(stack);
    setup_math_global_bindings(stack);
}

pub fn setup_io_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        define_fn!("exiba", ["texto"], |args, _| {
            let text = match args!(args, 0) {
                Value::String(value) => value.to_string(),
                value => format!("{}", value),
            };

            println!("{}", text);

            Ok(Value::Nil)
        })
    );
}

pub fn setup_list_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        define_assoc_array!("Lista", {
            "tamanho" => builtin_fn!(["lista"], |args, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());

                Ok(Value::Number(list.len() as f64))
            }),
            "insira" => builtin_fn!(["lista", "valor"], |args, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());

                list.push(args!(args, 1).clone());

                Ok(Value::Nil)
            }),
            "remova" => builtin_fn!(["lista", "valor"], |args, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());
                let value = args!(args, 1);
                let index = list.iter().position(|v| v == value);

                if let Some(index) = index {
                    return Ok(list.remove(index));
                }

                Ok(Value::Nil)
            }),
            "remova_todos" => builtin_fn!(["lista", "valor"], |args, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());
                let value = args!(args, 1);

                list.retain(|v| v != value);

                Ok(Value::Nil)
            }),
            "remova_por_índice" => builtin_fn!(["lista", "índice"], |args, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());
                let index = ensure!(args!(args, 1), Number(value) => *value as usize);

                if list.len() <= index {
                    return runtime_err!(IndexOutOfBounds { index, len: list.len() });
                }

                let element = list.remove(index);

                Ok(element)
            }),
            "obtenha" => builtin_fn!(["lista", "índice"], |args, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let index = ensure!(args!(args, 1), Number(value) => *value as usize);

                if list.len() <= index {
                    return runtime_err!(IndexOutOfBounds { index, len: list.len() });
                }

                Ok(list[index].clone())
            }),
            "índice_de" => builtin_fn!(["lista", "valor"], |args, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let value = args!(args, 1);
                let index = list.iter().position(|v| v == value).map(|i| i as f64);

                Ok(index.map(Value::Number).unwrap_or(Value::Nil))
            }),
            "contém" => builtin_fn!(["lista", "valor"], |args, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let value = args!(args, 1);

                Ok(Value::Boolean(list.contains(value)))
            }),
            "vazio" => builtin_fn!(["lista"], |args, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());

                Ok(Value::Boolean(list.is_empty()))
            }),
            "limpar" => builtin_fn!(["lista"], |args, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());

                list.clear();

                Ok(Value::Nil)
            }),
            "fatiar" => builtin_fn!(["lista", "início", "fim"], |args, _| {
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
            }),
            "para_cada" => builtin_fn!(["lista", "função"], |args, interpreter| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let function = ensure!(args!(args, 1), Function(function) => function);

                for (i, value) in list.iter().enumerate() {
                    let i = Value::Number(i as f64);
                    let args = vec![value, &i].into_iter();
                    let params = function
                        .get_params()
                        .iter()
                        .zip(args)
                        .map(|(param_name, arg_value)| (param_name.clone(), arg_value.clone()))
                        .collect();

                    function.call(params, interpreter)?;
                }

                Ok(Value::Nil)
            }),
            "transformar" => builtin_fn!(["lista", "função"], |args, interpreter| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let function = ensure!(args!(args, 1), Function(function) => function);

                let transformed: Result<Vec<Value>> = list.iter().try_fold(Vec::new(), |mut acc, value| {
                    let args = vec![value].into_iter();
                    let params = function
                        .get_params()
                        .iter()
                        .zip(args)
                        .map(|(param_name, arg_value)| (param_name.clone(), arg_value.clone()))
                        .collect();

                    let result = function.call(params, interpreter)?;

                    acc.push(result);

                    Ok(acc)
                });

                Ok(Value::List(Rc::new(RefCell::new(transformed?))))
            })
        })
    );
}

fn setup_math_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        define_fn!("absoluto", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.abs()))
        }),
        define_fn!("arredondar", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.round()))
        }),
        define_fn!("teto", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.ceil()))
        }),
        define_fn!("piso", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.floor()))
        }),
        define_fn!("raiz_quadrada", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.sqrt()))
        }),
        define_fn!("seno", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.sin()))
        }),
        define_fn!("cosseno", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.cos()))
        }),
        define_fn!("tangente", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.tan()))
        }),
        define_fn!("arco_seno", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.asin()))
        }),
        define_fn!("arco_cosseno", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.acos()))
        }),
        define_fn!("arco_tangente", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.atan()))
        }),
        define_fn!("logaritmo", ["número", "base"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);
            let base = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(number.log(base)))
        }),
        define_fn!("logaritmo_natural", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.ln()))
        }),
        define_fn!("logaritmo_10", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.log10()))
        }),
        define_fn!("potência", ["base", "expoente"], |args, _| {
            let base = ensure!(args!(args, 0), Number(value) => *value);
            let exponent = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(base.powf(exponent)))
        }),
        define_fn!("máximo", ["número_1", "número_2"], |args, _| {
            let number1 = ensure!(args!(args, 0), Number(value) => *value);
            let number2 = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(number1.max(number2)))
        }),
        define_fn!("mínimo", ["número_1", "número_2"], |args, _| {
            let number1 = ensure!(args!(args, 0), Number(value) => *value);
            let number2 = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(number1.min(number2)))
        }),
        define_fn!("aleatório", ["mínimo", "máximo"], |args, _| {
            let min = ensure!(args!(args, 0), Number(value) => *value);
            let max = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(rand::random::<f64>() * (max - min) + min))
        }),
        define_fn!("raiz_cúbica", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.cbrt()))
        }),
        define_fn!("seno_hiperbólico", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.sinh()))
        }),
        define_fn!("cosseno_hiperbólico", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.cosh()))
        }),
        define_fn!("tangente_hiperbólica", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.tanh()))
        }),
        define_fn!("arco_seno_hiperbólico", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.asinh()))
        }),
        define_fn!("arco_cosseno_hiperbólico", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.acosh()))
        }),
        define_fn!("arco_tangente_hiperbólica", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.atanh()))
        }),
        define_fn!("graus_para_radianos", ["número"], |args, _| {
            let degrees = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(degrees.to_radians()))
        }),
        define_fn!("radianos_para_graus", ["número"], |args, _| {
            let radians = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(radians.to_degrees()))
        }),
        define_fn!("truncar", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.trunc()))
        }),
        define_fn!("parte_fracionária", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.fract()))
        }),
        define_fn!("arco_tangente2", ["y", "x"], |args, _| {
            let y = ensure!(args!(args, 0), Number(value) => *value);
            let x = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(y.atan2(x)))
        }),
        define_fn!("resto", ["número_1", "número_2"], |args, _| {
            let n1 = ensure!(args!(args, 0), Number(value) => *value);
            let n2 = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(n1 % n2))
        }),
        define_fn!("resto_euclidiano", ["número_1", "número_2"], |args, _| {
            let n1 = ensure!(args!(args, 0), Number(value) => *value);
            let n2 = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(n1.rem_euclid(n2)))
        }),
        define_fn!("copiar_sinal", ["valor", "sinal"], |args, _| {
            let valor = ensure!(args!(args, 0), Number(value) => *value);
            let sinal = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(valor.copysign(sinal)))
        }),
        define_fn!("limitar", ["número", "mínimo", "máximo"], |args, _| {
            let x = ensure!(args!(args, 0), Number(value) => *value);
            let min_val = ensure!(args!(args, 1), Number(value) => *value);
            let max_val = ensure!(args!(args, 2), Number(value) => *value);

            let clamped = if x < min_val {
                min_val
            } else if x > max_val {
                max_val
            } else {
                x
            };

            Ok(Value::Number(clamped))
        }),
        define_fn!("signo", ["número"], |args, _| {
            let number = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(number.signum()))
        }),
        define_fn!("hipotenusa", ["número_1", "número_2"], |args, _| {
            let x = ensure!(args!(args, 0), Number(value) => *value);
            let y = ensure!(args!(args, 1), Number(value) => *value);

            Ok(Value::Number(x.hypot(y)))
        }),
        define_fn!("exponencial", ["número"], |args, _| {
            let x = ensure!(args!(args, 0), Number(value) => *value);

            Ok(Value::Number(x.exp()))
        }),
        define_fn!("fatorial", ["número"], |args, _| {
            let n = ensure!(args!(args, 0), Number(value) => *value);
            let int_n = n as i64;

            if n < 0.0 || (n - int_n as f64).abs() > f64::EPSILON {
                return runtime_err!(InvalidArgument {
                    value: Value::Number(n)
                });
            }

            let mut result = 1f64;

            for i in 1..=int_n {
                result *= i as f64;
            }

            Ok(Value::Number(result))
        })
    );
}
