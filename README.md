# EnglishCode

**A programming language in plain English. Write code like you speak, and build real applications.**

EnglishCode lets you write programs, databases, web servers, APIs, and user
authentication using natural English sentences. There is no syntax to memorize
beyond the words you already use. Under the hood it compiles down to Rust and
SQLite, so programs are fast and data is persisted on disk.

```englishcode
show "Hello, World!"
```

```bash
engcode run hello.eng
```

```
→ Running hello.eng

Hello, World!

Program completed successfully!
```

---

## Table of Contents

- [Quick Start](#quick-start)
- [Core Language](#core-language)
- [Databases](#databases)
- [Authentication](#authentication)
- [Web Server & APIs](#web-server--apis)
- [Server-Side Templating](#server-side-templating)
- [Mobile](#mobile)
- [Architecture](#architecture)
- [Running Tests](#running-tests)
- [Examples](#examples)
- [Known Limitations](#known-limitations)
- [Contributing](#contributing)
- [Support](#support)

---

## Quick Start

### Requirements

- [Rust](https://www.rust-lang.org/) (1.70 or newer)
- macOS or Linux

### Install

```bash
git clone https://github.com/ikaydreams-dev/OpenSoftware.git
cd OpenSoftware
cargo build --release
cp target/release/engcode /usr/local/bin/
```

### Your first program

Create `hello.eng`:

```
show "Hello, World!"
```

Run it:

```bash
engcode run hello.eng
```

The CLI has three commands:

| Command | Purpose |
|---------|---------|
| `engcode run <file>.eng` | Parse, analyze, and execute a program |
| `engcode mobile init/build/run-ios/run-android` | Generate and build mobile apps |
| `engcode version` | Print the version |

---

## Core Language

### Variables

```
set name to "Alice"
set age to 30
let score = 95.5
make active to true
store count to 100
save total to 500
```

### Data types

Numbers, strings, booleans, `null`, arrays, and objects:

```
set colors to ["red", "green", "blue"]
set first to colors[0]

set user to {name: "Alice", age: 25}
show user.name
```

### Control flow

```
if score greaterthan 90 then
  show "A"
else if score greaterthan 80 then
  show "B"
else
  show "C"
end

while counter lessthan 10
  set counter to counter plus 1
end

for i from 1 to 5
  show i
end

for each color in colors
  show color
end
```

Comparison words: `greaterthan`, `lessthan`, `equalto`, `and`, `or`, `not`.

### Functions

```
define function greet with name
  show "Hello, {name}!"
  return "done"
end
```

### Error handling

```
try
  throw "Something went wrong"
catch
  show "Caught an error"
end
```

### Comments

```
# This is a comment
show "Hi"  # inline comments work too
```

### String methods

You can chain methods on strings and arrays:

```
set text to "  Hello World  "
set upper to text.uppercase().trim()
set words to text.split(" ")

set items to [3, 1, 2]
items.push(4)
set total to items.length()
```

Methods available include `uppercase`, `lowercase`, `trim`, `split`, `replace`,
`length`, `contains`, `sort`, `reverse`, `first`, `last`, `push`, and `pop`.

### Output

```
show "Result: {total}"
```

Strings support `{variable}` interpolation.

---

## Databases

EnglishCode ships with a built-in SQLite database. No setup, drivers, or
connection strings are required — databases are created and stored in the
`databases/` directory.

### Create a database and collections

```
create a database called "MyApp"

create these collections in it
  users
  products
  orders
```

> Note: declaring collections is idempotent. Each run resets the listed
> collections before your inserts, so re-running a script never duplicates rows.

### Insert

`insert`, `add`, and `put` all add rows:

```
insert into users with name "Alice" and age 28 and active true
add products with title "Laptop" and price 999.99 and stock 50
put into orders with user_id 1 and total 199.50
```

### Select

`select`, `get`, `find`, and `fetch` all read data:

```
select all from users
select all from users where age greaterthan 18
```

### Update / Delete with WHERE clauses

`update`, `change`, and `modify` filter rows with `where`:

```
update users with age 29 where name equalto "Alice"
delete from users where active equalto false
remove from products where price lessthan 10
```

Full CRUD with `where age greaterthan 18` style filters is supported.

---

## Authentication

Built-in `signup` / `login` / `logout` persists users to the current database:

```
signup "sarah" with email "sarah@test.com" and password "secret123"
login "sarah" with password "secret123"
logout
```

After `login`, the interpreter tracks `current_user`, `logged_in`, and
`session_token`. Passwords are hashed before storage, and session tokens are
issued for logged-in users.

---

## Web Server & APIs

Create a server and register routes with plain English:

```
create a web server on port 3000

add route get "/api/users" returning "User list"
add route get "/api/data" returning rows from users
add route post "/api/users" returning "User created"

start the server
```

- **Static routes** return a JSON message: `{"message": "..."}`
- **Data routes** (`returning rows from <collection>`) query the live database
  on every request and return the rows as JSON: `{"data": [...]}`

`start the server` runs until you press `Ctrl+C` (graceful shutdown). To have
the program finish on its own, give it a lifetime:

```
start the server for 10 seconds
```

---

## Server-Side Templating

Any HTML page can embed live database data with `{{collection_name}}` markers.
When the page is served, the marker is replaced with an HTML table of that
collection's rows.

```
create a web server on port 3000
create a database called "MyApp"
create these collections in it
  users
insert into users with name "Alice" and age 28

create page called "home"
add heading "Our users"
add paragraph "{{users}}"
render page "home"

add route get "/api/users" returning rows from users
start the server
```

Visiting `http://localhost:3000/` serves `public/home.html` with a rendered
table of the `users` collection. Visiting `/api/users` returns the same data
as JSON.

---

## Mobile

`engcode mobile` transpiles EnglishCode into a React Native project:

```bash
engcode mobile init
engcode mobile build app.eng
engcode mobile run-ios
engcode mobile run-android
```

This generates runnable React Native code (components, screens, styles). The
final `.ipa` / `.apk` build still requires Node.js, the React Native CLI, and
Xcode / Android Studio — see `MOBILE_FEATURES.md` for details.

---

## Architecture

The project is a Rust workspace of focused crates:

```
crates/
├── engcode-lexer/      Tokenization + smart spell-checking
├── engcode-parser/     Recursive-descent parser -> AST
├── engcode-analyzer/   Semantic analysis + symbol tables
├── engcode-runtime/    Interpreter / VM
├── engcode-stdlib/     Standard library (SQLite, web server, auth, HTML)
├── engcode-mobile/     React Native transpiler
└── engcode-cli/        Command-line interface
```

Supporting folders:

```
examples/     Run example programs
databases/    SQLite database files (auto-created at runtime)
public/       Rendered HTML pages served by the web server
```

---

## Running Tests

```bash
# Run the entire test suite
cargo test

# Run a specific crate's tests
cargo test --package engcode-lexer
cargo test --package engcode-parser
cargo test --package engcode-runtime
```

Current status: **54 tests passing**.

---

## Examples

Try these with `engcode run examples/<file>.eng`:

| Example | Shows |
|---------|-------|
| `crud-test.eng` | Database CRUD with selective output |
| `method-calls.eng` | String/array methods and chaining |
| `string-interpolation.eng` | `{variable}` interpolation |
| `control-flow.eng` | if/else, loops, break/continue |
| `functions.eng` | Functions with parameters and return |
| `all-new-features.eng` | Objects, auth, WHERE clauses, and more |
| `web-server.eng` | Web server with JSON routes |
| `webapp.eng` | Server + database + data routes |
| `healthcare-system.eng` | Larger database-backed application |
| `homepage.eng` | HTML page generation and rendering |

---

## Known Limitations

See [NOT_WORKING.md](NOT_WORKING.md) for the full, candid list. The headline
items:

- External service integrations (Stripe, SendGrid, Twilio, S3, OAuth) are not implemented.
- No JavaScript generation yet (HTML and CSS only).
- No transactions, indexes, foreign keys, or migrations in the database layer.
- No classes, async/await, or package/import system.
- Mobile transpiles to React Native but does not build `.ipa`/`.apk` end-to-end.
- The web server binds to localhost for development and demos.

---

## Contributing

Contributions are welcome. To contribute:

1. Fork the repository.
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Commit your changes: `git commit -am "Add your feature"`
4. Push to the branch: `git push origin feature/your-feature`
5. Open a pull request.

---

## Support

- Report issues at [GitHub Issues](https://github.com/ikaydreams-dev/OpenSoftware/issues)
- Start a discussion at [GitHub Discussions](https://github.com/ikaydreams-dev/OpenSoftware/discussions)
- See the honest, verified status at [HONEST_STATUS.md](HONEST_STATUS.md)

---

EnglishCode is a Rust-powered, beginner-friendly programming language for
building real applications in plain English.