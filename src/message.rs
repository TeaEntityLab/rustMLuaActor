/*!
Credits(mainly, except Array & reverse conversions):
https://github.com/poga/actix-lua/blob/master/src/message.rs
*/

use std::collections::HashMap;
use std::iter::FromIterator;

use mlua::Result as LuaResult;
use mlua::{FromLua, Lua, MultiValue, ToLua, ToLuaMulti, Value, Variadic};

#[derive(Debug, PartialEq, Clone)]
pub enum LuaMessage {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    Nil,
    Table(HashMap<String, LuaMessage>),
    Array(Vec<LuaMessage>),
    Variadic(VariadicLuaMessage),
}

impl From<bool> for LuaMessage {
    fn from(s: bool) -> Self {
        LuaMessage::Boolean(s)
    }
}
impl From<LuaMessage> for Option<bool> {
    fn from(s: LuaMessage) -> Self {
        match s {
            LuaMessage::String(s) => match s.parse::<bool>() {
                Ok(_x) => Some(_x),
                Err(_e) => None,
            },
            LuaMessage::Integer(i) => Some(i > 0),
            LuaMessage::Number(f) => Some(f > 0_f64),
            LuaMessage::Boolean(b) => Some(b),
            LuaMessage::Nil => None,
            LuaMessage::Table(_h) => Some(!_h.is_empty()),
            LuaMessage::Array(_h) => Some(!_h.is_empty()),
            LuaMessage::Variadic(_h) => Some(!_h.0.is_empty()),
        }
    }
}

impl<'l> From<&'l str> for LuaMessage {
    fn from(s: &'l str) -> Self {
        LuaMessage::String(s.to_string())
    }
}
impl From<String> for LuaMessage {
    fn from(s: String) -> Self {
        LuaMessage::String(s)
    }
}
impl From<LuaMessage> for Option<String> {
    fn from(s: LuaMessage) -> Self {
        match s {
            LuaMessage::String(s) => Some(s),
            LuaMessage::Integer(i) => Some(i.to_string()),
            LuaMessage::Number(f) => Some(f.to_string()),
            LuaMessage::Boolean(b) => Some(b.to_string()),
            LuaMessage::Nil => None,
            LuaMessage::Table(_h) => Some(format!("{:?}", _h)),
            LuaMessage::Array(_h) => Some(format!("{:?}", _h)),
            LuaMessage::Variadic(_h) => Some(format!("{:?}", _h.0)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariadicLuaMessage(Variadic<LuaMessage>);

impl PartialEq<VariadicLuaMessage> for VariadicLuaMessage {
    fn eq(&self, other: &VariadicLuaMessage) -> bool {
        let other = other.0.to_vec();
        return self.0.to_vec().eq(&other);
    }
}

impl From<Variadic<LuaMessage>> for LuaMessage {
    fn from(s: Variadic<LuaMessage>) -> Self {
        LuaMessage::from(
            s.into_iter()
                .map(|v| LuaMessage::from(v.clone()))
                .collect::<Vec<LuaMessage>>(),
        )
    }
}

impl Into<Variadic<LuaMessage>> for LuaMessage {
    fn into(self) -> Variadic<LuaMessage> {
        return Variadic::from_iter([self]);
    }
}

macro_rules! lua_message_number_convert {
    ($x:ty) => {
        impl From<LuaMessage> for Option<$x> {
            fn from(s: LuaMessage) -> Self {
                match s {
                    LuaMessage::String(s) => match s.parse::<$x>() {
                        Ok(_x) => Some(_x),
                        Err(_e) => None,
                    },
                    LuaMessage::Integer(i) => Some(i as $x),
                    LuaMessage::Number(f) => Some(f as $x),
                    LuaMessage::Boolean(b) => {
                        if b {
                            Some(1 as $x)
                        } else {
                            Some(0 as $x)
                        }
                    }
                    LuaMessage::Nil => None,
                    LuaMessage::Table(_h) => None,
                    LuaMessage::Array(_h) => None,
                    LuaMessage::Variadic(_h) => None,
                }
            }
        }
    };
}

macro_rules! lua_message_convert_from_int {
    ($x:ty) => {
        impl From<$x> for LuaMessage {
            fn from(s: $x) -> Self {
                LuaMessage::Integer(<i64>::from(s))
            }
        }
        lua_message_number_convert!($x);
    };
}

lua_message_convert_from_int!(i8);
lua_message_convert_from_int!(u8);
lua_message_convert_from_int!(i16);
lua_message_convert_from_int!(u16);
lua_message_convert_from_int!(i32);
lua_message_convert_from_int!(u32);
lua_message_convert_from_int!(i64);
// lua_message_convert_from_int!(usize);
impl From<usize> for LuaMessage {
    fn from(s: usize) -> Self {
        LuaMessage::Integer(s as i64)
    }
}
lua_message_number_convert!(usize);
// lua_message_convert_from_int!(isize);
impl From<isize> for LuaMessage {
    fn from(s: isize) -> Self {
        LuaMessage::Integer(s as i64)
    }
}
lua_message_number_convert!(isize);

impl From<HashMap<String, LuaMessage>> for LuaMessage {
    fn from(s: HashMap<String, LuaMessage>) -> Self {
        LuaMessage::Table(s)
    }
}
impl From<LuaMessage> for Option<HashMap<String, LuaMessage>> {
    fn from(s: LuaMessage) -> Self {
        match s {
            LuaMessage::String(s) => {
                let mut h = HashMap::default();
                h.insert("x".to_string(), LuaMessage::from(s));
                Some(h)
            }
            LuaMessage::Integer(i) => {
                let mut h = HashMap::default();
                h.insert("x".to_string(), LuaMessage::from(i));
                Some(h)
            }
            LuaMessage::Number(f) => {
                let mut h = HashMap::default();
                h.insert("x".to_string(), LuaMessage::from(f));
                Some(h)
            }
            LuaMessage::Boolean(b) => {
                let mut h = HashMap::default();
                h.insert("x".to_string(), LuaMessage::from(b));
                Some(h)
            }
            LuaMessage::Nil => None,
            LuaMessage::Table(_h) => Some(_h),
            LuaMessage::Array(_h) => {
                let mut new_one = HashMap::default();
                for (k, v) in _h.into_iter().enumerate() {
                    new_one.insert(k.to_string(), v);
                }
                Some(new_one)
            }
            LuaMessage::Variadic(_h) => {
                let mut new_one = HashMap::default();
                for (k, v) in _h.0.into_iter().enumerate() {
                    new_one.insert(k.to_string(), v);
                }
                Some(new_one)
            }
        }
    }
}
impl From<Vec<LuaMessage>> for LuaMessage {
    fn from(s: Vec<LuaMessage>) -> Self {
        LuaMessage::Array(s)
    }
}
impl FromIterator<LuaMessage> for LuaMessage {
    fn from_iter<I: IntoIterator<Item = LuaMessage>>(iter: I) -> Self {
        LuaMessage::Array(Vec::<LuaMessage>::from_iter(iter))
    }
}
impl From<LuaMessage> for Option<Vec<LuaMessage>> {
    fn from(s: LuaMessage) -> Self {
        match s {
            LuaMessage::String(s) => Some(vec![LuaMessage::from(s)]),
            LuaMessage::Integer(i) => Some(vec![LuaMessage::from(i)]),
            LuaMessage::Number(f) => Some(vec![LuaMessage::from(f)]),
            LuaMessage::Boolean(b) => Some(vec![LuaMessage::from(b)]),
            LuaMessage::Nil => None,
            LuaMessage::Table(_h) => {
                let mut new_one = vec![];
                for (_k, v) in _h {
                    new_one.push(LuaMessage::from(_k));
                    new_one.push(v);
                }
                Some(new_one)
            }
            LuaMessage::Array(_h) => Some(_h),
            LuaMessage::Variadic(_h) => Some(_h.0.to_vec()),
        }
    }
}

macro_rules! lua_message_convert_from_float {
    ($x:ty) => {
        impl From<$x> for LuaMessage {
            fn from(s: $x) -> Self {
                LuaMessage::Number(f64::from(s))
            }
        }
        lua_message_number_convert!($x);
    };
}

lua_message_convert_from_float!(f32);
lua_message_convert_from_float!(f64);

impl<'lua> FromLua<'lua> for LuaMessage {
    fn from_lua(v: Value, lua: &'lua Lua) -> LuaResult<LuaMessage> {
        match v {
            Value::String(x) => Ok(LuaMessage::String(String::from_lua(Value::String(x), lua)?)),
            Value::Integer(_) => Ok(LuaMessage::Integer(lua.coerce_integer(v)?.unwrap() as i64)),
            Value::Number(_) => Ok(LuaMessage::Number(lua.coerce_number(v)?.unwrap() as f64)),
            Value::Boolean(b) => Ok(LuaMessage::Boolean(b)),
            Value::Nil => Ok(LuaMessage::Nil),
            Value::Table(t) => {
                if t.len()? > 0
                    && t.clone().pairs::<i32, LuaMessage>().count()
                        == t.clone().sequence_values::<LuaMessage>().count()
                {
                    Ok(LuaMessage::Array(Vec::from_lua(Value::Table(t), lua)?))
                } else {
                    Ok(LuaMessage::Table(HashMap::from_lua(Value::Table(t), lua)?))
                }
            }

            _ => unimplemented!(),
        }
    }
}

impl<'lua> ToLua<'lua> for LuaMessage {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<Value<'lua>> {
        match self {
            LuaMessage::String(x) => Ok(Value::String(lua.create_string(&x)?)),
            LuaMessage::Integer(x) => Ok(Value::Integer(x)),
            LuaMessage::Number(x) => Ok(Value::Number(x)),
            LuaMessage::Boolean(x) => Ok(Value::Boolean(x)),
            LuaMessage::Nil => Ok(Value::Nil),
            LuaMessage::Table(x) => Ok(Value::Table(lua.create_table_from(x)?)),
            LuaMessage::Array(x) => Ok(Value::Table(lua.create_sequence_from(x)?)),
            LuaMessage::Variadic(x) => Ok(Value::Table(lua.create_sequence_from(x.0)?)),
            // You should not create RPCNotifyLater from outside of lua
            // _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MultiLuaMessage(LuaMessage);

impl<'lua> ToLuaMulti<'lua> for MultiLuaMessage {
    fn to_lua_multi(self, lua: &'lua Lua) -> LuaResult<MultiValue<'lua>> {
        match self.0 {
            LuaMessage::Variadic(x) => Ok(x.0.to_lua_multi(lua)?),
            _ => Ok(self.0.to_lua_multi(lua)?),
        }
    }
}

impl Into<MultiLuaMessage> for LuaMessage {
    fn into(self) -> MultiLuaMessage {
        MultiLuaMessage { 0: self }
    }
}

impl From<Vec<LuaMessage>> for MultiLuaMessage {
    fn from(s: Vec<LuaMessage>) -> Self {
        MultiLuaMessage {
            0: LuaMessage::Variadic(VariadicLuaMessage {
                0: Variadic::<LuaMessage>::from_iter(s),
            }),
        }
    }
}

impl FromIterator<LuaMessage> for MultiLuaMessage {
    fn from_iter<I: IntoIterator<Item = LuaMessage>>(iter: I) -> Self {
        MultiLuaMessage {
            0: LuaMessage::Variadic(VariadicLuaMessage(Variadic::<LuaMessage>::from_iter(iter))),
        }
    }
}

impl From<Variadic<LuaMessage>> for MultiLuaMessage {
    fn from(s: Variadic<LuaMessage>) -> Self {
        MultiLuaMessage {
            0: LuaMessage::Variadic(VariadicLuaMessage(s)),
        }
    }
}

impl From<VariadicLuaMessage> for MultiLuaMessage {
    fn from(s: VariadicLuaMessage) -> Self {
        MultiLuaMessage {
            0: LuaMessage::Variadic(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::discriminant;

    #[test]
    fn constructors() {
        assert_eq!(LuaMessage::from(42), LuaMessage::Integer(42));
        assert_eq!(
            LuaMessage::from("foo"),
            LuaMessage::String("foo".to_string())
        );
        assert_eq!(LuaMessage::from(42.5), LuaMessage::Number(42.5));
        assert_eq!(LuaMessage::from(true), LuaMessage::Boolean(true));

        let mut t = HashMap::new();
        t.insert("bar".to_string(), LuaMessage::from("abc"));
        let mut t2 = HashMap::new();
        t2.insert("bar".to_string(), LuaMessage::from("abc"));
        assert_eq!(LuaMessage::from(t), LuaMessage::Table(t2));
    }

    #[test]
    fn to_lua() {
        // we only check if they have the correct variant
        let lua = Lua::new();
        assert_eq!(
            discriminant(&LuaMessage::Integer(42).to_lua(&lua).unwrap()),
            discriminant(&Value::Integer(42))
        );
        assert_eq!(
            discriminant(&LuaMessage::String("foo".to_string()).to_lua(&lua).unwrap()),
            discriminant(&Value::String(lua.create_string("foo").unwrap()))
        );
        assert_eq!(
            discriminant(&LuaMessage::Number(42.5).to_lua(&lua).unwrap()),
            discriminant(&Value::Number(42.5))
        );
        assert_eq!(
            discriminant(&LuaMessage::Boolean(true).to_lua(&lua).unwrap()),
            discriminant(&Value::Boolean(true))
        );
        assert_eq!(
            discriminant(&LuaMessage::Nil.to_lua(&lua).unwrap()),
            discriminant(&Value::Nil)
        );

        let mut t = HashMap::new();
        t.insert("bar".to_string(), LuaMessage::from("abc"));
        assert_eq!(
            discriminant(&LuaMessage::Table(t).to_lua(&lua).unwrap()),
            discriminant(&Value::Table(lua.create_table().unwrap()))
        );
    }

    #[test]
    fn from_lua() {
        // we only check if they have the correct variant
        let lua = Lua::new();
        assert_eq!(
            discriminant(&LuaMessage::from_lua(Value::Integer(42), &lua).unwrap()),
            discriminant(&LuaMessage::Integer(42))
        );
        assert_eq!(
            discriminant(&LuaMessage::from_lua(Value::Number(42.5), &lua).unwrap()),
            discriminant(&LuaMessage::Number(42.5))
        );
        assert_eq!(
            discriminant(
                &LuaMessage::from_lua(Value::String(lua.create_string("foo").unwrap()), &lua)
                    .unwrap()
            ),
            discriminant(&LuaMessage::String("foo".to_string()))
        );
        assert_eq!(
            discriminant(&LuaMessage::from_lua(Value::Boolean(true), &lua).unwrap()),
            discriminant(&LuaMessage::Boolean(true))
        );
        assert_eq!(
            discriminant(&LuaMessage::from_lua(Value::Nil, &lua).unwrap()),
            discriminant(&LuaMessage::Nil)
        );

        let mut t = HashMap::new();
        t.insert("bar".to_string(), LuaMessage::from("abc"));
        assert_eq!(
            discriminant(
                &LuaMessage::from_lua(Value::Table(lua.create_table().unwrap()), &lua).unwrap()
            ),
            discriminant(&LuaMessage::Table(t))
        );

        let t = vec![LuaMessage::from(1), LuaMessage::from(2)];
        assert_eq!(
            discriminant(
                &LuaMessage::from_lua(
                    Value::Table(
                        lua.create_sequence_from(vec!(LuaMessage::from(1), LuaMessage::from(2)))
                            .unwrap()
                    ),
                    &lua
                )
                .unwrap()
            ),
            discriminant(&LuaMessage::Array(t.clone()))
        );

        // println!("{:?}\n{:?}", LuaMessage::Array(t.clone()), LuaMessage::from_lua(
        //     Value::Table(lua.create_sequence_from(vec!(LuaMessage::from(12),LuaMessage::from(2))).unwrap()), &lua
        // ).unwrap());
    }
}
