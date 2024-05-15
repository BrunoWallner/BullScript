use crate::data::DataType;

use super::Value;

pub fn call_inbuilt(name: &str, arguments: Vec<Value>) -> Result<Option<Value>, ()> {
    let fns: Vec<&dyn Function> = vec![&Print {}, &Sin {}];
    for f in fns {
        if f.name() == name {
            return Ok(f.call(arguments))
        }
    }

    // match name {
    //     "print" => return Ok(None),
    //     _ => (),
    // };
    Err(())
}

trait Function {
    fn name(&self) -> &'static str;
    fn call(&self, args: Vec<Value>) -> Option<Value>;
}

struct Print {}
impl Function for Print {
    fn name(&self) -> &'static str {
        "print"
    }

    fn call(&self, args: Vec<Value>) -> Option<Value> {
        let mut string = String::new();
        for a in args {
            match a {
                Value::Data(d) => string.push_str(&format!("{:?}", d).trim_matches('\'')),
                // Value::Array(a) => string.push_str(&format!("{:?}", a)),
                Value::Array(a) => string.push_str(&format!("{:?}", a)),
            }
        }
        println!("{}", string);
        None
    }
}
struct Sin {}
impl Function for Sin {
    fn name(&self) -> &'static str {
        "sin"
    }

    fn call(&self, args: Vec<Value>) -> Option<Value> {
        let Some(v) = args.get(0) else {
            return None
        };
        match v {
            Value::Data(d) => match d {
                DataType::Float(n) => Some(Value::Data(DataType::Float(n.sin()))),
                _ => None
            },
            Value::Array(_) => None,
        }
    }
}
