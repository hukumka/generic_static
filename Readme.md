<!-- cargo-sync-readme start -->

### Problem

lets consider following code:

```rust
use once_cell::sync::OnceCell;

trait X{
    fn string() -> String;
}

// having to recompute string() over and over might be expensive (not in this example, but still)
// so we use lazy initialization
fn generic<T: X>() -> &'static str{
    static VALUE: OnceCell<String> = OnceCell::new();

    VALUE.get_or_init(||{
        T::string()
    })
}

// And now it can be used like this
struct A;
impl X for A{
    fn string() -> String{
        "A".to_string()
    }
}

struct B;
impl X for B{
    fn string() -> String{
        "B".to_string()
    }
}

fn main(){
    assert_eq!(generic::<A>(), "A");
    assert_eq!(generic::<B>(), "A"); // Wait what?
    // Not completely behaviour I was expecting
    // This is due to fact that static variable placed inside of generic function
    // wont be cloned into each version of function, but will be shared
    // Thus second call does not initialize value for B, but takes value
    // initialized in previous call.
}
```

### Solution
This crate was designed to solve this particular problem.

Lets make some changes:

```rust
use generic_static::StaticTypeMap;
use once_cell::sync::OnceCell;

trait X{
    fn string() -> String;
}

// having to recompute string() over and over might be expensive (not in this example, but still)
// so we use lazy initialization
fn generic<T: X + 'static>() -> &'static str{ // T is bound to 'static
    static VALUE: OnceCell<StaticTypeMap<String>> = OnceCell::new();
    let map = VALUE.get_or_init(|| StaticTypeMap::new());

    map.call_once::<T, _>(||{
        T::string()
    })
}

// And now it can be used like this
struct A;
impl X for A{
    fn string() -> String{
        "A".to_string()
    }
}

struct B;
impl X for B{
    fn string() -> String{
        "B".to_string()
    }
}

fn main(){
    assert_eq!(generic::<A>(), "A");
    assert_eq!(generic::<B>(), "B");
}
```

### Drawbacks

Current implementation uses RwLock to make it safe in concurrent
applications, which will be slightly slower then regular

<!-- cargo-sync-readme end -->