use crate::parser::{Operator, ParserNode};
use num_complex::Complex64;
use std::{collections::HashMap, fmt, ops};
use Value::*;
use std::time::Instant;

#[derive(Debug)]
pub enum Value {
    Number(Complex64),
    Vector(f64, f64),
    Array(Vec<Value>)
}

type ValueOutput = Result<Value, String>;

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Number(n) => match other {
                Number(n2) => n.re == n2.re && n.im == n2.im,
                _ => false,
            },
            Vector(x, y) => match other {
                Vector(x2, y2) => x == x2 && y == y2,
                _ => false,
            }
            Array(_) => false
        }
    }
}

impl Eq for Value { }

impl ops::Add<Value> for Value {
    type Output = ValueOutput;

    fn add(self, rhs: Value) -> Self::Output {
        match self {
            Number(c) => match rhs {
                Number(c2) => Ok(Number(c + c2)),
                Vector(_, _) => Err("cannot add a number to a vector".into()),
                Array(_) => Ok(rhs.push(self))
            },
            Vector(x, y) => match rhs {
                Number(_) => Err("cannot add a vector to a number".into()),
                Vector(x2, y2) => Ok(Vector(x + x2, y + y2)),
                Array(_) => Ok(rhs.push(self)),
            },
            Array(_) => Ok(self.push(rhs))
        }
    }
}

impl ops::Sub<Value> for Value {
    type Output = ValueOutput;

    fn sub(self, rhs: Value) -> Self::Output {
        match self {
            Number(c) => match rhs {
                Number(c2) => Ok(Number(c - c2)),
                Vector(_, _) => Err("cannot subtract a number from a vector".into()),
                Array(_) => Err("cannot subtract an array from a number".into()),
            },
            Vector(x, y) => match rhs {
                Number(_) => Err("cannot subtract a vector from a number".into()),
                Vector(x2, y2) => Ok(Vector(x - x2, y - y2)),
                Array(_) => Err("cannot subtract an array from a vector".into()),
            },
            Array(_) => match rhs {
                Number(_) => Err("cannot subtract a number from an array".into()),
                Vector(_, _) => Err("cannot subtract a vector from an array".into()),
                Array(_) => Err("cannot subtract an array from an array".into())
            }
        }
    }
}

impl ops::Mul<Value> for Value {
    type Output = ValueOutput;

    fn mul(self, rhs: Value) -> Self::Output {
        match self {
            Number(c) => match rhs {
                Number(c2) => Ok(Number(c * c2)),
                Vector(x, y) => if c.im != 0.0 {
                    Err("cannot multiply a vector with a complex number".into())
                }
                else {
                    Ok(Vector(x * c.re, y * c.re))
                },
                Array(_) => Err("cannot multiply a number by an array".into())
            },
            Vector(x, y) => match rhs {
                Number(c) => if c.im != 0.0 {
                    Err("cannot multiply a vector with a complex number".into())
                }
                else {
                    Ok(Vector(x * c.re, y * c.re))
                }
                Vector(_, _) => Err("cannot multiply a vector with a vector. use dot(vector, vector) or cross(vector, vector) instead".into()),
                Array(_) => Err("cannot multiply an array with a vector".into())
            },
            Array(_) => match rhs {
                Number(_) => Err("cannot multiply an array by a number".into()),
                Vector(_, _) => Err("cannot multiply an array with a vector".into()),
                Array(_) => Err("cannot multiply an array by an array".into())
            }
        }
    }
}

impl ops::Div<Value> for Value {
    type Output = ValueOutput;

    fn div(self, rhs: Value) -> Self::Output {
        match self {
            Number(c) => match rhs {
                Number(c2) => Ok(Number(c / c2)),
                Vector(x, y) => {
                    if c.im != 0.0 {
                        Err("cannot divide a vector by a complex number".into())
                    } else {
                        Ok(Vector(x / c.re, y / c.re))
                    }
                },
                Array(_) => Err("cannot divide a number by an array".into())
            },
            Vector(x, y) => match rhs {
                Number(c) => {
                    if c.im != 0.0 {
                        Err("cannot divide a vector by a complex number".into())
                    } else {
                        Ok(Vector(x / c.re, y / c.re))
                    }
                }
                Vector(_, _) => Err("cannot divide a vector by a vector".into()),
                Array(_) => Err("cannot divide a vector by an array".into())
            },
            Array(_) => match rhs {
                Number(_) => Err("cannot divide an array by a number".into()),
                Vector(_, _) => Err("cannot divide an array by a vector".into()),
                Array(_) => Err("cannot divide an array by an array".into())
            }
        }
    }
}

impl ops::Rem<Value> for Value {
    type Output = ValueOutput;

    fn rem(self, rhs: Value) -> Self::Output {
        use Value::*;

        match self {
            Number(c) => match rhs {
                Number(c2) => Ok(Number(c % c2)),
                Vector(_, _) => Err("cannot find remainder between number and vector".into()),
                Array(_) => Err("cannot find remainder of number in terms of array".into())
            },
            Vector(_, _) => match rhs {
                Number(_) => Err("cannot find remainder between vector and number".into()),
                Vector(_, _) => Err("cannot find remainder between vector and vector".into()),
                Array(_) => Err("cannot find remainder between vector and array".into())
            },
            Array(_) => match rhs {
                Number(_) => Err("cannot find remainder between arraay and number".into()),
                Vector(_, _) => Err("cannot find remainder between array and number".into()),
                Array(_) => Err("cannot find remainder between array and array".into())
            }
        }
    }
}

impl Value {
    fn pow(self, rhs: Value) -> ValueOutput {
        match self {
            Number(c) => match rhs {
                Number(c2) => {
                    if c.im == 0.0 && c.re == 0.0 {
                        Ok(Value::real(0.0))
                    } else if c2.im == 0.0 && c2.re == 0.0 {
                        Ok(Value::real(1.0))
                    } else {
                        Ok(Number(c.powc(c2)))
                    }
                }
                Vector(_, _) => Err("cannot raise a number to a vector power".into()),
                Array(_) => Err("cannot raise a number to an array power".into())
            },
            Vector(x, y) => match rhs {
                Number(c) => {
                    if c.im != 0.0 {
                        Err("cannot raise vector to a complex power".into())
                    } else {
                        Ok(Vector(x.powf(c.re), y.powf(c.re)))
                    }
                }
                Vector(_, _) => Err("cannot raise a vector to a vector power".into()),
                Array(_) => Err("cannot raise a vector to an array power".into())
            },
            Array(_) => match rhs {
                Number(_) => Err("cannot raise array to a number power".into()),
                Vector(_, _) => Err("cannot raise array to a vector power".into()),
                Array(_) => Err("cannot raise array to an array power".into())
            }
        }
    }

    // why are other inequalities a Result, but equals is not?
    // answer: other inequalities are literally undefined for different data types
    // but equals will work for all. a vector being greater than an imaginary is undefined,
    // but a vector being equal to an imaginary is very clearly false.
    fn equals(self, rhs: Value) -> Value {
        match self == rhs {
            true => Value::real(1.0),
            false => Value::real(0.0)
        }
    }

    fn greater_than(self, rhs: Value) -> ValueOutput {
        match self {
            Number(c) => match rhs {
                Number(c2) => Ok(if c.norm() > c2.norm() {
                    Number(Complex64::new(1.0, 0.0))
                } else {
                    Number(Complex64::new(0.0, 0.0))
                }),
                Vector(_, _) => Err("cannot compare greater-than between a number and vector".into()),
                Array(_) => Err("cannot compare greater-than between a number and array".into())
            },
            Vector(_, _) => match rhs {
                Number(_) => Err("cannot compare greater-than between a vector and a number".into()),
                Vector(_, _) => Err("cannot compare greater-than between a vector and a vector".into()),
                Array(_) => Err("cannot compare greater-than between a vector and an array".into())
            },
            Array(_) => match rhs {
                Number(_) => Err("cannot compare greater-than between an array and a number".into()),
                Vector(_, _) => Err("cannot compare greater-than between an array and a vector".into()),
                Array(_) => Err("cannot compare greater-than between an array and an array".into())
            }
        }
    }

    fn less_than(self, rhs: Value) -> ValueOutput {
        match self {
            Number(c) => match rhs {
                Number(c2) => Ok(if c.norm() < c2.norm() {
                    Number(Complex64::new(1.0, 0.0))
                } else {
                    Number(Complex64::new(0.0, 0.0))
                }),
                Vector(_, _) => Err("cannot compare less-than between a number and a vector".into()),
                Array(_) => Err("cannot compare less-than between a number and an array".into())
            },
            Vector(_, _) => match rhs {
                Number(_) => Err("cannot compare less-than between a vector and a number".into()),
                Vector(_, _) => Err("cannot compare less-than between a vector and a vector".into()),
                Array(_) => Err("cannot compare less-than between a vector and an array".into())
            },
            Array(_) => match rhs {
                Number(_) => Err("cannot compare less-than between an array and a number".into()),
                Vector(_, _) => Err("cannot compare less-than between an array and a vector".into()),
                Array(_) => Err("cannot compare less-than between an array and an array".into())
            }
        }
    }

    fn greater_than_or_equals(self, rhs: Value) -> ValueOutput {
        match self {
            Number(c) => match rhs {
                Number(c2) => Ok(if c.norm() >= c2.norm() {
                    Number(Complex64::new(1.0, 0.0))
                } else {
                    Number(Complex64::new(0.0, 0.0))
                }),
                Vector(_, _) => Err("cannot compare greater-than-or-equals between a number and vector".into()),
                Array(_) => Err("cannot compare greater-than-or-equals between a number and an array".into())
            },
            Vector(_, _) => match rhs {
                Number(_) => Err("cannot compare greater-than-or-equals between a vector and a number".into()),
                Vector(_, _) => Err("cannot compare greater-than-or-equals between a vector and a vector".into()),
                Array(_) => Err("cannot compare greater-than-or-equals between a vector and an array".into())
            },
            Array(_) => match rhs {
                Number(_) => Err("cannot compare greater-than-or-equals between an array and a number".into()),
                Vector(_, _) => Err("cannot compare greater-than-or-equals between an array and a vector".into()),
                Array(_) => Err("cannot compare greater-than-or-equals between an array and an array".into())
            }
        }
    }

    fn less_than_or_equals(self, rhs: Value) -> ValueOutput {
        match self {
            Number(c) => match rhs {
                Number(c2) => Ok(if c.norm() <= c2.norm() {
                    Number(Complex64::new(1.0, 0.0))
                } else {
                    Number(Complex64::new(0.0, 0.0))
                }),
                Vector(_, _) => Err("cannot compare less-than-or-equals between a number and vector".into()),
                Array(_) => Err("cannot compare less-than-or-equals between a number and an array".into())
            },
            Vector(_, _) => match rhs {
                Number(_) => Err("cannot compare less-than-or-equals between a vector and a number".into()),
                Vector(_, _) => Err("cannot compare less-than-or-equals between a vector and a vector".into()),
                Array(_) => Err("cannot compare less-than-or-equals between a vector and an array".into())
            },
            Array(_) => match rhs {
                Number(_) => Err("cannot compare less-than-or-equals between an array and a number".into()),
                Vector(_, _) => Err("cannot compare less-than-or-equals between an array and a vector".into()),
                Array(_) => Err("cannot compare less-than-or-equals between an array and an array".into())
            }
        }
    }

    fn expect_real<'a>(&self, message: &'a str) -> Result<f64, &'a str> {
        match self {
            Number(c) => {
                if c.im == 0.0 {
                    Ok(c.re)
                } else {
                    Err(message)
                }
            }
            _ => Err(message),
        }
    }

    fn expect_complex<'a>(&self, message: &'a str) -> Result<Complex64, &'a str> {
        match self {
            Number(c) => Ok(*c),
            _ => Err(message),
        }
    }

    fn expect_vector<'a>(&self, message: &'a str) -> Result<(f64, f64), &'a str> {
        match self {
            Vector(x, y) => Ok((*x, *y)),
            _ => Err(message),
        }
    }

    fn expect_array<'a>(&self, message: &'a str) -> Result<&Vec<Value>, &'a str> {
        match self {
            Array(arr) => Ok(arr),
            _ => Err(message)
        }
    }

    fn mem_size(&self) -> usize {
        let value_size = std::mem::size_of::<Value>();

        match self {
            Array(arr) => {
                let mut size = value_size;

                for value in arr {
                    size += value.mem_size();
                }

                size
            },
            _ => value_size
        }
    }

    fn real(r: f64) -> Self {
        Number(Complex64::new(r, 0.0))
    }

    fn imaginary(i: f64) -> Self {
        Number(Complex64::new(0.0, i))
    }

    fn push(mut self, val: Value) -> Value {
        if let Array(ref mut arr) = self {
            arr.push(val);
        }
        self
    }

    fn gamma(self) -> ValueOutput {
        const P: [f64; 8] = [
            676.5203681218851, -1259.1392167224028,
            771.32342877765313, -176.61502916214059,
            12.507343278686905, -0.13857109526572012,
            9.9843695780195716e-6, 1.5056327351493116e-7
        ];

        let pi = Complex64::new(std::f64::consts::PI, 0.0);
        let mut c = self.expect_complex("cannot calculate gamma for non-number")?;

        if c.re < 0.5 {
            Ok(Value::Number(pi / (((pi * c).sin()) * (1.0 - c))).gamma()?)
        } else {
            let mut x = Complex64::new(0.99999999999980993, 0.0);
            c -= 1.0;

            for i in 0..P.len() {
                x += P[i] / (c + (i as f64) + 1.0)
            }

            let t = c + (P.len() as f64) - 0.5;
            Ok(Value::Number((2.0 * pi).sqrt() * t.powc(c + 0.5) * (-t).exp() * x))
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number(c) => {
                if c.im == 0.0 {
                    write!(f, "{}", c.re)
                } else if c.re == 0.0 {
                    if c.im == 1.0 {
                        write!(f, "i")
                    } else {
                        write!(f, "{}i", c.im)
                    }
                } else {
                    if c.im > 0.0 {
                        write!(f, "{} + {}i", c.re, c.im)
                    } else {
                        write!(f, "{} - {}i", c.re, c.im * -1.0)
                    }
                }
            }
            Vector(x, y) => write!(f, "({}, {})", x, y),
            Array(arr) => {
                let elements = arr.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>();

                write!(f, "[{}]", elements.join(", "))
            }
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Number(n) => Number(*n),
            Vector(x, y) => Vector(*x, *y),
            Array(arr) => Array(arr.clone())
        }
    }
}

struct BuiltinFunction {
    parameter_count: usize,
    body: fn(&[Value], &RuntimeState) -> Result<Value, String>,
}

impl BuiltinFunction {
    fn new(params: usize, closure: fn(&[Value], &RuntimeState) -> Result<Value, String>) -> Self {
        BuiltinFunction {
            parameter_count: params,
            body: closure,
        }
    }
}

struct RuntimeState<'a> {
    globals: HashMap<&'a str, Value>,
    locals: HashMap<&'a str, Value>,
    functions: HashMap<&'a str, &'a ParserNode<'a>>,
    builtin_functions: HashMap<&'a str, BuiltinFunction>,
    in_function: bool,
    start_instant: Instant
}

impl<'a> RuntimeState<'a> {
    fn new() -> Self {
        RuntimeState {
            globals: HashMap::new(),
            locals: HashMap::new(),
            functions: HashMap::new(),
            builtin_functions: HashMap::new(),
            in_function: false,
            start_instant: Instant::now() // this will be set later
        }
    }

    fn add_default_globals_and_functions(&mut self) {
        self.add_global("pi", Value::real(std::f64::consts::PI));
        self.add_global("e", Value::real(std::f64::consts::E));

        self.add_builtin(
            "vec",
            BuiltinFunction::new(2, |params, _| {
                let x = params[0].expect_real("the x component of the vector is not a number")?;
                let y = params[1].expect_real("the y component of the vector is not a number")?;
                Ok(Vector(x, y))
            }),
        );

        self.add_builtin(
            "x",
            BuiltinFunction::new(1, |params, _| {
                let vec = params[0].expect_vector("expected vector to take x component out of")?;
                Ok(Value::real(vec.0))
            }),
        );

        self.add_builtin(
            "y",
            BuiltinFunction::new(1, |params, _| {
                let vec = params[0].expect_vector("expected vector to take y component out of")?;
                Ok(Value::real(vec.1))
            }),
        );

        self.add_builtin(
            "sin",
            BuiltinFunction::new(1, |params, _| {
                let num = params[0].expect_complex("expected number to find sine of")?;
                Ok(Number(num.sin()))
            }),
        );

        self.add_builtin(
            "cos",
            BuiltinFunction::new(1, |params, _| {
                let num = params[0].expect_complex("expected number to find cosine of")?;
                Ok(Number(num.cos()))
            }),
        );

        self.add_builtin(
            "tan",
            BuiltinFunction::new(1, |params, _| {
                let num = params[0].expect_complex("expected number to find tangent of")?;
                Ok(Number(num.tan()))
            }),
        );

        self.add_builtin(
            "log",
            BuiltinFunction::new(1, |params, _| {
                let num = params[0].expect_complex("expected number to find logarithm of")?;
                Ok(Number(num.log(10.0)))
            }),
        );

        self.add_builtin(
            "logn",
            BuiltinFunction::new(2, |params, _| {
                let n = params[0].expect_real("expected real base to logarithm")?;
                let x = params[1].expect_complex("expected number to find logarithm of")?;
                Ok(Number(x.log(n)))
            }),
        );

        self.add_builtin(
            "ln",
            BuiltinFunction::new(1, |params, _| {
                let num =
                    params[0].expect_complex("expected number to find natural logarithm of")?;
                Ok(Number(num.ln()))
            }),
        );

        self.add_builtin(
            "print",
            BuiltinFunction::new(1, |params, _| {
                println!("{}", params[0]);
                Ok(params[0].clone())
            }),
        );

        self.add_builtin(
            "conjugate",
            BuiltinFunction::new(1, |params, _| {
                let num =
                    params[0].expect_complex("expected a complex number to find conjugate of")?;
                Ok(Number(Complex64::new(num.re, -num.im)))
            }),
        );

        self.add_builtin(
            "Re",
            BuiltinFunction::new(1, |params, _| {
                let num = params[0]
                    .expect_complex("expected a complex number to find real part of")?;

                Ok(Value::real(num.re))
            }),
        );

        self.add_builtin(
            "Im",
            BuiltinFunction::new(1, |params, _| {
                let num = params[0]
                    .expect_complex("expected a complex number to find imaginary part of")?;

                Ok(Value::real(num.im))
            }),
        );

        self.add_builtin(
            "len",
            BuiltinFunction::new(1, |params, _| {
                let array =
                    params[0].expect_array("expected an array to find length of")?;
                Ok(Value::real(array.len() as f64))
            }),
        );

        self.add_builtin(
            "rm",
            BuiltinFunction::new(2, |params, _| {
                let mut array = params[0].expect_array("expected an array to remove value from")?.clone();
                let index = params[1].expect_real("expected a real number to index array with in rm(x, y)")?;

                if index.fract() != 0.0 || index < 0.0 || index >= array.len() as f64 {
                    Err(format!("cannot index array in rm(x, y) where y is {}", index))
                } else {
                    array.remove(index as usize);
                    Ok(Value::Array(array))
                }
            })
        );

        self.add_builtin(
            "ins",
            BuiltinFunction::new(3, |params, _| {
                let mut array = params[0].expect_array("expected an array to remove value from")?.clone();
                let index = params[1].expect_real("expected a real number to index array with in ins(x, y, z)")?;
                let value = params[2].clone();

                if index.fract() != 0.0 || index < 0.0 || index >= array.len() as f64 {
                    Err(format!("cannot index array in ins(x, y, z) where y is {}", index))
                } else {
                    array.insert(index as usize, value);
                    Ok(Value::Array(array))
                }
            })
        );

        self.add_builtin(
            "mem",
            BuiltinFunction::new(1, |params, _| {
                Ok(Value::real(params[0].mem_size() as f64))
            })
        );

        self.add_builtin(
            "clock",
            BuiltinFunction::new(1, |params, state| {
                let time = params[0].expect_real("expected real number in clock(x)")?;
                Ok(Value::real(state.start_instant.elapsed().as_secs_f64() - time))
            })
        );
    }

    fn add_global(&mut self, name: &'a str, value: Value) {
        self.globals.insert(name, value);
    }

    fn add_local(&mut self, name: &'a str, value: Value) {
        self.locals.insert(name, value);
    }

    fn remove_local(&mut self, name: &'a str) {
        self.locals.remove(name);
    }

    fn has_local(&self, name: &'a str) -> bool {
        self.locals.contains_key(name)
    }

    fn has_global(&self, name: &'a str) -> bool {
        self.globals.contains_key(name)
    }

    fn add_function(&mut self, name: &'a str, body: &'a ParserNode) {
        self.functions.insert(name, body);
    }

    fn add_builtin(&mut self, name: &'a str, function: BuiltinFunction) {
        self.builtin_functions.insert(name, function);
    }

    fn has_function(&self, name: &'a str) -> bool {
        self.functions.contains_key(name) || self.builtin_functions.contains_key(name)
    }

    fn evaluate(&mut self, node: &'a ParserNode<'a>) -> Result<Value, String> {
        match node {
            ParserNode::Number(num, imaginary) => {
                if *imaginary {
                    Ok(Value::imaginary(*num))
                } else {
                    Ok(Value::real(*num))
                }
            }
            ParserNode::Identifier(identifier) => {
                if self.has_local(identifier) {
                    Ok(self.locals[identifier].clone())
                } else if self.has_global(identifier) {
                    Ok(self.globals[identifier].clone())
                } else {
                    return Err(format!("unknown variable: {}", identifier));
                }
            }
            ParserNode::Operation(left, operator, right) => {
                let left = self.evaluate(&*left)?;
                let right = self.evaluate(&*right)?;

                Ok(match operator {
                    Operator::Add => (left + right)?,
                    Operator::Subtract => (left - right)?,
                    Operator::Multiply => (left * right)?,
                    Operator::Divide => (left / right)?,
                    Operator::Power => left.pow(right)?,
                    Operator::Modulo => (left % right)?,
                    Operator::Equals => left.equals(right),
                    Operator::GreaterThan => left.greater_than(right)?,
                    Operator::LessThan => left.less_than(right)?,
                    Operator::GreaterThanOrEquals => left.greater_than_or_equals(right)?,
                    Operator::LessThanOrEquals => left.less_than_or_equals(right)?,
                })
            }
            ParserNode::FunctionCall(name, arguments) => {
                if !self.has_function(name) {
                    return Err(format!("unknown function: {}", name));
                }

                self.in_function = true;

                if self.builtin_functions.contains_key(name) {
                    if arguments.len() != self.builtin_functions[name].parameter_count {
                        return Err(format!(
                            "{} expects {} parameters, but only {} were supplied",
                            name,
                            self.builtin_functions[name].parameter_count,
                            arguments.len()
                        ));
                    }

                    let evaluated_arguments: Vec<Result<Value, String>> = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect();

                    for argument in evaluated_arguments.iter() {
                        if let Err(_) = argument {
                            return argument.clone();
                        }
                    }

                    let mut evaluated_arguments: Vec<Value> = evaluated_arguments
                        .into_iter()
                        .map(|argument| argument.unwrap())
                        .collect();

                    return (self.builtin_functions[name].body)(&mut evaluated_arguments, &self);
                }

                let mut functions = self
                    .functions
                    .iter()
                    .filter(|function| *function.0 == *name)
                    .map(|function| *function.1)
                    .collect::<Vec<&ParserNode>>();

                if let ParserNode::FunctionDeclaration(_, parameters, body) =
                functions.pop().unwrap()
                {
                    if arguments.len() != parameters.len() {
                        return Err(format!(
                            "{} expects {} parameters, but only {} were supplied",
                            name,
                            parameters.len(),
                            arguments.len()
                        ));
                    }

                    let mut preserved_locals = HashMap::new();

                    for parameter in parameters.iter() {
                        if self.has_local(parameter) {
                            preserved_locals.insert(*parameter, self.locals[parameter].clone());
                        }
                    }

                    for i in 0..arguments.len() {
                        let arg_value = self.evaluate(&arguments[i])?;
                        self.add_local(parameters[i], arg_value);
                    }

                    let result = self.evaluate(&*body)?;

                    for i in 0..parameters.len() {
                        let parameter = parameters[i];

                        if !preserved_locals.contains_key(parameter) {
                            self.remove_local(parameter);
                        } else {
                            self.add_local(parameter, preserved_locals[parameter].clone());
                        }
                    }

                    self.in_function = false;

                    return Ok(result);
                } else {
                    unreachable!()
                }
            }
            ParserNode::Conditional(predicate, true_expr, false_expr) => {
                let predicate = self
                    .evaluate(&*predicate)?
                    .expect_real("a predicate to a conditional expression must be a number")?;

                if predicate != 0.0 {
                    return Ok(self.evaluate(&*true_expr)?);
                } else {
                    return Ok(self.evaluate(&*false_expr)?);
                }
            }
            ParserNode::FunctionDeclaration(name, _, _) => {
                if self.has_function(name) {
                    return Err(format!(
                        "redeclared a function that already is defined: {}",
                        name
                    ));
                }

                self.add_function(name, node);

                Ok(Value::real(0.0))
            }
            ParserNode::VariableDeclaration(name, expression) => {
                if self.has_global(name) || self.has_local(name) {
                    return Err(format!("you cannot redeclare a variable: {}", name));
                }

                let value = self.evaluate(&*expression)?;

                if self.in_function {
                    self.add_local(name, value);
                } else {
                    self.add_global(name, value);
                }

                Ok(Value::real(0.0))
            }
            ParserNode::Loop(parameter, range, body) => {
                if let ParserNode::Range(first, second, step) = &**range
                /* :S */
                {
                    let first_bound = self
                        .evaluate(&*first)?
                        .expect_real("the first bound must be a real number")?;
                    let second_bound = self
                        .evaluate(&*second)?
                        .expect_real("the second bound must be a real number")?;
                    let step = self
                        .evaluate(&*step)?
                        .expect_real("the step must be a number")?;

                    if step == 0.0 {
                        return Err("a step cannot be 0".to_string());
                    }

                    let preserved_local = if self.has_local(parameter) {
                        Some(self.locals[parameter].clone())
                    } else {
                        None
                    };

                    let mut x = first_bound;
                    let mut sum = Value::real(0.0);

                    if first_bound < second_bound {
                        while x < second_bound {
                            self.add_local(parameter, Value::real(x));
                            sum = (sum + self.evaluate(&*body)?)?;
                            x = if x + step < second_bound {
                                x + step
                            } else {
                                self.add_local(parameter, Value::real(second_bound));
                                sum = (sum + self.evaluate(&*body)?)?;
                                break;
                            };
                        }
                    } else {
                        while x > second_bound {
                            self.add_local(parameter, Value::real(x));
                            sum = (sum + self.evaluate(&*body)?)?;

                            x = if x - step > second_bound {
                                x - step
                            } else {
                                self.add_local(parameter, Value::real(second_bound));
                                sum = (sum + self.evaluate(&*body)?)?;
                                break;
                            };
                        }
                    }

                    if let Some(val) = preserved_local {
                        self.add_local(parameter, val);
                    }

                    Ok(sum)
                } else {
                    unreachable!()
                }
            }
            ParserNode::Assignment(identifiers, expression) => {
                let expression = self.evaluate(expression)?;

                for identifier in identifiers.iter() {
                    if !self.has_local(identifier) && !self.has_global(identifier) {
                        return Err(format!("use of undefined variable: {}", identifier));
                    }

                    if self.in_function && self.has_global(identifier) && !self.has_local(identifier ) {
                        return Err(format!("attempted to affect external variable {} from within a function", identifier));
                    }

                    self.add_local(identifier, expression.clone());
                }

                Ok(expression)
            }
            ParserNode::Factorial(expression) => {
                let c = self.evaluate(expression)?
                    .expect_complex("attempted to find factorial of non-number")?;

                if c.im == 0.0 && c.re > 0.0 && c.re.fract() == 0.0 {
                    let mut factorial = c.re as i64;

                    for n in (2..c.re as i64).rev() {
                        factorial *= n;
                    }

                    return Ok(Value::real(factorial as f64));
                } else if c.re == 0.0 && c.im == 0.0 {
                    return Ok(Value::real(1.0));
                } else {
                    Value::Number(c + 1.0).gamma()
                }
            }
            ParserNode::Tree(nodes) => {
                if nodes.is_empty() {
                    return Ok(Value::real(0.0));
                }

                match nodes.last().unwrap() {
                    ParserNode::VariableDeclaration(_, _)
                    | ParserNode::FunctionDeclaration(_, _, _) => {
                        Err("a tree must end with an expression".to_owned())
                    }
                    _ => {
                        let mut last_evaluated = Value::real(0.0);
                        let mut new_locals = Vec::new();

                        for node in nodes.into_iter() {
                            last_evaluated = self.evaluate(node)?;

                            if let ParserNode::VariableDeclaration(name, _) = node {
                                new_locals.push(name);
                            } else if let ParserNode::FunctionDeclaration(name, _, _) = node {
                                new_locals.push(name);
                            }
                        }

                        for local in new_locals {
                            self.remove_local(local);
                        }

                        return Ok(last_evaluated);
                    }
                }
            },
            ParserNode::Array(expressions) => {
                let mut evaluated_expressions = Vec::new();

                for expression in expressions {
                    evaluated_expressions.push(self.evaluate(expression)?);
                }

                Ok(Array(evaluated_expressions))
            },
            ParserNode::Index(array, index) => {
                let array = self.evaluate(array)?;
                let array = array.expect_array("cannot index a non-array")?;

                let index = self.evaluate(index)?.expect_real("tried to index using non-number")?;

                if index.fract() != 0.0 {
                    return Err("cannot index arrays with non-integers".to_owned())
                }

                if index as usize >= array.len() || index < 0.0 {
                    return Err(format!("attempted to index array of length {} with index {}", array.len(), index))
                }

                Ok(array[index as usize].clone())
            }
            ParserNode::Range(_, _, _) => unreachable!()
        }
    }
}

pub fn execute(root: ParserNode) -> Result<Value, String> {
    let mut runtime = RuntimeState::new();
    runtime.add_default_globals_and_functions();
    runtime.start_instant = Instant::now();
    runtime.evaluate(&root)
}