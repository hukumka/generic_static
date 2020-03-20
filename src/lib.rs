//! ### Problem
//!
//! lets consider following code:
//!
//! ```
//! use std::sync::Once;
//!
//! trait X{
//!     fn string() -> String;
//! }
//!
//! // having to recompute string() over and over might be expensive (not in this example, but still)
//! // so we use lazy initialization
//! fn generic<T: X>() -> &'static str{
//!     static mut VALUE: Option<String> = None;
//!     static INIT: Once = Once::new();
//!
//!     unsafe{
//!         INIT.call_once(||{
//!             VALUE = Some(T::string());
//!         });
//!         VALUE.as_ref().unwrap().as_str()
//!     }
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
//! use std::sync::Once;
//!
//! trait X{
//!     fn string() -> String;
//! }
//!
//! // having to recompute string() over and over might be expensive (not in this example, but still)
//! // so we use lazy initialization
//! fn generic<T: X + 'static>() -> &'static str{ // T is bound to 'static
//!     static mut VALUE: Option<StaticTypeMap<String>> = None;
//!     static INIT: Once = Once::new();
//!
//!     let map = unsafe{
//!         INIT.call_once(||{
//!             VALUE = Some(StaticTypeMap::new());
//!         });
//!         VALUE.as_ref().unwrap()
//!     };
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

use std::sync::RwLock;
use std::any::TypeId;
use std::collections::HashMap;


pub struct StaticTypeMap<T: 'static>{
    map: RwLock<HashMap<TypeId, &'static T>>
}

pub struct Entry<Type: 'static>{
    _marker: std::marker::PhantomData<Type>
}

impl<T: 'static> StaticTypeMap<T>{
    pub fn new() -> Self{
        Self{map: RwLock::new(HashMap::new())}
    }

    /// Initialize static value corresponding to provided type.
    ///
    /// Initialized value will stay on heap until program terminated.
    /// No drop method will be called.
    pub fn call_once<Type, Init>(&'static self, f: Init) -> &'static T
        where Type: 'static, Init: FnOnce() -> T
    {
        // If already initialized, just return stored value
        {
            let reader = self.map.read().unwrap();
            if let Some(ref reference) = reader.get(&TypeId::of::<Type>()){
                return &reference;
            }
        }
        let mut writer = self.map.write().unwrap();
        let reference = writer.entry(TypeId::of::<Type>())
            .or_insert_with(|| {
                // otherwise construct new value and put inside map
                // allocate value on heap
                let boxed = Box::new(f());
                // leak it's value until program is terminated
                let reference: &'static T = Box::leak(boxed);
                reference
            });
        reference
    }
}


#[cfg(test)]
mod tests {
    use super::*;

}
