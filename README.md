# rustMLuaActor

[![tag](https://img.shields.io/github/tag/TeaEntityLab/rustMLuaActor.svg)](https://github.com/TeaEntityLab/rustMLuaActor)
[![Crates.io](https://img.shields.io/crates/d/mlua_actor.svg)](https://crates.io/crates/mlua_actor)
[![Travis CI Build Status](https://api.travis-ci.org/TeaEntityLab/rustMLuaActor.svg?branch=master)](https://travis-ci.org/TeaEntityLab/rustMLuaActor)
[![docs](https://img.shields.io/badge/docs-online-5023dd.svg)](https://docs.rs/mlua_actor/)

[![license](https://img.shields.io/github/license/TeaEntityLab/rustMLuaActor.svg?style=social&label=License)](https://github.com/TeaEntityLab/rustMLuaActor)
[![stars](https://img.shields.io/github/stars/TeaEntityLab/rustMLuaActor.svg?style=social&label=Stars)](https://github.com/TeaEntityLab/rustMLuaActor)
[![forks](https://img.shields.io/github/forks/TeaEntityLab/rustMLuaActor.svg?style=social&label=Fork)](https://github.com/TeaEntityLab/rustMLuaActor)

Lua Actor implementation for Rust (in sync/async modes)

# Why

I love Lua scripting; however it's hard to communicate between Lua & Rust, specially in some async scenarios.

Thus I implemented rustMLuaActor. I hope you would like it :)

# Features

* Actor (*`mlua_actor::actor`*)
  * An Lua actor (sync/async)
  * You could run it on the specific handler (*`fp_rust::handler::HandlerThread`*)

# Dependencies

* [fp_rust](https://crates.io/crates/fp_rust)
* [mlua](https://crates.io/crates/mlua)

# Setup

*`Cargo.toml`*:

```
fp_rust="*"
mlua="*"
mlua_actor="*"
```

# Contribution

Special Thanks to [poga's actix-lua](https://github.com/poga/actix-lua).

Most of *`LuaMessage`* parts are coded by him (except `Array` & reverse conversions).

# Usage

## Actor (sync/async)

Example:
```rust

extern crate mlua;
extern crate fp_rust;
extern crate mlua_actor;

fn main() {

  use mlua::{Variadic};
  use mlua_actor::{actor::Actor, message::LuaMessage};


  fn test_actor(act: Actor) {
      let _ = act.exec_nowait(
          r#"
          i = 1
      "#,
          None,
      );
      assert_eq!(Some(1), Option::from(act.get_global("i").ok().unwrap()));

      let v = act.eval(
          r#"
          3
      "#,
          None,
      );
      assert_eq!(Some(3), Option::from(v.ok().unwrap()));

      act.exec(
          r#"
          function testit (i)
              return i + 1
          end
      "#,
          None,
      ).ok().unwrap();
      match act.call("testit", LuaMessage::from(1)) {
          Ok(_v) => {
              assert_eq!(Some(2), Option::from(_v));
          }
          Err(_err) => {
              println!("{:?}", _err);
              panic!(_err);
          }
      }

      {
          act.def_fn_with_name_nowait(|_, (list1, list2): (Vec<String>, Vec<String>)| {
              // This function just checks whether two string lists are equal, and in an inefficient way.
              // Lua callbacks return `mlua::Result`, an Ok value is a normal return, and an Err return
              // turns into a Lua 'error'.  Again, any type that is convertible to lua may be returned.
              Ok(list1 == list2)
          }, "check_equal").ok().unwrap();
          act.def_fn_with_name_nowait(
              |_, strings: Variadic<String>| {
                  // (This is quadratic!, it's just an example!)
                  Ok(strings.iter().fold("".to_owned(), |a, b| a + b))
              },
              "join",
          ).ok()
          .unwrap();
          assert_eq!(
              Option::<bool>::from(
                  act.eval(r#"check_equal({"a", "b", "c"}, {"a", "b", "c"})"#, None)
                      .ok()
                      .unwrap()
              ).unwrap(),
              true
          );
          assert_eq!(
              Option::<bool>::from(
                  act.eval(r#"check_equal({"a", "b", "c"}, {"d", "e", "f"})"#, None)
                      .ok()
                      .unwrap()
              ).unwrap(),
              false
          );
          assert_eq!(
              Option::<String>::from(act.eval(r#"join("a", "b", "c")"#, None).ok().unwrap())
                  .unwrap(),
              "abc"
          );
      }

      act.set_global(
          "arr1",
          LuaMessage::from(vec![LuaMessage::from(1), LuaMessage::from(2)]),
      ).ok()
      .unwrap();

      let v = Option::<Vec<LuaMessage>>::from(act.get_global("arr1").ok().unwrap());
      assert_eq!(LuaMessage::from(1), v.clone().unwrap()[0]);
      assert_eq!(LuaMessage::from(2), v.clone().unwrap()[1]);
      assert_eq!(2, v.clone().unwrap().len());
  }

  let _ = test_actor(Actor::new_with_handler(None));
  let _ = test_actor(Actor::new());
}

```
