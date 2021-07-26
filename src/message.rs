/*!
Credits(mainly, except Array & reverse conversions):
https://github.com/poga/actix-lua/blob/master/src/message.rs
*/

use std::collections::{HashMap, VecDeque};
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
    Array(VecDeque<LuaMessage>),
    Variadic(VariadicLuaMessage),
}

impl LuaMessage {
    pub fn from_slice<I: IntoIterator<Item = impl Into<LuaMessage>>>(iter: I) -> Self {
        LuaMessage::from(
            iter.into_iter()
                .map(|v| v.into())
                .collect::<Vec<LuaMessage>>(),
        )
    }
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
pub struct VariadicLuaMessage(VecDeque<LuaMessage>);

impl VariadicLuaMessage {
    pub fn push_front(&mut self, value: LuaMessage) {
        self.0.push_front(value);
    }
}

impl PartialEq<VariadicLuaMessage> for VariadicLuaMessage {
    fn eq(&self, other: &VariadicLuaMessage) -> bool {
        let other = &other.0;
        return self.0.eq(other);
    }
}

impl From<VecDeque<LuaMessage>> for LuaMessage {
    fn from(s: VecDeque<LuaMessage>) -> Self {
        LuaMessage::from_iter(s.into_iter())
    }
}

impl Into<VecDeque<LuaMessage>> for VariadicLuaMessage {
    fn into(self) -> VecDeque<LuaMessage> {
        return self.0;
    }
}

impl From<VariadicLuaMessage> for LuaMessage {
    fn from(s: VariadicLuaMessage) -> Self {
        LuaMessage::from_slice(s.0)
    }
}
impl From<Variadic<LuaMessage>> for LuaMessage {
    fn from(s: Variadic<LuaMessage>) -> Self {
        LuaMessage::from(s.into_iter().map(|v| v).collect::<Vec<LuaMessage>>())
    }
}

macro_rules! lua_message_convert_from_collection {
    ($x:tt, $y:ty) => {
        impl From<$x<$y>> for LuaMessage {
            fn from(s: $x<$y>) -> Self {
                LuaMessage::from(
                    s.into_iter()
                        .map(|v| LuaMessage::from(v))
                        .collect::<Vec<LuaMessage>>(),
                )
            }
        }
        impl From<$x<$y>> for MultiLuaMessage {
            fn from(s: $x<$y>) -> Self {
                LuaMessage::from(s).into()
            }
        }
    };
}
macro_rules! lua_message_convert_from_collection_option {
    ($x:tt, $y:ty) => {
        impl From<$x<Option<$y>>> for LuaMessage {
            fn from(s: $x<Option<$y>>) -> Self {
                LuaMessage::from(
                    s.into_iter()
                        .map(|v| match v {
                            Some(s) => LuaMessage::from(s),
                            None => LuaMessage::Nil,
                        })
                        .collect::<Vec<LuaMessage>>(),
                )
            }
        }
        impl From<$x<Option<$y>>> for MultiLuaMessage {
            fn from(s: $x<Option<$y>>) -> Self {
                LuaMessage::from(s).into()
            }
        }
    };
}
macro_rules! lua_message_convert_from_collection_variants_only {
    ($y:ty) => {
        lua_message_convert_from_collection!(Vec, $y);
        lua_message_convert_from_collection!(Variadic, $y);
        lua_message_convert_from_collection_option!(Vec, $y);
        lua_message_convert_from_collection_option!(Variadic, $y);
    };
}
macro_rules! lua_message_convert_from_collection_variants_and_types {
    ($y:ty) => {
        lua_message_convert_from_collection_variants_only!($y);
        impl From<$y> for MultiLuaMessage {
            fn from(s: $y) -> Self {
                LuaMessage::from(s).into()
            }
        }
    };
}
lua_message_convert_from_collection_variants_and_types!(String);
lua_message_convert_from_collection_variants_and_types!(&str);
lua_message_convert_from_collection_variants_and_types!(bool);
lua_message_convert_from_collection_variants_and_types!(i8);
lua_message_convert_from_collection_variants_and_types!(u8);
lua_message_convert_from_collection_variants_and_types!(i16);
lua_message_convert_from_collection_variants_and_types!(u16);
lua_message_convert_from_collection_variants_and_types!(i32);
lua_message_convert_from_collection_variants_and_types!(u32);
lua_message_convert_from_collection_variants_and_types!(i64);
lua_message_convert_from_collection_variants_and_types!(isize);
lua_message_convert_from_collection_variants_and_types!(usize);
lua_message_convert_from_collection_variants_and_types!(f32);
lua_message_convert_from_collection_variants_and_types!(f64);
lua_message_convert_from_collection_variants_and_types!(HashMap<String, LuaMessage>);
lua_message_convert_from_collection_variants_only!(Vec<LuaMessage>);
lua_message_convert_from_collection_variants_only!(VariadicLuaMessage);
lua_message_convert_from_collection_variants_only!(Variadic<LuaMessage>);

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
        LuaMessage::Array(VecDeque::from(s))
    }
}
impl FromIterator<LuaMessage> for LuaMessage {
    fn from_iter<I: IntoIterator<Item = LuaMessage>>(iter: I) -> Self {
        LuaMessage::Array(VecDeque::<LuaMessage>::from_iter(iter))
    }
}
/*
impl<I: IntoIterator<Item = LuaMessage>> From<I> for LuaMessage {
    fn from(s: I) -> Self {
        LuaMessage::Array(Vec::<LuaMessage>::from_iter(s))
    }
}
// */
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
            LuaMessage::Array(_h) => Some(_h.into()),
            LuaMessage::Variadic(_h) => Some(_h.0.into()),
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
    fn from_lua(v: Value<'lua>, lua: &'lua Lua) -> LuaResult<LuaMessage> {
        match v {
            Value::String(x) => Ok(LuaMessage::String(String::from_lua(Value::String(x), lua)?)),
            Value::Integer(_) => Ok(LuaMessage::Integer(lua.coerce_integer(v).unwrap().unwrap())),
            Value::Number(_) => Ok(LuaMessage::Number(lua.coerce_number(v).unwrap().unwrap())),
            Value::Boolean(b) => Ok(LuaMessage::Boolean(b)),
            Value::Nil => Ok(LuaMessage::Nil),
            Value::Table(t) => {
                if t.len()? > 0
                    && t.clone().pairs::<i32, LuaMessage>().count()
                        == t.clone().sequence_values::<LuaMessage>().count()
                {
                    Ok(LuaMessage::Array(VecDeque::from(Vec::from_lua(
                        Value::Table(t),
                        lua,
                    )?)))
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

impl MultiLuaMessage {
    pub fn from_slice<I: IntoIterator<Item = impl Into<LuaMessage>>>(iter: I) -> Self {
        LuaMessage::Variadic(VariadicLuaMessage(
            iter.into_iter()
                .map(|v| v.into())
                .collect::<VecDeque<LuaMessage>>(),
        ))
        .into()
    }

    pub fn push_front(&mut self, value: LuaMessage) {
        match &mut self.0 {
            LuaMessage::Variadic(v) => {
                v.0.push_front(value);
            }
            _ => {
                let mut v = VecDeque::<LuaMessage>::new();
                v.push_front(self.0.clone());
                v.push_front(value);

                self.0 = LuaMessage::Variadic(VariadicLuaMessage(v))
            }
        }
    }
}

impl<'lua> ToLuaMulti<'lua> for MultiLuaMessage {
    fn to_lua_multi(self, lua: &'lua Lua) -> LuaResult<MultiValue<'lua>> {
        match self.0 {
            LuaMessage::Variadic(x) => {
                Ok(Variadic::<LuaMessage>::from_iter(x.0.into_iter()).to_lua_multi(lua)?)
            }
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
                0: VecDeque::<LuaMessage>::from(s),
            }),
        }
    }
}

impl From<VecDeque<LuaMessage>> for MultiLuaMessage {
    fn from(s: VecDeque<LuaMessage>) -> Self {
        MultiLuaMessage {
            0: LuaMessage::Variadic(VariadicLuaMessage { 0: s }),
        }
    }
}

impl FromIterator<LuaMessage> for MultiLuaMessage {
    fn from_iter<I: IntoIterator<Item = LuaMessage>>(iter: I) -> Self {
        MultiLuaMessage {
            0: LuaMessage::Variadic(VariadicLuaMessage(VecDeque::<LuaMessage>::from_iter(iter))),
        }
    }
}

impl From<Variadic<LuaMessage>> for MultiLuaMessage {
    fn from(s: Variadic<LuaMessage>) -> Self {
        MultiLuaMessage {
            0: LuaMessage::Variadic(VariadicLuaMessage(VecDeque::<LuaMessage>::from(s.to_vec()))),
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

macro_rules! impl_tuple {
    () => (
        impl Into<MultiLuaMessage> for () {
            fn into(self) -> MultiLuaMessage {
                MultiLuaMessage::from_iter([])
            }
        }
        impl Into<LuaMessage> for () {
            fn into(self) -> LuaMessage {
                LuaMessage::from_iter([])
            }
        }
    );

    ($last:ident $($name:ident)*) => (
        impl<$($name,)* $last> Into<MultiLuaMessage> for ($($name,)* $last,)
            where $($name: Into<LuaMessage>,)*
                  $last: Into<MultiLuaMessage>
        {
            #[allow(non_snake_case)]
            fn into(self) -> MultiLuaMessage {
                let ($($name,)* $last,) = self;

                let mut v = MultiLuaMessage::from_iter([]);
                let lastOne = $last.into().0;
                v.push_front(lastOne);
                push_reverse!(v, $($name.into(),)*);

                // println!("{:?}", v);
                v
            }
        }
        impl<$($name,)* $last> From<($($name,)* $last,)> for LuaMessage
            where $($name: Into<LuaMessage>,)*
                  $last: Into<LuaMessage>
        {
            #[allow(non_snake_case)]
            fn from(s: ($($name,)* $last,)) -> LuaMessage {
                let ($($name,)* $last,) = s;

                let last_instance = $last.into();

                let mut v = VecDeque::<LuaMessage>::from_iter([]);
                v.push_front(last_instance);
                push_reverse!(v, $($name.into(),)*);

                // println!("{:?}", v);
                LuaMessage::Array(v)
            }
        }
    );
}

macro_rules! push_reverse {
    ($multi_value:expr, $first:expr, $($rest:expr,)*) => (
        push_reverse!($multi_value, $($rest,)*);
        $multi_value.push_front($first);
    );

    ($multi_value:expr, $first:expr) => (
        $multi_value.push_front($first);
    );

    ($multi_value:expr,) => ();
}

impl_tuple!();
impl_tuple!(A);
impl_tuple!(A B);
impl_tuple!(A B C);
impl_tuple!(A B C D);
impl_tuple!(A B C D E);
impl_tuple!(A B C D E F);
impl_tuple!(A B C D E F G);
impl_tuple!(A B C D E F G H);
impl_tuple!(A B C D E F G H I);
impl_tuple!(A B C D E F G H I J);
impl_tuple!(A B C D E F G H I J K);
impl_tuple!(A B C D E F G H I J K L);
impl_tuple!(A B C D E F G H I J K L M);
impl_tuple!(A B C D E F G H I J K L M N);
impl_tuple!(A B C D E F G H I J K L M N O);
impl_tuple!(A B C D E F G H I J K L M N O P);

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
            discriminant(&LuaMessage::Array(VecDeque::from(t)))
        );

        // println!("{:?}\n{:?}", LuaMessage::Array(t.clone()), LuaMessage::from_lua(
        //     Value::Table(lua.create_sequence_from(vec!(LuaMessage::from(12),LuaMessage::from(2))).unwrap()), &lua
        // ).unwrap());
    }
}
