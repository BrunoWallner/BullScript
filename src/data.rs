#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    String(String),
    Float(f64),
    Int(i64),
    Bool(bool),
}
impl DataType {
    pub fn add(&self, rhs: Self) -> Option<Self> {
        match self.clone() {
            DataType::String(sl) => match rhs {
                DataType::String(sr) => Some(DataType::String(sl + &sr)),
                DataType::Float(nr) => Some(DataType::String(sl + &format!("{}", nr))),
                DataType::Int(nr) => Some(DataType::String(sl + &format!("{}", nr))),
                _ => None,
            },
            DataType::Float(nl) => match rhs {
                DataType::String(_) => None,
                DataType::Float(nr) => Some(DataType::Float(nl + nr)),
                DataType::Int(nr) => Some(DataType::Float(nl + nr as f64)),
                _ => None,
            },
            DataType::Int(nl) => match rhs {
                DataType::String(_) => None,
                DataType::Float(nr) => Some(DataType::Int(nl + nr as i64)),
                DataType::Int(nr) => Some(DataType::Int(nl + nr)),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn sub(&self, rhs: Self) -> Option<Self> {
        match self.clone() {
            DataType::Float(nl) => match rhs {
                DataType::String(_) => None,
                DataType::Float(nr) => Some(DataType::Float(nl - nr)),
                DataType::Int(nr) => Some(DataType::Float(nl - nr as f64)),
                _ => None,
            },
            DataType::Int(nl) => match rhs {
                DataType::String(_) => None,
                DataType::Float(nr) => Some(DataType::Int(nl - nr as i64)),
                DataType::Int(nr) => Some(DataType::Int(nl - nr)),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn mul(&self, rhs: Self) -> Option<Self> {
        match self.clone() {
            DataType::Float(nl) => match rhs {
                DataType::String(_) => None,
                DataType::Float(nr) => Some(DataType::Float(nl * nr)),
                DataType::Int(nr) => Some(DataType::Float(nl * nr as f64)),
                _ => None,
            },
            DataType::Int(nl) => match rhs {
                DataType::String(_) => None,
                DataType::Float(nr) => Some(DataType::Int(nl * nr as i64)),
                DataType::Int(nr) => Some(DataType::Int(nl * nr)),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn div(&self, rhs: Self) -> Option<Self> {
        match self.clone() {
            DataType::Float(nl) => match rhs {
                DataType::String(_) => None,
                DataType::Float(nr) => Some(DataType::Float(nl / nr)),
                DataType::Int(nr) => Some(DataType::Float(nl / nr as f64)),
                _ => None,
            },
            DataType::Int(nl) => match rhs {
                DataType::String(_) => None,
                DataType::Float(nr) => Some(DataType::Int(nl / nr as i64)),
                DataType::Int(nr) => Some(DataType::Int(nl / nr)),
                _ => None,
            },
            _ => None,
        }
    }
}
// impl std::fmt::Debug for DataType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::String(s) => write!(f, "'{}'", s),
//             Self::Float(n) => write!(f, "{}", n),
//             Self::Int(i) => write!(f, "{}", i),
//             Self::Bool(b) => write!(f, "{}", b),
//         }
//     }
// }

#[derive(Clone, Debug)]
pub enum Value {
    Data(DataType),
    Array(Vec<Value>),
}
impl Value {
    pub fn add(&self, rhs: Value) -> Option<Self> {
        match self.clone() {
            Value::Data(dl) => match rhs {
                Value::Data(dr) => dl.add(dr).map(|d| Value::Data(d)),
                Value::Array(_) => None,
            },
            Value::Array(mut ar) => match rhs {
                Value::Data(dl) => {
                    for a in ar.iter_mut() {
                        *a = a.add(Value::Data(dl.clone()))?;
                    }
                    Some(Value::Array(ar))
                }
                Value::Array(al) => {
                    for (i, a) in ar.iter_mut().enumerate() {
                        *a = a.add(
                            al.get(i)
                                .cloned()
                                .unwrap_or(Value::Data(DataType::Float(0.0))),
                        )?;
                    }
                    Some(Value::Array(ar))
                }
            },
        }
    }
    pub fn sub(&self, rhs: Value) -> Option<Self> {
        match self.clone() {
            Value::Data(dl) => match rhs {
                Value::Data(dr) => dl.sub(dr).map(|d| Value::Data(d)),
                Value::Array(_) => None,
            },
            Value::Array(mut ar) => match rhs {
                Value::Data(dl) => {
                    for a in ar.iter_mut() {
                        *a = a.sub(Value::Data(dl.clone()))?;
                    }
                    Some(Value::Array(ar))
                }
                Value::Array(al) => {
                    for (i, a) in ar.iter_mut().enumerate() {
                        *a = a.sub(
                            al.get(i)
                                .cloned()
                                .unwrap_or(Value::Data(DataType::Float(0.0))),
                        )?;
                    }
                    Some(Value::Array(ar))
                }
            },
        }
    }
    pub fn mul(&self, rhs: Value) -> Option<Self> {
        match self.clone() {
            Value::Data(dl) => match rhs {
                Value::Data(dr) => dl.mul(dr).map(|d| Value::Data(d)),
                Value::Array(_) => None,
            },
            Value::Array(mut ar) => match rhs {
                Value::Data(dl) => {
                    for a in ar.iter_mut() {
                        *a = a.mul(Value::Data(dl.clone()))?;
                    }
                    Some(Value::Array(ar))
                }
                Value::Array(al) => {
                    for (i, a) in ar.iter_mut().enumerate() {
                        *a = a.mul(
                            al.get(i)
                                .cloned()
                                .unwrap_or(Value::Data(DataType::Float(1.0))),
                        )?;
                    }
                    Some(Value::Array(ar))
                }
            },
        }
    }
    pub fn div(&self, rhs: Value) -> Option<Self> {
        match self.clone() {
            Value::Data(dl) => match rhs {
                Value::Data(dr) => dl.div(dr).map(|d| Value::Data(d)),
                Value::Array(_) => None,
            },
            Value::Array(mut ar) => match rhs {
                Value::Data(dl) => {
                    for a in ar.iter_mut() {
                        *a = a.div(Value::Data(dl.clone()))?;
                    }
                    Some(Value::Array(ar))
                }
                Value::Array(al) => {
                    for (i, a) in ar.iter_mut().enumerate() {
                        *a = a.div(
                            al.get(i)
                                .cloned()
                                .unwrap_or(Value::Data(DataType::Float(1.0))),
                        )?;
                    }
                    Some(Value::Array(ar))
                }
            },
        }
    }
}

pub trait IntoValue {
    fn into_value(&self) -> Value;
}
impl IntoValue for &str {
    fn into_value(&self) -> Value {
        Value::Data(DataType::String(self.to_string()))
    }
}
impl IntoValue for f64 {
    fn into_value(&self) -> Value {
        Value::Data(DataType::Float(*self))
    }
}
impl<T: IntoValue> IntoValue for &[T] {
    fn into_value(&self) -> Value {
        let vec = self.iter().map(|x| x.into_value()).collect();
        Value::Array(vec)
    }
}
impl<T: IntoValue, const N: usize> IntoValue for [T; N] {
    fn into_value(&self) -> Value {
        let vec = self.iter().map(|x| x.into_value()).collect();
        Value::Array(vec)
    }
}
