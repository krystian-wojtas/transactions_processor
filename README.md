# Purpose

Engine to process transactions and print final state of accounts.

# Operations

- deposit(client, tx, amount): deposit given `amount` to `client` account
  recording `tx` transaction
- withdrawal(client, tx, amount): withdrawal given `amount` from `client`
account recording `tx` transaction
- dispute(client, tx): get amount from recorded `tx` transaction and hold it on
`client` account
- resolve(client, tx): get amount from recorded `tx` transaction and allow
  unhold it which means let `client` to use it again
- chargeback(client, tx): get amount from recorded `tx` transaction and
  withdrawal it from `client` account. Account is then locked and any further
  deposit or withdrawal are not allowed.

# Input

``` sh
% cat transactions.txt
type,       client,  tx, amount
deposit,         1,   1,    1.0
deposit,         1,   2,    1.0
deposit,         2,   3,    10.0
withdrawal,      2,   4,    5.0
```

# Output

Prints states of each client account after processing series of input
transactions. Extra field `total` is sum of `available` and `held`.
``` sh
% cargo run --quiet -- transactions.txt
client, available, held, total, locked
1,2.0,0.0,2.0,false
2,5.0,0.0,5.0,false
```

Any error goes to standard output stream.

# Representation

Input data is text parsed from csv format. Amount could have up to 4 fractional
digits. It is stored as unsignet integer on 64 bits. It gives maksimum amount
`1844674407370955.1615`. Any operations on client account cannot violate this
limit. If so transaction is not applied and error is returned.

# Assumptions

- Transactions are recorded for both `deposit` and `withdrawal`. If there is any
dispute, then it can operate on both of them.
- Transactions are recorded even if `deposit` or `withdrawal` operations do not
  succeed. Then transaction `tx` cannot be reused as it always have to be unique.
- If client account is locked because of chargeback, then `deoposit` and
`withdrawal` cannot happen. It is still ok to `dispute`, `resolve` and any
further `chargeback`.
- If transaction exceeds limits of internal representation, then it is not
applied.
- `total` is calculated only when printing clients accounts as sum of
  `available` and `held`. If `total` exceeds limit of internal representation,
  then warning is printed on error stream and printed `total` contains only
  `available`.
- `dispute` cannot be applied to transaction which is already disputed.

# Multithreading

Application is single threaded. But api behind allows to use it in multithreaded
applications as proper locking mechanism are implemented.

In single threaded application locking only gives extra performance cost and complexity. It could
only benefit in implemening in some kind of web server and serving customers in parallel.

# Testing

``` sh
cargo test
```

# Track error

Some errors gives chain where it come from.
``` sh
% cargo run --quiet -- file-does-not-exist.txt
Error: cannot read input file: "file-does-not-exist.txt", reason: Error(Io(Os { code: 2, kind: NotFound, message: "No such file or directory" }))

Caused by:
    0: No such file or directory (os error 2)
    1: No such file or directory (os error 2)
```

To have backtrace use nigthly toolchain

``` sh
% RUST_BACKTRACE=1 cargo +nightly run --quiet -- file-does-not-exist.txt
Error: cannot read input file: "file-does-not-exist.txt", reason: Error(Io(Os { code: 2, kind: NotFound, message: "No such file or directory" }))

Caused by:
    0: No such file or directory (os error 2)
    1: No such file or directory (os error 2)

Stack backtrace:
   0: anyhow::error::<impl core::convert::From<E> for anyhow::Error>::from
             at /home/k/.cargo/registry/src/github.com-1ecc6299db9ec823/anyhow-1.0.43/src/error.rs:519:25
   1: <core::result::Result<T,F> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible,E>>>::from_residual
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/core/src/result.rs:1915:27
   2: transactions_processor::process
             at ./src/lib.rs:22:19
   3: transactions_processor::main
             at ./src/main.rs:16:23
   4: core::ops::function::FnOnce::call_once
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/core/src/ops/function.rs:227:5
   5: std::sys_common::backtrace::__rust_begin_short_backtrace
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/sys_common/backtrace.rs:125:18
   6: std::rt::lang_start::{{closure}}
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/rt.rs:63:18
   7: core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/core/src/ops/function.rs:259:13
   8: std::panicking::try::do_call
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/panicking.rs:403:40
   9: std::panicking::try
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/panicking.rs:367:19
  10: std::panic::catch_unwind
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/panic.rs:129:14
  11: std::rt::lang_start_internal::{{closure}}
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/rt.rs:45:48
  12: std::panicking::try::do_call
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/panicking.rs:403:40
  13: std::panicking::try
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/panicking.rs:367:19
  14: std::panic::catch_unwind
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/panic.rs:129:14
  15: std::rt::lang_start_internal
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/rt.rs:45:20
  16: std::rt::lang_start
             at /rustc/30a0a9b694cde95cbab863f7ef4d554f0f46b606/library/std/src/rt.rs:62:5
  17: main
  18: __libc_start_main
  19: _start
  ```

Bactrack feature is only in nightly to stabilize. [issue](https://github.com/rust-lang/rust/issues/53487).
