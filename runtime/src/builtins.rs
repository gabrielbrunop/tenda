use crate::date::Date;
use crate::environment::StoredValue;
use crate::function::{params, Function};
use crate::runtime_error::RuntimeError;
use crate::stack::Stack;
use crate::value::Value;
use crate::{platform, value};

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

macro_rules! def_value {
    ($name:literal, $value:expr) => {
        ($name.to_string(), $value)
    };
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

macro_rules! assoc_array_enum {
    ($($name:literal),+ $(,)?) => {{
        builtin_assoc_array!($($name => Value::String($name.to_string())),+)
    }};
}

macro_rules! def_assoc_array {
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
    ($body:expr) => {
        Value::Function(Function::new_builtin(
            Vec::new(),
            $body,
        ))
    };
}

macro_rules! def_fn {
    ($name:literal, [$($param:expr),*], $body:expr) => {
        ($name.to_string(), builtin_fn!([$($param),*], $body))
    };
}

macro_rules! args {
    ($args:expr, $index:expr) => {
        &$args[$index].1
    };
}

macro_rules! error_object {
    ($kind:literal) => {
        builtin_assoc_array! {
            "erro" => builtin_assoc_array! {
                "tipo" => Value::String($kind.to_string()),
            }
        }
    };
}

macro_rules! success_object {
    ($value:expr) => {
        builtin_assoc_array! {
            "valor" => $value,
        }
    };
}

macro_rules! ensure {
    ($val:expr, $variant:ident($binding:pat) => $body:expr) => {{
        use crate::runtime_error::RuntimeError;
        use crate::value::Value;
        use crate::value::ValueType;

        match $val {
            Value::$variant($binding) => $body,
            value => {
                return Err(Box::new(RuntimeError::UnexpectedTypeError {
                    expected: ValueType::$variant,
                    found: value.kind(),
                    span: None,
                    message: None,
                    stacktrace: vec![],
                }))
            }
        }
    }};
}

pub fn setup_native_bindings(stack: &mut Stack) {
    setup_io_global_bindings(stack);
    setup_list_global_bindings(stack);
    setup_math_global_bindings(stack);
    setup_string_global_bindings(stack);
    setup_file_global_bindings(stack);
    setup_program_global_bindings(stack);
    setup_date_global_bindings(stack);
}

pub fn setup_io_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        def_fn!("exiba", ["texto"], |args, runtime, _| {
            let text = match args!(args, 0) {
                Value::String(value) => value.to_string(),
                value => format!("{}", value),
            };

            runtime.get_platform().println(&text);

            Ok(Value::Nil)
        }),
        def_fn!("entrada", [], |_, runtime, _| {
            let input = runtime.get_platform().read_line();

            Ok(Value::String(input))
        }),
        def_fn!("leia", ["texto"], |args, runtime, _| {
            let prompt = match args!(args, 0) {
                Value::String(value) => value.to_string(),
                value => format!("{}", value),
            };

            runtime.get_platform().print(&prompt);

            let input = runtime.get_platform().read_line();

            Ok(Value::String(input))
        }),
        def_assoc_array!("Saída", {
            "exiba" => builtin_fn!(["texto"], |args, runtime, _| {
                let text = match args!(args, 0) {
                    Value::String(value) => value.to_string(),
                    value => format!("{}", value),
                };

                runtime.get_platform().println(&text);

                Ok(Value::Nil)
            }),
            "escreva" => builtin_fn!(["texto"], |args, runtime, _| {
                let text = match args!(args, 0) {
                    Value::String(value) => value.to_string(),
                    value => format!("{}", value),
                };

                runtime.get_platform().write(&text);

                Ok(Value::Nil)
            }),
            "leia" => builtin_fn!(["texto"], |args, runtime, _| {
                let prompt = match args!(args, 0) {
                    Value::String(value) => value.to_string(),
                    value => format!("{}", value),
                };

                runtime.get_platform().print(&prompt);

                let input = runtime.get_platform().read_line();

                Ok(Value::String(input))
            }),
            "entrada" => builtin_fn!([], |_, runtime, _| {
                let input = runtime.get_platform().read_line();

                Ok(Value::String(input))
            })
        })
    );
}

pub fn setup_list_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        def_assoc_array!("Lista", {
            "tamanho" => builtin_fn!(["lista"], |args, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());

                Ok(Value::Number(list.len() as f64))
            }),
            "insira" => builtin_fn!(["lista", "valor"], |args, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());

                list.push(args!(args, 1).clone());

                Ok(Value::Nil)
            }),
            "remova" => builtin_fn!(["lista", "valor"], |args, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());
                let value = args!(args, 1);
                let index = list.iter().position(|v| v == value);

                if let Some(index) = index {
                    return Ok(list.remove(index));
                }

                Ok(Value::Nil)
            }),
            "remova_todos" => builtin_fn!(["lista", "valor"], |args, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());
                let value = args!(args, 1);

                list.retain(|v| v != value);

                Ok(Value::Nil)
            }),
            "remova_por_índice" => builtin_fn!(["lista", "índice"], |args, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());
                let index = ensure!(args!(args, 1), Number(value) => *value as usize);

                if list.len() <= index {
                    return Err(Box::new(RuntimeError::IndexOutOfBounds {
                        index,
                        len: list.len(),
                        span: None,
                        help: vec![],
                        stacktrace: vec![],
                    }));
                }

                let element = list.remove(index);

                Ok(element)
            }),
            "obtenha" => builtin_fn!(["lista", "índice"], |args, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let index = ensure!(args!(args, 1), Number(value) => *value as usize);

                if list.len() <= index {
                    return Err(Box::new(RuntimeError::IndexOutOfBounds {
                        index,
                        len: list.len(),
                        span: None,
                        help: vec![],
                        stacktrace: vec![],
                    }));
                }

                Ok(list[index].clone())
            }),
            "índice_de" => builtin_fn!(["lista", "valor"], |args, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let value = args!(args, 1);
                let index = list.iter().position(|v| v == value).map(|i| i as f64);

                Ok(index.map(Value::Number).unwrap_or(Value::Nil))
            }),
            "contém" => builtin_fn!(["lista", "valor"], |args, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let value = args!(args, 1);

                Ok(Value::Boolean(list.contains(value)))
            }),
            "vazio" => builtin_fn!(["lista"], |args, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());

                Ok(Value::Boolean(list.is_empty()))
            }),
            "limpa" => builtin_fn!(["lista"], |args, _, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());

                list.clear();

                Ok(Value::Nil)
            }),
            "fatia" => builtin_fn!(["lista", "início", "fim"], |args, _, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let start = ensure!(args!(args, 1), Number(value) => *value as usize);
                let end = ensure!(args!(args, 2), Number(value) => *value as usize);

                if start > end {
                    return Err(Box::new(RuntimeError::InvalidRangeBounds {
                        bound: end as f64,
                        span: None,
                        stacktrace: vec![],
                    }));
                }

                if end >= list.len() {
                    return Err(Box::new(RuntimeError::IndexOutOfBounds {
                        index: end,
                        len: list.len(),
                        span: None,
                        help: vec![],
                        stacktrace: vec![],
                    }));
                }

                let extracted = list[start..=end].to_vec();

                Ok(Value::List(Rc::new(RefCell::new(extracted))))
            }),
            "para_cada" => builtin_fn!(["lista", "função"], |args, runtime, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let function = ensure!(args!(args, 1), Function(function) => function);

                for (i, value) in list.iter().enumerate() {
                    let i = Value::Number(i as f64);
                    let args = vec![value.clone(), i];

                    runtime.call_function(None, function.clone(), args, None)?;
                }

                Ok(Value::Nil)
            }),
            "de_intervalo" => builtin_fn!(["intervalo"], |args, _, _| {
                let (from, to) = match args!(args, 0) {
                    Value::Range(from, start) => (from, start),
                    value => return Err(Box::new(RuntimeError::UnexpectedTypeError {
                        expected: value::ValueType::Range,
                        found: value.kind(),
                        span: None,
                        message: None,
                        stacktrace: vec![],
                    }))
                };

                let list = (*from as i64..=*to as i64)
                    .map(|i| Value::Number(i as f64))
                    .collect::<Vec<_>>();

                Ok(Value::List(Rc::new(RefCell::new(list))))
            }),
            "de_texto" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                let list = text
                    .chars()
                    .map(|c| Value::String(c.to_string()))
                    .collect::<Vec<_>>();

                Ok(Value::List(Rc::new(RefCell::new(list))))
            }),
            "transforma" => builtin_fn!(["lista", "função"], |args, runtime, _| {
                let list = ensure!(args!(args, 0), List(list) => list.borrow());
                let function = ensure!(args!(args, 1), Function(function) => function);

                let mut new_list = vec![];

                for value in list.iter() {
                    let args = vec![value.clone()];
                    let result = runtime.call_function(None, function.clone(), args, None)?;

                    new_list.push(result);
                }

                Ok(Value::List(Rc::new(RefCell::new(new_list))))
            })
        })
    );
}

fn setup_math_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        def_value!("infinito", Value::Number(f64::INFINITY)),
        def_value!("NaN", Value::Number(f64::NAN)),
        def_assoc_array!("Matemática", {
            "maior_número" => Value::Number(f64::MAX),
            "menor_número" => Value::Number(f64::MIN),
            "pi" => Value::Number(std::f64::consts::PI),
            "e" => Value::Number(std::f64::consts::E),
            "absoluto" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.abs()))
            }),
            "arredonda" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.round()))
            }),
            "teto" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.ceil()))
            }),
            "piso" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.floor()))
            }),
            "raiz_quadrada" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.sqrt()))
            }),
            "seno" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.sin()))
            }),
            "cosseno" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.cos()))
            }),
            "tangente" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.tan()))
            }),
            "arco_seno" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.asin()))
            }),
            "arco_cosseno" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.acos()))
            }),
            "arco_tangente" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.atan()))
            }),
            "logaritmo" => builtin_fn!(["número", "base"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);
                let base = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(number.log(base)))
            }),
            "logaritmo_natural" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.ln()))
            }),
            "logaritmo_10" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.log10()))
            }),
            "potência" => builtin_fn!(["base", "expoente"], |args, _, _| {
                let base = ensure!(args!(args, 0), Number(value) => *value);
                let exponent = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(base.powf(exponent)))
            }),
            "máximo" => builtin_fn!(["número_1", "número_2"], |args, _, _| {
                let number1 = ensure!(args!(args, 0), Number(value) => *value);
                let number2 = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(number1.max(number2)))
            }),
            "mínimo" => builtin_fn!(["número_1", "número_2"], |args, _, _| {
                let number1 = ensure!(args!(args, 0), Number(value) => *value);
                let number2 = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(number1.min(number2)))
            }),
            "aleatório" => builtin_fn!(["mínimo", "máximo"], |args, runtime, _| {
                let min = ensure!(args!(args, 0), Number(value) => *value);
                let max = ensure!(args!(args, 1), Number(value) => *value);

                let number = runtime.get_platform().rand();

                Ok(Value::Number(number * (max - min) + min))
            }),
            "raiz_cúbica" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.cbrt()))
            }),
            "seno_hiperbólico" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.sinh()))
            }),
            "cosseno_hiperbólico" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.cosh()))
            }),
            "tangente_hiperbólica" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.tanh()))
            }),
            "arco_seno_hiperbólico" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.asinh()))
            }),
            "arco_cosseno_hiperbólico" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.acosh()))
            }),
            "arco_tangente_hiperbólica" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.atanh()))
            }),
            "graus_para_radianos" => builtin_fn!(["número"], |args, _, _| {
                let degrees = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(degrees.to_radians()))
            }),
            "radianos_para_graus" => builtin_fn!(["número"], |args, _, _| {
                let radians = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(radians.to_degrees()))
            }),
            "trunca" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.trunc()))
            }),
            "parte_fracionária" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.fract()))
            }),
            "arco_tangente2" => builtin_fn!(["y", "x"], |args, _, _| {
                let y = ensure!(args!(args, 0), Number(value) => *value);
                let x = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(y.atan2(x)))
            }),
            "resto" => builtin_fn!(["número_1", "número_2"], |args, _, _| {
                let n1 = ensure!(args!(args, 0), Number(value) => *value);
                let n2 = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(n1 % n2))
            }),
            "resto_euclidiano" => builtin_fn!(["número_1", "número_2"], |args, _, _| {
                let n1 = ensure!(args!(args, 0), Number(value) => *value);
                let n2 = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(n1.rem_euclid(n2)))
            }),
            "copia_sinal" => builtin_fn!(["valor", "sinal"], |args, _, _| {
                let valor = ensure!(args!(args, 0), Number(value) => *value);
                let sinal = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(valor.copysign(sinal)))
            }),
            "limita" => builtin_fn!(["número", "mínimo", "máximo"], |args, _, _| {
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
            "sinal" => builtin_fn!(["número"], |args, _, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.signum()))
            }),
            "hipotenusa" => builtin_fn!(["número_1", "número_2"], |args, _, _| {
                let x = ensure!(args!(args, 0), Number(value) => *value);
                let y = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(x.hypot(y)))
            }),
            "exponencial" => builtin_fn!(["número"], |args, _, _| {
                let x = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(x.exp()))
            }),
            "fatorial" => builtin_fn!(["número"], |args, _, _| {
                let n = ensure!(args!(args, 0), Number(value) => *value);
                let int_n = n as i64;

                if n < 0.0 || (n - int_n as f64).abs() > f64::EPSILON {
                    return Err(Box::new(RuntimeError::InvalidArgument {
                        value: Value::Number(n),
                        span: None,
                        stacktrace: vec![],
                    }));
                }

                let mut result = 1f64;

                for i in 1..=int_n {
                    result *= i as f64;
                }

                Ok(Value::Number(result))
            })
        })
    );
}

fn setup_string_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        def_assoc_array!("Texto", {
            "tamanho" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::Number(text.len() as f64))
            }),
            "vazio" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::Boolean(text.is_empty()))
            }),
            "subtexto" => builtin_fn!(["texto", "início", "tamanho"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let start = ensure!(args!(args, 1), Number(value) => *value as usize);
                let len = ensure!(args!(args, 2), Number(value) => *value as usize);

                if start >= text.len() {
                    return Err(Box::new(RuntimeError::IndexOutOfBounds {
                        index: start,
                        len: text.len(),
                        span: None,
                        help: vec![],
                        stacktrace: vec![],
                    }));
                }

                Ok(Value::String(text[start..start + len].to_string()))
            }),
            "para_lista" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::List(Rc::new(RefCell::new(
                    text.chars().map(|c| Value::String(c.to_string())).collect(),
                ))))
            }),
            "para_maiúsculas" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.to_uppercase()))
            }),
            "para_minúsculas" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.to_lowercase()))
            }),
            "contém" => builtin_fn!(["texto", "subtexto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let subtext = ensure!(args!(args, 1), String(value) => value);

                Ok(Value::Boolean(text.contains(subtext)))
            }),
            "começa_com" => builtin_fn!(["texto", "prefixo"], |args, _, _| {
                let text = ensure!(args!(args, 0),
                    String(value) => value
                );

                let prefix = ensure!(args!(args, 1),
                    String(value) => value
                );

                Ok(Value::Boolean(text.starts_with(prefix)))
            }),
            "termina_com" => builtin_fn!(["texto", "sufixo"], |args, _, _| {
                let text = ensure!(args!(args, 0),
                    String(value) => value
                );

                let suffix = ensure!(args!(args, 1),
                    String(value) => value
                );

                Ok(Value::Boolean(text.ends_with(suffix)))
            }),
            "índice_de" => builtin_fn!(["texto", "subtexto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let subtext = ensure!(args!(args, 1), String(value) => value);
                let index = text.find(subtext).map(|i| i as f64);

                Ok(index.map(Value::Number).unwrap_or(Value::Nil))
            }),
            "repita" => builtin_fn!(["texto", "vezes"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let times = ensure!(args!(args, 1), Number(value) => *value as usize);

                Ok(Value::String(text.repeat(times)))
            }),
            "substitua" => builtin_fn!(["texto", "antigo", "novo"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let old = ensure!(args!(args, 1), String(value) => value);
                let new = ensure!(args!(args, 2), String(value) => value);

                Ok(Value::String(text.replace(old, new)))
            }),
            "corta" => builtin_fn!(["texto", "início", "fim"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let start = ensure!(args!(args, 1), Number(value) => *value as usize);
                let end = ensure!(args!(args, 2), Number(value) => *value as usize);

                if start > end {
                    return Err(Box::new(RuntimeError::InvalidRangeBounds {
                        bound: end as f64,
                        span: None,
                        stacktrace: vec![],
                    }));
                }

                if end >= text.len() {
                    return Err(Box::new(RuntimeError::IndexOutOfBounds {
                        index: end,
                        len: text.len(),
                        span: None,
                        help: vec![],
                        stacktrace: vec![],
                    }));
                }

                Ok(Value::String(text[start..=end].to_string()))
            }),
            "inverta" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.chars().rev().collect()))
            }),
            "remova" => builtin_fn!(["texto", "início", "tamanho"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let start = ensure!(args!(args, 1), Number(value) => *value as usize);
                let len = ensure!(args!(args, 2), Number(value) => *value as usize);

                if start >= text.len() {
                    return Err(Box::new(RuntimeError::IndexOutOfBounds {
                        index: start,
                        len: text.len(),
                        span: None,
                        help: vec![],
                        stacktrace: vec![],
                    }));
                }

                Ok(Value::String(
                    text.chars()
                        .enumerate()
                        .filter_map(|(i, c)| {
                            if i < start || i >= start + len {
                                Some(c)
                            } else {
                                None
                            }
                        })
                        .collect(),
                ))
            }),
            "remova_prefixo" => builtin_fn!(["texto", "prefixo"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let prefix = ensure!(args!(args, 1), String(value) => value);

                if text.starts_with(prefix) {
                    Ok(Value::String(text[prefix.len()..].to_string()))
                } else {
                    Ok(Value::String(text.to_string()))
                }
            }),
            "remova_sufixo" => builtin_fn!(["texto", "sufixo"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let suffix = ensure!(args!(args, 1), String(value) => value);

                if text.ends_with(suffix) {
                    Ok(Value::String(text[..text.len() - suffix.len()].to_string()))
                } else {
                    Ok(Value::String(text.to_string()))
                }
            }),
            "remova_espaços" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(
                    text.chars().filter(|c| !c.is_whitespace()).collect(),
                ))
            }),
            "remova_espaços_início" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.trim_start().to_string()))
            }),
            "remova_espaços_fim" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.trim_end().to_string()))
            }),
            "remova_espaços_início_fim" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.trim().to_string()))
            }),
            "para_número" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                match text.parse::<f64>() {
                    Ok(number) => Ok(Value::Number(number)),
                    Err(_) => Err(Box::new(RuntimeError::InvalidValueForConversion  {
                        value: Value::String(text.to_string()),
                        span: None,
                        stacktrace: vec![],
                    })),
                }
            })
        })
    );
}

fn setup_file_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        def_assoc_array!("Arquivo", {
            "erros" => assoc_array_enum! {
                "NÃO_ENCONTRADO",
                "PERMISSÃO_NEGADA",
                "JÁ_EXISTE",
                "OUTRO",
            },
            "leia" => builtin_fn!(["caminho"], |args, runtime, _| {
                let path = ensure!(args!(args, 0), String(value) => value);

                match runtime.get_platform().read_file(path) {
                    Ok(text) => Ok(success_object!(Value::String(text))),
                    Err(kind) => Ok(io_error_to_error_object(kind)),
                }
            }),
            "escreva" => builtin_fn!(["caminho", "conteúdo"], |args, runtime, _| {
                let path = ensure!(args!(args, 0), String(value) => value);
                let content = ensure!(args!(args, 1), String(value) => value);

                match runtime.get_platform().write_file(path, content) {
                    Ok(_) => Ok(success_object!(Value::Nil)),
                    Err(kind) => Ok(io_error_to_error_object(kind)),
                }
            }),
            "acrescenta" => builtin_fn!(["caminho", "conteúdo"], |args, runtime, _| {
                let path = ensure!(args!(args, 0), String(value) => value);
                let content = ensure!(args!(args, 1), String(value) => value);

                match runtime.get_platform().file_append(path, content) {
                    Ok(_) => Ok(success_object!(Value::Nil)),
                    Err(kind) => Ok(io_error_to_error_object(kind)),
                }
            }),
            "remova" => builtin_fn!(["caminho"], |args, runtime, _| {
                let path = ensure!(args!(args, 0), String(value) => value);

                match runtime.get_platform().remove_file(path) {
                    Ok(_) => Ok(success_object!(Value::Nil)),
                    Err(kind) => Ok(io_error_to_error_object(kind)),
                }
            }),
            "lista" => builtin_fn!(["caminho"], |args, runtime, _| {
                let path = ensure!(args!(args, 0), String(value) => value);

                match runtime.get_platform().list_files(path) {
                    Ok(files) => {
                        let files = files.into_iter().map(Value::String).collect();
                        let value = Value::List(Rc::new(RefCell::new(files)));

                        Ok(success_object!(value))
                    },
                    Err(kind) => Ok(io_error_to_error_object(kind)),
                }
            }),
            "cria_diretório" => builtin_fn!(["caminho"], |args, runtime, _| {
                let path = ensure!(args!(args, 0), String(value) => value);

                match runtime.get_platform().create_dir(path) {
                    Ok(_) => Ok(success_object!(Value::Nil)),
                    Err(kind) => Ok(io_error_to_error_object(kind)),
                }
            }),
            "remova_diretório" => builtin_fn!(["caminho"], |args, runtime, _| {
                let path = ensure!(args!(args, 0), String(value) => value);

                match runtime.get_platform().remove_dir(path) {
                    Ok(_) => Ok(success_object!(Value::Nil)),
                    Err(kind) => Ok(io_error_to_error_object(kind)),
                }
            }),
            "caminho_atual" => builtin_fn!(|_, runtime, _| {
                match runtime.get_platform().current_dir() {
                    Ok(path) => Ok(success_object!(Value::String(path))),
                    Err(kind) => Ok(io_error_to_error_object(kind)),
                }
            }),
        })
    );
}

fn setup_program_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        def_assoc_array!("Programa", {
            "argumentos" => builtin_fn!(|_, runtime, _| {
                let args = runtime.get_platform().args();
                let args = args.into_iter().map(Value::String).collect();
                let value = Value::List(Rc::new(RefCell::new(args)));

                Ok(value)
            }),
            "encerra" => builtin_fn!(["código"], |args, runtime, _| {
                let code = ensure!(args!(args, 0), Number(value) => *value as i32);

                runtime.get_platform().exit(code);

                Ok(Value::Nil)
            }),
            "espera" => builtin_fn!(["segundos"], |args, runtime, _| {
                let seconds = ensure!(args!(args, 0), Number(value) => *value);

                runtime.get_platform().sleep(seconds);

                Ok(Value::Nil)
            }),
        })
    );
}

fn setup_date_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        def_assoc_array!("Data", {
            "erros" => assoc_array_enum! {
                "ISO_INVÁLIDA",
                "TIMESTAMP_INVÁLIDO",
                "FUSO_HORÁRIO_INVÁLIDO",
            },
            "agora" => builtin_fn!(|_, runtime, _| {
                let now = runtime.get_platform().date_now();
                let tz = runtime.get_platform().timezone_offset();

                let value = Value::Date(Date::from_timestamp_millis(now, Some(tz)).unwrap());

                Ok(value)
            }),
            "para_iso" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::String(date.to_iso_string()))
            }),
            "para_timestamp" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.to_timestamp_millis() as f64))
            }),
            "de_iso" => builtin_fn!(["texto"], |args, _, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                 match Date::from_iso_string(text) {
                    Ok(date) => Ok(Value::Date(date)),
                    Err(kind) => Ok(date_error_to_error_object(*kind))
                }
            }),
            "de_timestamp" => builtin_fn!(["número"], |args, _, _| {
                let timestamp = ensure!(args!(args, 0), Number(value) => *value as i64);

                match Date::from_timestamp_millis(timestamp, None) {
                    Ok(date) => Ok(Value::Date(date)),
                    Err(kind) => Ok(date_error_to_error_object(*kind))
                }
            }),
            "com_região" => builtin_fn!(["data", "região"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);
                let offset = ensure!(args!(args, 1), String(value) => value);

                match date.with_named_timezone(offset) {
                    Ok(date) => Ok(Value::Date(date)),
                    Err(kind) => Ok(date_error_to_error_object(*kind))
                }
            }),
            "desvio_fuso_horário" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::String(date.to_offset_string()))
            }),
            "ano" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.year() as f64))
            }),
            "mês" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.month() as f64))
            }),
            "dia" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.day() as f64))
            }),
            "hora" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.hour() as f64))
            }),
            "minuto" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.minute() as f64))
            }),
            "segundo" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.second() as f64))
            }),
            "dia_da_semana" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.weekday() as f64))
            }),
            "dia_do_ano" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.ordinal() as f64))
            }),
            "semana_do_ano" => builtin_fn!(["data"], |args, _, _| {
                let date = ensure!(args!(args, 0), Date(date) => date);

                Ok(Value::Number(date.iso_week() as f64))
            }),
        })
    );
}

fn io_error_to_error_object(kind: platform::FileErrorKind) -> Value {
    use platform::FileErrorKind::*;

    match kind {
        NotFound => error_object!("NÃO_ENCONTRADO"),
        PermissionDenied => error_object!("PERMISSÃO_NEGADA"),
        AlreadyExists => error_object!("JÁ_EXISTE"),
        _ => error_object!("OUTRO"),
    }
}

fn date_error_to_error_object(kind: RuntimeError) -> Value {
    use RuntimeError::*;

    match kind {
        InvalidTimestamp { .. } => error_object!("TIMESTAMP_INVÁLIDO"),
        DateIsoParseError { .. } => error_object!("ISO_INVÁLIDA"),
        InvalidTimeZoneString { .. } => error_object!("FUSO_HORÁRIO_INVÁLIDO"),
        _ => panic!("unexpected date error kind: {:?}", kind),
    }
}
