//! ### Problem
//!
//! lets consider following code:
//!
//! ```
//! use once_cell::sync::OnceCell;
//!
//! trait X{
//!     fn string() -> String;
//! }
//!
//! // having to recompute string() over and over might be expensive (not in this example, but still)
//! // so we use lazy initialization
//! fn generic<T: X>() -> &'static str{
//!     static VALUE: OnceCell<String> = OnceCell::new();
//!
//!     VALUE.get_or_init(||{
//!         T::string()
//!     })
//! }
//!
//! // And now it can be used like this
//! struct A;
//! impl X for A{
//!     fn string() -> String{
//!         "A".to_string()
//!     }
//! }
//!
//! struct B;
//! impl X for B{
//!     fn string() -> String{
//!         "B".to_string()
//!     }
//! }
//!
//! fn main(){
//!     assert_eq!(generic::<A>(), "A");
//!     assert_eq!(generic::<B>(), "A"); // Wait what?
//!     // Not completely behaviour I was expecting
//!     // This is due to fact that static variable placed inside of generic function
//!     // wont be cloned into each version of function, but will be shared
//!     // Thus second call does not initialize value for B, but takes value
//!     // initialized in previous call.
//! }
//! ```
//!
//! ### Solution
//! This crate was designed to solve this particular problem.
//!
//! Lets make some changes:
//!
//! ```
//! use generic_static::StaticTypeMap;
//! use once_cell::sync::OnceCell;
//!
//! trait X{
//!     fn string() -> String;
//! }
//!
//! // having to recompute string() over and over might be expensive (not in this example, but still)
//! // so we use lazy initialization
//! fn generic<T: X + 'static>() -> &'static str{ // T is bound to 'static
//!     static VALUE: OnceCell<StaticTypeMap<String>> = OnceCell::new();
//!     let map = VALUE.get_or_init(|| StaticTypeMap::new());
//!
//!     map.call_once::<T, _>(||{
//!         T::string()
//!     })
//! }
//!
//! // And now it can be used like this
//! struct A;
//! impl X for A{
//!     fn string() -> String{
//!         "A".to_string()
//!     }
//! }
//!
//! struct B;
//! impl X for B{
//!     fn string() -> String{
//!         "B".to_string()
//!     }
//! }
//!
//! fn main(){
//!     assert_eq!(generic::<A>(), "A");
//!     assert_eq!(generic::<B>(), "B");
//! }
//! ```
//!
//! ### Drawbacks
//!
//! Current implementation uses RwLock to make it safe in concurrent
//! applications, which will be slightly slower then regular

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::OnceCell;

pub struct StaticTypeMap<T: 'static> {
    map: RwLock<HashMap<TypeId, &'static OnceCell<T>>>,
}

impl<T: 'static> StaticTypeMap<T> {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize static value corresponding to provided type.
    ///
    /// Initialized value will stay on heap until program terminated.
    /// No drop method will be called.
    pub fn call_once<Type, Init>(&'static self, f: Init) -> &'static T
    where
        Type: 'static,
        Init: FnOnce() -> T,
    {
        // If already initialized, just return stored value
        let cell = {
            let reader = self.map.read().unwrap();
            reader.get(&TypeId::of::<Type>()).cloned() // Clone reference
        };
        if let Some(cell) = cell {
            return cell.get_or_init(f);
        }
        let cell = {
            let mut writer = self.map.write().unwrap();
            let cell = writer
                .entry(TypeId::of::<Type>())
                .or_insert_with(|| {
                    let boxed = Box::new(OnceCell::new());
                    Box::leak(boxed)
                });
            *cell
        };
        cell
            .get_or_init(f)
    }
}

impl<T: 'static> Default for StaticTypeMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadlock_issue4() {
        fn map() -> &'static StaticTypeMap<String> {
            static VALUE: OnceCell<StaticTypeMap<String>> = OnceCell::new();
            VALUE.get_or_init(|| StaticTypeMap::new())
        }

        fn get_u32_value() -> &'static str {
            map().call_once::<u32, _>(|| "u32".to_string())
        }

        let res = map().call_once::<u64, _>(|| format!("{} and", get_u32_value()));

        assert_eq!(res, "u32 and")
    }
}
