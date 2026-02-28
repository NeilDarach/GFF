use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use typst::foundations as t;
use typst_bake::__internal::typst::foundations as tb;
use typst_library::foundations as tl;

#[derive(Default, Serialize, Deserialize)]
pub struct TypstMap<T: tb::IntoValue>(HashMap<String, T>);

impl<T: tb::IntoValue> DerefMut for TypstMap<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T: tb::IntoValue> Deref for TypstMap<T> {
    type Target = HashMap<String, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: tb::IntoValue> From<HashMap<String, T>> for TypstMap<T> {
    fn from(value: HashMap<String, T>) -> Self {
        TypstMap(value)
    }
}

impl<T: tb::IntoValue> t::IntoValue for TypstMap<T> {
    fn into_value(self) -> tb::Value {
        let d = {
            #[allow(unused_mut)]
            let mut map = tl::IndexMap::default();
            for (k, v) in self.0.into_iter() {
                map.insert(
                    k.into(),
                    tl::IntoValue::into_value(tb::IntoValue::into_value(v)),
                );
            }
            tl::Dict::from(map)
        };
        tb::Value::Dict(d)
    }
}

impl<T: tb::IntoValue> TypstMap<T> {
    #[inline]
    #[must_use]
    pub fn into_dict(self) -> tb::Dict {
        {
            #[allow(unused_mut)]
            let mut map = tl::IndexMap::default();
            for (k, v) in self.0.into_iter() {
                map.insert(
                    k.into(),
                    tl::IntoValue::into_value(tb::IntoValue::into_value(v)),
                );
            }

            tl::Dict::from(map)
        }
    }
}

impl<T: t::IntoValue> From<TypstMap<T>> for t::Dict {
    fn from(value: TypstMap<T>) -> Self {
        value.into_dict()
    }
}

impl From<Vec<(&str, &str)>> for TypstMap<String> {
    fn from(value: Vec<(&str, &str)>) -> Self {
        value
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<HashMap<String, String>>()
            .into()
    }
}
