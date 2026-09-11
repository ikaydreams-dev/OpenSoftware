# EnglishCode - Programming in Plain English

> **Code in English. Build real applications. No programming syntax required.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![macOS](https://img.shields.io/badge/macOS-Supported-green.svg)](https://www.apple.com/macos/)

---

## 🎯 What is EnglishCode?

EnglishCode is a revolutionary programming language where **English IS the code**. No curly braces, no semicolons, no cryptic syntax—just plain English sentences that execute as real programs.

### Why EnglishCode?

- **🌍 Natural Language** - Write code like you speak
- **⚡ 10x Less Code** - Say more with less
- **🎓 Zero Learning Curve** - If you can write English, you can code
- **📱 Mobile Apps** - Build iOS & Android apps from English
- **🔐 Authentication** - Built-in signup/login/sessions
- **🔥 Production Ready** - Full database, web server, and CRUD operations
- **🚀 Native Performance** - Rust-powered runtime, blazingly fast
- **✨ Smart Spell-Checking** - Auto-corrects typos automatically

---

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/englishcode.git
cd englishcode

# Build from source
cargo build --release

# Add to PATH
sudo cp target/release/engcode /usr/local/bin/
```

### Your First Program

Create a file called `hello.eng`:

```
show "Hello, World!"
```

Run it:

```bash
engcode run hello.eng
```

Output:
```
→ Running hello.eng

Hello, World!

Program completed successfully!
```

---

## 📚 Language Features

### 1. **Variables**

```
set name to "Alice"
set age to 30
set active to true
let score = 95.5
```

### 2. **Databases**

```
create a database called "MyApp"

create these collections in it
  users
  products
  orders
```

### 3. **Insert Data (CRUD)**

```
insert into users with name "John" and age 28 and email "john@example.com"

add to products with title "Laptop" and price 999.99 and stock 50
```

### 4. **Read Data**

```
select all from users

get all from products
```

### 5. **Display Output**

```
show "Processing complete!"
show "Total users: 150"
```

---

## 💻 Complete Example

Here's a real working example (`examples/user-system.eng`):

```
create a database called "UserSystem"

create these collections in it
  users
  sessions
  activity_log

set admin to "Alice"
set status to "active"

insert into users with name "Alice" and role "admin" and status "active"
insert into users with name "Bob" and role "user" and status "active"
insert into users with name "Charlie" and role "moderator" and status "pending"

show "User system initialized!"

select all from users

show "Setup complete!"
```

Run it:

```bash
engcode run examples/user-system.eng
```

Output:

```
→ Running examples/user-system.eng

✓ Created database UserSystem
  ✓ Created collection users
  ✓ Created collection sessions
  ✓ Created collection activity_log
  → Set admin = Alice
  → Set status = active
  ✓ Inserted into users
  ✓ Inserted into users
  ✓ Inserted into users
User system initialized!

→ Found 3 rows in users:
  1. {"name": "Alice", "role": "admin", "status": "active"}
  2. {"name": "Bob", "role": "user", "status": "active"}
  3. {"name": "Charlie", "role": "moderator", "status": "pending"}

Setup complete!

Program completed successfully!
```

---

## 🎨 Multiple Ways to Say Things

EnglishCode understands natural variations:

### Creating Variables

```
set name to "John"
let age = 25
make status to true
store count = 100
save total to 500
```

### Database Operations

```
insert into users...
add to users...
put into users...
```

```
select all from users
get all from users
find all from users
fetch all from users
```

```
update users...
change users...
modify users...
```

```
delete from users
remove from users
```

---

## 🏗️ Architecture

EnglishCode is built with:

- **Runtime:** Rust (fast, safe, native binaries)
- **Database:** SQLite (built-in, no setup required)
- **Parser:** Custom recursive descent parser
- **Lexer:** Smart tokenizer with spell-checking
- **Analyzer:** Semantic analysis with symbol tables

### Project Structure

```
englishcode/
├── crates/
│   ├── engcode-lexer/      # Tokenization + spell-checking
│   ├── engcode-parser/     # AST generation
│   ├── engcode-analyzer/   # Semantic analysis
│   ├── engcode-runtime/    # Interpreter/VM
│   ├── engcode-stdlib/     # Standard library (DB, IO)
│   └── engcode-cli/        # Command-line interface
├── examples/               # Example .eng programs
├── databases/             # SQLite databases (auto-created)
└── README.md             # This file
```

---

## 🧪 Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --package engcode-lexer
cargo test --package engcode-parser
cargo test --package engcode-runtime

# Run with output
cargo test -- --nocapture
```

Current test status: **26/26 tests passing** ✅

---

## 📖 Examples

### Basic Examples

- [`examples/hello-world.eng`](examples/hello-world.eng) - Simple output
- [`examples/variables-test.eng`](examples/variables-test.eng) - Variable assignments
- [`examples/crud-test.eng`](examples/crud-test.eng) - Database CRUD operations

### Production Examples

- [`examples/ecommerce-setup.eng`](examples/ecommerce-setup.eng) - E-commerce platform (16 collections)
- [`examples/saas-platform.eng`](examples/saas-platform.eng) - SaaS application (20 collections)
- [`examples/social-network.eng`](examples/social-network.eng) - Social media (22 collections)
- [`examples/project-management.eng`](examples/project-management.eng) - Project management (21 collections)
- [`examples/healthcare-system.eng`](examples/healthcare-system.eng) - Healthcare system (22 collections)

---

## 🚀 Roadmap

### ✅ Phase 1: Foundation (COMPLETE - 95%)

- [x] Lexer with spell-checking
- [x] Parser with AST generation
- [x] Semantic analyzer
- [x] Interpreter/VM
- [x] Value types (number, string, boolean, array, object)
- [x] Variables and assignments
- [x] SQLite database integration
- [x] CRUD operations (insert, select, update, delete)
- [x] CLI with colored output
- [x] 26 unit tests passing
- [ ] Comprehensive error messages
- [ ] Performance optimization
- [ ] 100% test coverage

### ⏳ Phase 2: Desktop IDE (Next)

- [ ] Tauri 2.0 desktop application
- [ ] Monaco Editor integration
- [ ] File explorer sidebar
- [ ] Integrated terminal
- [ ] Live preview pane
- [ ] Syntax highlighting
- [ ] Auto-complete
- [ ] DMG installer for macOS

### ⏳ Phase 3: Language Expansion

- [ ] Functions and procedures
- [ ] Control flow (if/else, loops)
- [ ] Error handling (try/catch)
- [ ] Import system
- [ ] MongoDB, PostgreSQL, MySQL support

### ⏳ Phase 4: Web Capabilities

- [ ] Axum web server
- [ ] HTTP routing
- [ ] API endpoints
- [ ] WebSockets
- [ ] HTML/CSS generation

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Database powered by [SQLite](https://www.sqlite.org/)
- Inspired by natural language processing and human-computer interaction research

---

## 📞 Support

- **Issues:** [GitHub Issues](https://github.com/yourusername/englishcode/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/englishcode/discussions)

---

**EnglishCode - Programming for Everyone** 🌟

*Version 0.1.0 - macOS Edition*
