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
    setup_io_global_bindings(stack);
    setup_list_global_bindings(stack);
    setup_math_global_bindings(stack);
    setup_string_global_bindings(stack);
}

pub fn setup_io_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        define_fn!("exiba", ["texto"], |args, interpreter| {
            let text = match args!(args, 0) {
                Value::String(value) => value.to_string(),
                value => format!("{}", value),
            };

            interpreter.get_platform().println(&text);

            Ok(Value::Nil)
        }),
        define_assoc_array!("Saída", {
            "exiba" => builtin_fn!(["texto"], |args, interpreter| {
                let text = match args!(args, 0) {
                    Value::String(value) => value.to_string(),
                    value => format!("{}", value),
                };

                interpreter.get_platform().println(&text);

                Ok(Value::Nil)
            }),
            "escreva" => builtin_fn!(["texto"], |args, interpreter| {
                let text = match args!(args, 0) {
                    Value::String(value) => value.to_string(),
                    value => format!("{}", value),
                };

                interpreter.get_platform().write(&text);

                Ok(Value::Nil)
            }),
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
            "limpa" => builtin_fn!(["lista"], |args, _| {
                let mut list = ensure!(args!(args, 0), List(list) => list.borrow_mut());

                list.clear();

                Ok(Value::Nil)
            }),
            "fatia" => builtin_fn!(["lista", "início", "fim"], |args, _| {
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
            "transforma" => builtin_fn!(["lista", "função"], |args, interpreter| {
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
        define_assoc_array!("Matemática", {
            "absoluto" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.abs()))
            }),
            "arredonda" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.round()))
            }),
            "teto" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.ceil()))
            }),
            "piso" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.floor()))
            }),
            "raiz_quadrada" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.sqrt()))
            }),
            "seno" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.sin()))
            }),
            "cosseno" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.cos()))
            }),
            "tangente" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.tan()))
            }),
            "arco_seno" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.asin()))
            }),
            "arco_cosseno" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.acos()))
            }),
            "arco_tangente" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.atan()))
            }),
            "logaritmo" => builtin_fn!(["número", "base"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);
                let base = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(number.log(base)))
            }),
            "logaritmo_natural" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.ln()))
            }),
            "logaritmo_10" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.log10()))
            }),
            "potência" => builtin_fn!(["base", "expoente"], |args, _| {
                let base = ensure!(args!(args, 0), Number(value) => *value);
                let exponent = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(base.powf(exponent)))
            }),
            "máximo" => builtin_fn!(["número_1", "número_2"], |args, _| {
                let number1 = ensure!(args!(args, 0), Number(value) => *value);
                let number2 = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(number1.max(number2)))
            }),
            "mínimo" => builtin_fn!(["número_1", "número_2"], |args, _| {
                let number1 = ensure!(args!(args, 0), Number(value) => *value);
                let number2 = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(number1.min(number2)))
            }),
            "aleatório" => builtin_fn!(["mínimo", "máximo"], |args, interpreter| {
                let min = ensure!(args!(args, 0), Number(value) => *value);
                let max = ensure!(args!(args, 1), Number(value) => *value);

                let number = interpreter.get_platform().rand();

                Ok(Value::Number(number * (max - min) + min))
            }),
            "raiz_cúbica" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.cbrt()))
            }),
            "seno_hiperbólico" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.sinh()))
            }),
            "cosseno_hiperbólico" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.cosh()))
            }),
            "tangente_hiperbólica" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.tanh()))
            }),
            "arco_seno_hiperbólico" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.asinh()))
            }),
            "arco_cosseno_hiperbólico" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.acosh()))
            }),
            "arco_tangente_hiperbólica" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.atanh()))
            }),
            "graus_para_radianos" => builtin_fn!(["número"], |args, _| {
                let degrees = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(degrees.to_radians()))
            }),
            "radianos_para_graus" => builtin_fn!(["número"], |args, _| {
                let radians = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(radians.to_degrees()))
            }),
            "trunca" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.trunc()))
            }),
            "parte_fracionária" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.fract()))
            }),
            "arco_tangente2" => builtin_fn!(["y", "x"], |args, _| {
                let y = ensure!(args!(args, 0), Number(value) => *value);
                let x = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(y.atan2(x)))
            }),
            "resto" => builtin_fn!(["número_1", "número_2"], |args, _| {
                let n1 = ensure!(args!(args, 0), Number(value) => *value);
                let n2 = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(n1 % n2))
            }),
            "resto_euclidiano" => builtin_fn!(["número_1", "número_2"], |args, _| {
                let n1 = ensure!(args!(args, 0), Number(value) => *value);
                let n2 = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(n1.rem_euclid(n2)))
            }),
            "copia_sinal" => builtin_fn!(["valor", "sinal"], |args, _| {
                let valor = ensure!(args!(args, 0), Number(value) => *value);
                let sinal = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(valor.copysign(sinal)))
            }),
            "limita" => builtin_fn!(["número", "mínimo", "máximo"], |args, _| {
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
            "sinal" => builtin_fn!(["número"], |args, _| {
                let number = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(number.signum()))
            }),
            "hipotenusa" => builtin_fn!(["número_1", "número_2"], |args, _| {
                let x = ensure!(args!(args, 0), Number(value) => *value);
                let y = ensure!(args!(args, 1), Number(value) => *value);

                Ok(Value::Number(x.hypot(y)))
            }),
            "exponencial" => builtin_fn!(["número"], |args, _| {
                let x = ensure!(args!(args, 0), Number(value) => *value);

                Ok(Value::Number(x.exp()))
            }),
            "fatorial" => builtin_fn!(["número"], |args, _| {
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
        })
    );
}

fn setup_string_global_bindings(stack: &mut Stack) {
    global!(
        stack,
        define_assoc_array!("Texto", {
            "tamanho" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::Number(text.len() as f64))
            }),
            "vazio" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::Boolean(text.is_empty()))
            }),
            "subtexto" => builtin_fn!(["texto", "início", "tamanho"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let start = ensure!(args!(args, 1), Number(value) => *value as usize);
                let len = ensure!(args!(args, 2), Number(value) => *value as usize);

                if start >= text.len() {
                    return runtime_err!(IndexOutOfBounds {
                        index: start,
                        len: text.len()
                    });
                }

                Ok(Value::String(text[start..start + len].to_string()))
            }),
            "para_lista" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::List(Rc::new(RefCell::new(
                    text.chars().map(|c| Value::String(c.to_string())).collect(),
                ))))
            }),
            "para_maiúsculas" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.to_uppercase()))
            }),
            "para_minúsculas" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.to_lowercase()))
            }),
            "contém" => builtin_fn!(["texto", "subtexto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let subtext = ensure!(args!(args, 1), String(value) => value);

                Ok(Value::Boolean(text.contains(subtext)))
            }),
            "começa_com" => builtin_fn!(["texto", "prefixo"], |args, _| {
                let text = ensure!(args!(args, 0),
                    String(value) => value
                );

                let prefix = ensure!(args!(args, 1),
                    String(value) => value
                );

                Ok(Value::Boolean(text.starts_with(prefix)))
            }),
            "termina_com" => builtin_fn!(["texto", "sufixo"], |args, _| {
                let text = ensure!(args!(args, 0),
                    String(value) => value
                );

                let suffix = ensure!(args!(args, 1),
                    String(value) => value
                );

                Ok(Value::Boolean(text.ends_with(suffix)))
            }),
            "índice_de" => builtin_fn!(["texto", "subtexto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let subtext = ensure!(args!(args, 1), String(value) => value);
                let index = text.find(subtext).map(|i| i as f64);

                Ok(index.map(Value::Number).unwrap_or(Value::Nil))
            }),
            "repita" => builtin_fn!(["texto", "vezes"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let times = ensure!(args!(args, 1), Number(value) => *value as usize);

                Ok(Value::String(text.repeat(times)))
            }),
            "substitua" => builtin_fn!(["texto", "antigo", "novo"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let old = ensure!(args!(args, 1), String(value) => value);
                let new = ensure!(args!(args, 2), String(value) => value);

                Ok(Value::String(text.replace(old, new)))
            }),
            "corte" => builtin_fn!(["texto", "início", "fim"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let start = ensure!(args!(args, 1), Number(value) => *value as usize);
                let end = ensure!(args!(args, 2), Number(value) => *value as usize);

                if start > end {
                    return runtime_err!(InvalidRangeBounds { bound: end as f64 });
                }

                if end >= text.len() {
                    return runtime_err!(IndexOutOfBounds {
                        index: end,
                        len: text.len()
                    });
                }

                Ok(Value::String(text[start..=end].to_string()))
            }),
            "inverta" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.chars().rev().collect()))
            }),
            "remova" => builtin_fn!(["texto", "início", "tamanho"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let start = ensure!(args!(args, 1), Number(value) => *value as usize);
                let len = ensure!(args!(args, 2), Number(value) => *value as usize);

                if start >= text.len() {
                    return runtime_err!(IndexOutOfBounds {
                        index: start,
                        len: text.len()
                    });
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
            "remova_prefixo" => builtin_fn!(["texto", "prefixo"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let prefix = ensure!(args!(args, 1), String(value) => value);

                if text.starts_with(prefix) {
                    Ok(Value::String(text[prefix.len()..].to_string()))
                } else {
                    Ok(Value::String(text.to_string()))
                }
            }),
            "remova_sufixo" => builtin_fn!(["texto", "sufixo"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);
                let suffix = ensure!(args!(args, 1), String(value) => value);

                if text.ends_with(suffix) {
                    Ok(Value::String(text[..text.len() - suffix.len()].to_string()))
                } else {
                    Ok(Value::String(text.to_string()))
                }
            }),
            "remova_espaços" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(
                    text.chars().filter(|c| !c.is_whitespace()).collect(),
                ))
            }),
            "remova_espaços_início" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.trim_start().to_string()))
            }),
            "remova_espaços_fim" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.trim_end().to_string()))
            }),
            "remova_espaços_início_fim" => builtin_fn!(["texto"], |args, _| {
                let text = ensure!(args!(args, 0), String(value) => value);

                Ok(Value::String(text.trim().to_string()))
            })
        })
    );
}
