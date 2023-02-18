# Rust beginners cheatsheet.

- [Option](#option)
  - [Short circuiting](#short-circuiting)
- [Result](#result)
  - [Short circuiting with `?` (return on error)](#short-circuiting-with--return-on-error)
- [Loops](#loops)
  - [iterator loops](#iterator-loops)
- [Converting String Types](#converting-string-types)
- [Slices](#slices)
- [Fn closures](#fn-closures)
- [Iterators](#iterators)
  - [consuming adaptor (eg sum)](#consuming-adaptor-eg-sum)
  - [iterator adaptor](#iterator-adaptor)
- [Rc\<T\> , Weak\<T\>](#rct--weakt)
- [RefCell and Interior Mutability](#refcell-and-interior-mutability)
- [Patterns](#patterns)
  - [match arms](#match-arms)
  - [conditional if/while let](#conditional-ifwhile-let)
  - [destructuring](#destructuring)
  - [multiple patterns](#multiple-patterns)
  - [destructuring enum with match](#destructuring-enum-with-match)
- [Trait Objects](#trait-objects)
  - [Traits as parameters](#traits-as-parameters)
  - [Implementation](#implementation)
  - [Conditional Implementation](#conditional-implementation)
  - [Heterogenous Collections](#heterogenous-collections)
- [Common Traits](#common-traits)
  - [Display](#display)
  - [Drop](#drop)
  - [Copy](#copy)
  - [Clone](#clone)
  - [Debug](#debug)
  - [PartialOrd](#partialord)
  - [Send, Sync (std::marker)](#send-sync-stdmarker)
  - [Summary (example)](#summary-example)
- [Concurrency](#concurrency)
  - [message passing](#message-passing)
  - [Mutex\<T\>](#mutext)
- [Crates, Modules](#crates-modules)
  - [Declaring modules](#declaring-modules)
  - [Declaring submodules](#declaring-submodules)
  - [Paths to code in modules](#paths-to-code-in-modules)
  - [Private vs public](#private-vs-public)
  - [The use keyword](#the-use-keyword)
  - [customizing builds](#customizing-builds)
  - [workspaces](#workspaces)
- [Documenting](#documenting)
- [Optimizing](#optimizing)
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

Use `match` since Result is enum.  Useful methods of 
Result: `is_ok()`, `is_err()`.

### Short circuiting with `?` (return on error)

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

## Loops

### iterator loops

```rs
let v = &["apples", "cake", "coffee"]; // ref to array

for text in v {
    println!("I like {}.", text);
}

let mut sum = 0;
// 1..11 is a range expression, 
// [1..11] is an array with one range expression
for n in 1..11 { 
    sum += n;
}
assert_eq!(sum, 55);
```

---

## Converting String Types

https://profpatsch.de/notes/rust-string-conversions

---

## Slices

https://doc.rust-lang.org/reference/types/slice.html

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

```rs
fn doit(mut args: impl Iterator<Item = String>
{
    while let Some(arg) = args.next() 
    {
        /*d o stuff 
         * can call args.next() within.. 
         * also: if let x = args.next() to protect against end-of-it
         */
    }
}
```

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
## Rc\<T\> , Weak\<T\>

ref-counted smart pointer, aka `shared_ptr`

```rs
use std::cell::RefCell;
use std::rc::{Rc, Weak};

enum List {
 Cons(i32, Rc<List>),
 Nil,
}
use crate::List::{Cons, Nil}; // refer to names in List above
use std::rc::Rc;
fn main() {
   let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
   let b = Cons(3, Rc::clone(&a));
   let c = Cons(4, Rc::clone(&a));
}

#[derive(Debug)]
struct Node {
 value: i32,
 parent: RefCell<Weak<Node>>,
 children: RefCell<Vec<Rc<Node>>>,
}
```

## RefCell and Interior Mutability

* `Rc<T>` enables multiple owners of the same data; Box<T> and RefCell<T>
  have single owners.
* `Box<T>` allows immutable or mutable borrows checked at compile time;
  `Rc<T>` allows only immutable borrows checked at compile time; RefCell<T>
  allows immutable or mutable borrows checked at runtime.
* Because `RefCell<T>` allows mutable borrows checked at runtime, you
  can mutate the value inside the RefCell<T> even when the RefCell<T> is
  immutable.

```rs
#[derive(Debug)]
enum List {
 Cons(Rc<RefCell<i32>>, Rc<List>),
 Nil,
}
use crate::List::{Cons, Nil};
use std::cell::RefCell;
use std::rc::Rc;
fn main() {
 let value = Rc::new(RefCell::new(5));
 let a = Rc::new(Cons(Rc::clone(&value), Rc::new(Nil)));
 let b = Cons(Rc::new(RefCell::new(3)), Rc::clone(&a));
 let c = Cons(Rc::new(RefCell::new(4)), Rc::clone(&a));
 *value.borrow_mut() += 10;
 println!("a after = {:?}", a);
 println!("b after = {:?}", b);
 println!("c after = {:?}", c);
}
```

## Patterns

> Patterns are a special syntax in Rust for matching 
> against the structure of types, both complex and 
> simple. Using patterns in conjunction with match 
> expressions and other constructs gives you more 
> control over a program’s control flow.

### match arms

Matches must be _exhaustive_. Use `_` as default case.

```rs
match VALUE {
 PATTERN => EXPRESSION,
 PATTERN => EXPRESSION,
 PATTERN => EXPRESSION,
}
```

### conditional if/while let

```rs
let mut stack = Vec::new();
stack.push(1);
stack.push(2);
stack.push(3);
while let Some(top) = stack.pop() {
 println!("{top}");
}
```

### destructuring

```rs
let ((feet, inches), Point { x, y }) =
        ((3, 10), Point { x: 3, y: -10 });
```

### multiple patterns

```rs
println!("Multimatcher");
for x in 1..9
{
    match x {
        x if x %5 == 0 => println!("{x} is multiple of 5"),
        1 | 2 => println!("{x} is one or two"),
        3 => println!("{x} is three"),
        4..=6 => println!("{x} is 4-6"),
        _ => println!("{x} is everything else"),
    }
}
```

### destructuring enum with match
```rs
enum Message {
 Quit,
 Move { x: i32, y: i32 },
 Write(String),
 ChangeColor(i32, i32, i32),
}
fn main() {
 let msg = Message::ChangeColor(0, 160, 255);
 match msg {
   Message::Quit => {
     println!("The Quit variant has no data to destructure.");
    }
    Message::Move { x, y } => {
     println!("Move in the x dir {x}, in the y dir {y}");
    }
    Message::Write(text) => {
     println!("Text message: {text}");
    }
    Message::ChangeColor(r, g, b) => {
     println!("Change color to red {r}, green {g}, and blue {b}");
    }
}
```

## Trait Objects

> When we use trait objects, Rust must use dynamic dispatch.

```rs
pub trait Draw
{
    fn draw(&self);
}
```

### Traits as parameters

```rs
pub fn calldraw(item: &impl Draw) {
    item.draw();
}
// sugar-for
pub fn calldraw<T: Draw>(item: &T) {
    item.draw();
}
// multiple-traits
pub fn calldraw<T: Draw + Summary>(item: &T) 

// multiple-traits via where clause, and return traits
pub fn calldraw<T, U>(t: &T, u: &U) -> impl Summary
where
    T: Draw + Summary
    U: Clone + Debug
```

### Implementation

```rs
impl Summary for Tweet {
 fn summarize_author(&self) -> String {
    format!("@{}", self.username)
 }
}
```

### Conditional Implementation
```rs
impl<T:Display> ToString for T {}
```

### Heterogenous Collections
```rs
drawable: Vec<Box<dyn Draw>>
```

## Common Traits

### Display

### Drop
```rs
impl Drop for CustomSmartPointer {
 fn drop(&mut self) {
   println!("Dropping CustomSmartPointer with data `{}`!",
     self.data);
 }
}
```

### Copy

### Clone

### Debug

```rs
#[derive(Debug)]
struct Rectangle {
 width: u32,
 height: u32,
}
```

### PartialOrd

### Send, Sync (std::marker)

> The Send marker trait indicates that ownership of values of 
> the type implementing Send can be transferred between threads.
> notably, Rc\<T\> can't be sent.

>The Sync marker trait indicates that it is safe for the type implementing Sync
>to be referenced from multiple threads. In other words, any type T is Sync if
>&T (an immutable reference to T) is Send, meaning the reference can be sent
>safely to another thread. Similar to Send, primitive types are Sync, and types
>composed entirely of types that are Sync are also Sync.

### Summary (example)

```rs
pub trait Summary {
 fn summarize(&self) -> String
 {
    return String::from("default summary");
 }
}
```

---
## Concurrency

```rs
use std::thread;
fn main() {
 let v = vec![1, 2, 3]; // move closure
 let handle = thread::spawn(move || {
    println!("Here's a vector: {:?}", v);
 });
 handle.join().unwrap();
}
```

### message passing

```rs
use std::sync::mpsc;
use std::thread;
fn main() {
 let (tx, rx) = mpsc::channel();
 thread::spawn(move || {
    let val = String::from("hi");
    tx.send(val).unwrap();
 });
 let received = rx.recv().unwrap();
 println!("Got: {received}");
}
```

### Mutex\<T\>

> As you might suspect, Mutex<T> is a smart pointer. More accurately, 
> the call to lock returns a smart pointer called MutexGuard, 
> wrapped in a LockResult that we handled with the call to unwrap.
> The MutexGuard smart pointer implements Deref to point at our 
> inner data; the smart pointer also has a Drop implementation 
> that releases the lock automatically when a MutexGuard goes
> out of scope, which happens at the end of the inner scope.
```rs
println!("Mutex/shared memory -------------");
// NB: Rc not safe to share, use Arc (atomic refcount)
let counter = Arc::new(Mutex::new(0)); 
let mut handles = vec![];
for _ in 0..10
{
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move ||
    {
        // inner mutability via Mutex!
        let mut num = counter.lock().unwrap(); 
        *num += 1;
    });
    handles.push(handle);
}

for handle in handles
{
    handle.join().unwrap();
}
println!("Result: {}", *counter.lock().unwrap());
```
---

## Crates, Modules

How do we refer to other pieces of code. (Intra-crate, Inter-crate).

Start from the crate root When compiling a crate, the compiler first looks in
the crate root file (usually src/lib.rs for a library crate or src/main.rs for 
a binary crate) for code to compile.  

### Declaring modules 

In the crate root file, you can declare new modules; say you declare a “garden” 
module with `mod garden;`. The compiler will look for the module’s code in 
these places:

* Inline, within curly brackets that replace the semicolon following mod garden
* In the file src/garden.rs
* In the file src/garden/mod.rs

### Declaring submodules 

In any file other than the crate root, you can declare submodules. For 
example, you might declare `mod vegetables;` in src/garden.rs.

The compiler will look for the submodule’s code within the directory named for
the parent module in these places:

* Inline, directly following mod vegetables, within curly brackets instead of
the semicolon
* In the file src/garden/vegetables.rs
* In the file src/garden/vegetables/mod.rs

### Paths to code in modules 

Once a module is part of your crate, you can refer to code in that module 
from anywhere else in that same crate, as long as the privacy rules allow, 
using the path to the code. For example, an Asparagus type in the garden 
vegetables module would be found at `crate::garden::vegetables::Asparagus`.

### Private vs public 

Code within a module is private from its parent modules by default. To make 
a module public, declare it with `pub mod` instead of mod. To make items within 
a public module public as well, use `pub` before their declarations.

### The use keyword 

Within a scope, the `use` keyword creates shortcuts to items to reduce 
repetition of long paths. In any scope that can refer to 
`crate::garden::vegetables::Asparagus`, you can create a shortcut with 
use `crate::garden::vegetables::Asparagus;` and from then on you only 
need to write Asparagus to make use of that type in the scope.

### customizing builds

https://doc.rust-lang.org/cargo/reference/profiles.html

```ini
[profile.dev]
opt-level = 0
[profile.release]
opt-level = 3
```

### workspaces

We use workspaces to encourage separation amongst sibling submodules. 
For example, we'd like to test a parser independent of runtime.
Each subdir of a workspace is an independent Cargo context but
the external dependencies are shared through the root-level 
`Cargo.lock`.   Circular dependencies between siblings may work
but is probably a sign of poor factoring.

```ini
[workspace]
members = [
    "src/parser",
    "src/ast",
    "src/runtime",
    "src/main",
]
```
Intra-workspace dependencies must be expressed in submodules' `Cargo.toml`.

```ini
[dependencies]
ast = {path = "../ast"}
```

---
## Documenting

```rs
//! # My Crate
//!   
//!   here are docs for my crate.
```

```rs
/// Adds one to the number given.
///
/// # Examples
///
/// ```
/// let arg = 5;
/// let answer = my_crate::add_one(arg);
///
/// assert_eq!(6, answer);
/// ```
pub fn add_one(x: i32) -> i32 { return x+1; }
```
---

## Optimizing

* use iterators instead of loops
* divide loops into chunks
* cargo can express per-build optimization settings eg, for profiling.

https://www.reidatcheson.com/hpc/architecture/performance/rust/c++/2019/10/19/measure-cache.html

https://docs.rs/noisy_float/latest/noisy_float/


## Resources

* https://doc.rust-lang.org/reference/introduction.html
* The Rust Programming Language 2nd Edition (Rust 2021)
  Klabnik, Nichols.
* https://play.rust-lang.org/