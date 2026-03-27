# Install Rust

curl https://sh.rustup.rs -sSf | sh

cd C:\

cargo new chint_ats

cd chint_ats


C:\chint_ats\          ← Dossier du projet
│

├── Cargo.toml                      ← Fichier de configuration

│

├── src\

│   └── main.rs                     ← code Rust

│

└── index.html                      ← Fichier HTML

http://localhost:5000


[package]
name = "chint_ats"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
actix-files = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serialport = "4"

---------------------------------------------------------------

error[E0432]: unresolved import `serialport::prelude`
 --> src\main.rs:6:17
  |
6 | use serialport::prelude::*;
  |                 ^^^^^^^ could not find `prelude` in `serialport`

warning: unused import: `Deserialize`
 --> src\main.rs:3:24
  |
3 | use serde::{Serialize, Deserialize};
  |                        ^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

error[E0308]: mismatched types
   --> src\main.rs:134:25
    |
133 |         (0x0006, "v1a", |x| format!("{} V", x)),
    |                         --- the expected closure
134 |         (0x0007, "v1b", |x| format!("{} V", x)),
    |                         ^^^^^^^^^^^^^^^^^^^^^^ expected closure, found a different closure
    |
    = note: expected closure `{closure@src\main.rs:133:25: 133:28}`
               found closure `{closure@src\main.rs:134:25: 134:28}`
    = note: no two closures, even if identical, have the same type
    = help: consider boxing your closure and/or using it as a trait object

error[E0308]: mismatched types
   --> src\main.rs:135:25
    |
133 |         (0x0006, "v1a", |x| format!("{} V", x)),
    |                         --- the expected closure
134 |         (0x0007, "v1b", |x| format!("{} V", x)),
135 |         (0x0008, "v1c", |x| format!("{} V", x)),
    |                         ^^^^^^^^^^^^^^^^^^^^^^ expected closure, found a different closure
    |
    = note: expected closure `{closure@src\main.rs:133:25: 133:28}`
               found closure `{closure@src\main.rs:135:25: 135:28}`
    = note: no two closures, even if identical, have the same type
    = help: consider boxing your closure and/or using it as a trait object

error[E0308]: mismatched types
   --> src\main.rs:136:25
    |
133 |         (0x0006, "v1a", |x| format!("{} V", x)),
    |                         --- the expected closure
...
136 |         (0x0009, "v2a", |x| format!("{} V", x)),
    |                         ^^^^^^^^^^^^^^^^^^^^^^ expected closure, found a different closure
    |
    = note: expected closure `{closure@src\main.rs:133:25: 133:28}`
               found closure `{closure@src\main.rs:136:25: 136:28}`
    = note: no two closures, even if identical, have the same type
    = help: consider boxing your closure and/or using it as a trait object

error[E0308]: mismatched types
   --> src\main.rs:137:25
    |
133 |         (0x0006, "v1a", |x| format!("{} V", x)),
    |                         --- the expected closure
...
137 |         (0x000A, "v2b", |x| format!("{} V", x)),
    |                         ^^^^^^^^^^^^^^^^^^^^^^ expected closure, found a different closure
    |
    = note: expected closure `{closure@src\main.rs:133:25: 133:28}`
               found closure `{closure@src\main.rs:137:25: 137:28}`
    = note: no two closures, even if identical, have the same type
    = help: consider boxing your closure and/or using it as a trait object

error[E0308]: mismatched types
   --> src\main.rs:138:25
    |
133 |         (0x0006, "v1a", |x| format!("{} V", x)),
    |                         --- the expected closure
...
138 |         (0x000B, "v2c", |x| format!("{} V", x)),
    |                         ^^^^^^^^^^^^^^^^^^^^^^ expected closure, found a different closure
    |
    = note: expected closure `{closure@src\main.rs:133:25: 133:28}`
               found closure `{closure@src\main.rs:138:25: 138:28}`
    = note: no two closures, even if identical, have the same type
    = help: consider boxing your closure and/or using it as a trait object

error[E0308]: mismatched types
   --> src\main.rs:139:27
    |
133 |         (0x0006, "v1a", |x| format!("{} V", x)),
    |                         --- the expected closure
...
139 |         (0x000C, "swVer", |x| format!("{:.2}", x as f32 / 100.0)),
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected closure, found a different closure
    |
    = note: expected closure `{closure@src\main.rs:133:25: 133:28}`
               found closure `{closure@src\main.rs:139:27: 139:30}`
    = note: no two closures, even if identical, have the same type
    = help: consider boxing your closure and/or using it as a trait object

error[E0282]: type annotations needed
   --> src\main.rs:140:27
    |
140 |         (0x0015, "cnt1", |x| x.to_string()),
    |                           ^  - type must be known at this point
    |
help: consider giving this closure parameter an explicit type
    |
140 |         (0x0015, "cnt1", |x: /* Type */| x.to_string()),
    |                            ++++++++++++

error[E0282]: type annotations needed
   --> src\main.rs:141:27
    |
141 |         (0x0016, "cnt2", |x| x.to_string()),
    |                           ^  - type must be known at this point
    |
help: consider giving this closure parameter an explicit type
    |
141 |         (0x0016, "cnt2", |x: /* Type */| x.to_string()),
    |                            ++++++++++++

error[E0308]: mismatched types
   --> src\main.rs:142:29
    |
133 |         (0x0006, "v1a", |x| format!("{} V", x)),
    |                         --- the expected closure
...
142 |         (0x0017, "runtime", |x| format!("{} h", x)),
    |                             ^^^^^^^^^^^^^^^^^^^^^^ expected closure, found a different closure
    |
    = note: expected closure `{closure@src\main.rs:133:25: 133:28}`
               found closure `{closure@src\main.rs:142:29: 142:32}`
    = note: no two closures, even if identical, have the same type
    = help: consider boxing your closure and/or using it as a trait object

Some errors have detailed explanations: E0282, E0308, E0432.
For more information about an error, try `rustc --explain E0282`.
warning: `chint_ats` (bin "chint_ats") generated 1 warning
error: could not compile `chint_ats` (bin "chint_ats") due to 10 previous errors; 1 warning emitted

C:\Users\thier\Downloads\Rust-ATS\chint_ats>
