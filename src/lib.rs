//! ```
//! use lazy_static::lazy_static;
//!
//! trait AssocName{
//!     fn name() -> String;
//! }
//!
//! fn generic<T1: AssocName + 'static, T2: AssocName + 'static>() -> &'static str{
//!     lazy_static!{
//!         static ref FORMAT: StaticTypeMap<String> = StaticTypeMap::new();
//!     }
//!     FORMAT.call_once::<(T1, T2), _>(||{
//!         let res = format!("({}, {})", T1::name(), T2::name());
//!         println!("init {}", res);
//!         res
//!     }).as_str()
//! }
//!
//! struct A;
//! struct B;
//! struct C;
//!
//! impl AssocName for A{
//!     fn name() -> String{
//!         "A".to_string()
//!     }
//! }
//!
//! impl AssocName for B{
//!     fn name() -> String{
//!         "B".to_string()
//!     }
//! }
//!
//! impl AssocName for C{
//!     fn name() -> String{
//!         "C".to_string()
//!     }
//! }
//!
//! fn main() {
//!     println!("{}", generic::<A, B>());
//!     println!("{}", generic::<A, C>());
//!     println!("{}", generic::<A, A>());
//!     println!("{}", generic::<B, A>());
//!     println!("{}", generic::<A, B>());
//!     println!("{}", generic::<A, B>());
//!     println!("{}", generic::<B, A>());
//! }
//! ```

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
        // otherwise construct new value and put inside map
        // allocate value on heap
        let boxed = Box::new(f());
        // leak it's value until program is terminated
        let reference: &'static T = Box::leak(boxed);

        let mut writer = self.map.write().unwrap();
        let old = writer.insert(TypeId::of::<Type>(), reference);
        if old.is_some(){
            panic!("StaticTypeMap value was reinitialized. This is a bug.")
        }
        reference
    }
}


#[cfg(test)]
mod tests {
    use super::*;

}
