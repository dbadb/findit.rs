# Rust beginners cheatsheet.

- [Option](#option)
  - [Short circuiting](#short-circuiting)
- [Result](#result)
  - [Short circuiting with `?`:](#short-circuiting-with-)
- [Converting String Types](#converting-string-types)
- [Slices](#slices)
- [Fn closures](#fn-closures)
- [Iterators](#iterators)
  - [consuming adaptor (eg sum)](#consuming-adaptor-eg-sum)
  - [iterator adaptor](#iterator-adaptor)
- [Optimizating](#optimizating)
- [Resources](#resources)

---

## Option

[Option template](https://doc.rust-lang.org/nightly/core/option/index.html)

used when a function wants the option to signal invalid return value.

`Some` means there's a value.

`None` means no value.

### Short circuiting

| op                  | description                                                                              |
| :------------------ | :--------------------------------------------------------------------------------------- |
| `?`                 | is short-circuit for check if fail, then return None - _so caller has to be Option too_. |
| `unwrap`            | can trigger panic                                                                        |
| `unwrap_or`         |                                                                                          |
| `unwrap_op_default` |                                                                                          |
| `unwrap_or_else`    |`x.unwrap_or_else(\|\| self.mymeth())` |

Compare:
```rs
fn add_last_numbers(stack: &mut Vec<i32>) -> Option<i32> {
    let a = stack.pop();
    let b = stack.pop();

    match (a, b) {
        (Some(x), Some(y)) => Some(x + y),
        _ => None,
    }
}
```

With:
```r
fn add_last_numbers(stack: &mut Vec<i32>) -> Option<i32> {
    Some(stack.pop()? + stack.pop()?)
}
```

---

## Result

[Result template](https://doc.rust-lang.org/nightly/core/result/index.html)

Provides error signalling-capable return. The __must be used__.

```rs
enum Result<T, E> {
   Ok(T),
   Err(E),
}
```

### Short circuiting with `?`:

replace this:
```rs
fn write_info(info: &Info) -> io::Result<()> 
{
    // Early return on error
    let mut file = match File::create("my_best_friends.txt") {
           Err(e) => return Err(e),
           Ok(f) => f,
    };
    if let Err(e) = file.write_all(format!("name: {}\n", info.name).as_bytes()) {
        return Err(e)
    }
    if let Err(e) = file.write_all(format!("age: {}\n", info.age).as_bytes()) {
        return Err(e)
    }
    if let Err(e) = file.write_all(format!("rating: {}\n", info.rating).as_bytes()) {
        return Err(e)
    }
    Ok(())
}
```

with this
```rs
fn write_info(info: &Info) -> io::Result<()> 
{
    let mut file = File::create("my_best_friends.txt")?;
    // Early return on error
    file.write_all(format!("name: {}\n", info.name).as_bytes())?;
    file.write_all(format!("age: {}\n", info.age).as_bytes())?;
    file.write_all(format!("rating: {}\n", info.rating).as_bytes())?;
    Ok(())
}
```
---

## Converting String Types

https://profpatsch.de/notes/rust-string-conversions

---

## Slices

tbd

---

## Fn closures

> Closures are typically short and relevant only within a narrow context
> rather than in any arbitrary scenario. Within these limited contexts, the
> compiler can infer the types of the parameters and the return type, similar to how it’s able to infer the types of most variables (there are rare cases
> where the compiler needs closure type annotations too).

Closures can capture values from their environment in three ways, which
directly map to the three ways a function can take a parameter: 
1. borrowing immutably
3. borrowing mutably
5. taking ownership

Closures will automatically implement one, two or three of
thse Fn traits in an additive fashion, depending on how the
closures _body_ handles captured values:

* `FnOnce`: can only be called once
* `FnMut`: can modify captures, can be called more than once.
* `Fn`: can't modify captures, can be called more than once without
  mutating their environment - important for eg multiple conconcurrent
  calls. 

> `dyn` refers to dynamic dispatch, relates to trait-objects (interfaces)
> and function pointers. (rather than static/inlined code).

```rs
fn fun_test_impl(value: i32, f: impl Fn(i32) -> i32) -> i32 {
    println!("{}", f(value));
    value
}
fn fun_test_dyn(value: i32, f: &dyn Fn(i32) -> i32) -> i32 {
    println!("{}", f(value));
    value
}
fn fun_test_ptr(value: i32, f: fn(i32) -> i32) -> i32 {
    println!("{}", f(value));
    value
}

fn times2(value: i32) -> i32 {
    2 * value
}

fn main() {
    let y = 2;
    //static dispatch (only borrows)
    fun_test_impl(5, times2);
    fun_test_impl(5, |x| 2*x);
    fun_test_impl(5, |x| y*x);
    //dynamic dispatch
    fun_test_dyn(5, &times2);
    fun_test_dyn(5, &|x| 2*x);
    fun_test_dyn(5, &|x| y*x);
    //C-like pointer to function
    fun_test_ptr(5, times2);
    fun_test_ptr(5, |x| 2*x); //ok: empty capture set
    fun_test_ptr(5, |x| y*x); //error: expected fn pointer, found closure
}
```

```rs
// style variations
fn add_one_v1 (x: u32) -> u32 { x + 1 }
let add_one_v2 = |x: u32| -> u32 { x + 1 };
let add_one_v3 = |x| { x + 1 };
let add_one_v4 = |x| x + 1 ;
```

```rs
// deferred types for borrowed
let example_closure = |x| x;
let s = example_closure(String::from("hello"));
let n = example_closure(5); // <---- error 
```

```rs
// mutable reference capture, no print allowe
// in closure because no other borrows are allowed
// when there's a mutable borrow.
let mut list = vec![1, 2, 3];
println!("Before defining closure: {:?}", list);
let mut borrows_mutably = || list.push(7);
borrows_mutably();
println!("After calling closure: {:?}", list);
 ```
 
 ```rs
// using move to force closure for thread to take ownership
use std::thread;
fn main() {
    let list = vec![1, 2, 3];
    println!("Before defining closure: {:?}", list);
    thread::spawn(move || {
        println!("From thread: {:?}", list)
    }).join().unwrap();
    // println!("(error) after defining closure: {:?}", list);
}
 ```

---

## Iterators

All iterators implement the `Iterator` trait:

```rs
pub trait Iterator {
 type Item;
 fn next(&mut self) -> Option<Self::Item>;
 // methods with default implementations elided
}
```

They are typically useful only as mutable since
their internal state changes on `next()`.

> The `iter` method produces an iterator over _immutable_ 
> references. If we want to create an iterator that 
> takes ownership of v1 and returns owned values, we 
> can call `into_iter` instead of `iter`.  Similarly, if 
> we want to iterate over mutable references, we can 
> call `iter_mut` instead of `iter`.

### consuming adaptor (eg sum)

```rs
let v1 = vec![1, 2, 3];
let v1_iter = v1.iter();
let total: i32 = v1_iter.sum();
assert_eq!(total, 6);
```

### iterator adaptor

These don’t consume the iterator. Instead, they produce different 
iterators by changing some aspect of the original iterator

```rs
let v1: Vec<i32> = vec![1, 2, 3];
let v2: Vec<_> = v1.iter().map(|x| x + 1).collect();
assert_eq!(v2, vec![2, 3, 4]);
```

> You can chain multiple calls to iterator adapters to perform 
> complex actions in a readable way. But because all iterators 
> are lazy, you have to call one of the consuming adapter methods 
> to get results from calls to iterator adapters.

---

## Optimizating

* use iterators instead of loops
* divide loops into chunks

https://www.reidatcheson.com/hpc/architecture/performance/rust/c++/2019/10/19/measure-cache.html

https://docs.rs/noisy_float/latest/noisy_float/


## Resources

* The Rust Programming Language 2nd Edition (Rust 2021)
  Klabnik, Nichols.
* https://play.rust-lang.org/