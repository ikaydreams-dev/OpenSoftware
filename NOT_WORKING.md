# EnglishCode - Complete List of What's NOT Working

**Last Updated:** September 11, 2026
**Reality Check:** Everything that doesn't work or is incomplete.

> **Note:** This file used to claim that method calls, comments, else-if, array
> indexing, negative numbers, string interpolation, WHERE clauses, and auth were
> all broken. **Those are all verified working now.** This is the accurate list.

---

## RECENTLY FIXED / VERIFIED WORKING

The following were previously listed as broken but are now tested end-to-end:

1. **Method calls** — `text.uppercase()`, `"a,b".split(",")`, chaining works
2. **Comments** — `# this is a comment` works (full-line and inline)
3. **Else-if** — `else if` chains work
4. **Array indexing** — `items[0]`, `items[2]` works
5. **Negative numbers** — `set n to -5` works
6. **String interpolation** — `show "Hello {name}"` works
7. **Object literals & property access** — `{name: "Alice"}` and `person.name` work
8. **WHERE clauses** — `select/update/delete ... where age greaterthan 18` works
9. **Auth in .eng** — `signup`, `login`, `logout` persist to the database
10. **Server-side templating** — `{{collection}}` markers in pages render live tables
11. **Lifecycle control** — `start the server for 5 seconds` exits cleanly; Ctrl+C is graceful
12. **Idempotent DB runs** — `create collections in it` resets existing collections, so re-running a script no longer duplicates rows
13. **Data-backed routes** — `add route get "/api/users" returning rows from users` returns real DB rows as JSON
14. **Form submissions** — `add form` + `add input with name` renders `<form onsubmit="engcSubmit(this)">`; named inputs are JSON-posted to the form's action; `handle post` receives the parsed JSON body
15. **HTTP Cookies** — `set cookie "name" to "value"` / `set cookie "name" expires in 7 days` emit `Set-Cookie` headers; subsequent requests see the cookie via the `cookies` variable
16. **CSS styling** — `set style "selector" with property "value"` outputs CSS blocks in generated pages (hyphenated properties like `font-size` supported)
17. **CORS & Logging middleware** — `add middleware cors` adds `Access-Control-Allow-*` headers to all responses (including OPTIONS preflight); `add middleware logging` logs `[engcode] METHOD /path -> STATUS (Nms)`; middleware is applied at server start so routes added after are covered
18. **Mobile navigation** — `navigate to page "profile"` and `go back` transpile to React Navigation's `navigation.navigate`/`navigation.goBack`; E2E build with React Navigation dependencies verified
19. **Mobile data fetching** — `fetch data from "url"` transpiles to React `fetch` + `.then(data => setState(data))`; verified in E2E mobile build
20. **Word arithmetic** — `2 plus 3`, `10 minus 4`, `2 times 3`, `12 divided by 4` (and symbol forms `+ - * /`) all parse and evaluate
21. **Test/assert framework** — `test "name" ... assert <cond> with message "..." ... end` runs, counts assertions, reports per-test results, and exits non-zero on failure (Category 5 now works)
22. **File uploads** — `add upload route "/upload" to "dir"` accepts multipart POSTs, saves files, returns JSON; files are served from `/uploads/...`
23. **Database joins** — `select all from A join B on A.key is B.key` does an inner join, merging matching rows (prefixed and bare field paths supported)
24. **`is` as equality** — `where name is "bob"`, `if n is 5 then`, and join conditions now work (previously `Unexpected token: Is`)
25. **WebSockets** — `add websocket route "/ws"` upgrades to a live WebSocket that echoes messages back

---

## CATEGORY 1: Database Gaps (still missing)

### Transactions
```englishcode

begin transaction
 insert into users
 insert into logs
commit transaction # Not implemented
```

**Status:** Each write commits immediately; no transaction support.

### Indexes
```englishcode

create index on users for email # Not implemented
```

**Status:** No index creation — full table scans on `where`.

### Foreign Keys / Relations
```englishcode

set user_id as foreign key # Not implemented
```

**Status:** No relationships or referential integrity.

### Migrations
```bash

engcode migrate up # Doesn't exist
```

**Status:** No migration system.

### Database Joins
```englishcode

select all from users join posts on user_id # Not implemented
```

**Status:** No join support — do lookups manually in EnglishCode.

### Database Backups
```englishcode

backup database to "backup.db" # Not implemented
```

**Status:** No backup tools (files are plain SQLite in `databases/`).

---

## CATEGORY 2: External Services (0% Done)

### Stripe Payments - NOT IMPLEMENTED
```englishcode

charge customer "tok_visa" amount 1999 # Not implemented
create subscription for "alice" # Not implemented
```

### SendGrid Email - NOT IMPLEMENTED
```englishcode

send email to "alice@test.com" with subject "Hi" and body "Hello" # Not implemented
```

### Twilio SMS - NOT IMPLEMENTED
```englishcode

send sms to "+1234567890" with message "Code: 1234" # Not implemented
```

### AWS S3 - NOT IMPLEMENTED
```englishcode

upload file "photo.jpg" to s3 bucket "mybucket" # Not implemented
```

### OAuth - NOT IMPLEMENTED
```englishcode

login with google # Not implemented
login with github # Not implemented
```

### JWT Tokens - PARTIAL
**Status:** Session tokens work in-memory; no JWT issuance/verification.

---

## CATEGORY 3: Mobile Incomplete

- ~~Navigation (`navigate to page "profile"`, `go back`)~~ ✅ works
- State management (no Context/Redux) — basic assignment/state exists; no full Context
- ~~API calls from mobile (`fetch data from ...`)~~ ✅ works (`.then(data => setState(data))`)
- Native modules (camera, GPS, push notifications)
- Biometric auth (FaceID/TouchID)
- Deep linking
- App icons/splash manual in Xcode/Android Studio
- Build pipeline manual: `engcode mobile build` generates React Native code, then you run `npm install` / `pod install` / `xcodebuild` yourself

---

## CATEGORY 4: Web Development Gaps

### Form Submissions - ✅ DONE
Forms now generate client-side JSON submission via `engcSubmit()`:

```englishcode
add form with action "/api/contact" and method "post"
add input with type "email" and name "email"
add input with type "text" and name "subject"
add button labeled "Send"
```

`<form>` renders with `onsubmit="event.preventDefault(); engcSubmit(this); return false;"`.
`engcSubmit` collects all `[name]` inputs and POSTs JSON to the form's `action`.
A `handle post "/api/contact" with body as message` block receives it as a JSON object.

### Form Validation - NOT IMPLEMENTED (client-side)
```englishcode
validate email is valid # Not implemented
```

Server-side input validation via `validate request with` handler body IS implemented.

### JavaScript Generation - NOT IMPLEMENTED
```englishcode
add onclick handler "alert('hi')" # Generates HTML but no JS
```

**Status:** HTML + CSS + minimal JS (engcSubmit) only; no general client-side JS.

### WebSockets - ✅ DONE
```englishcode
add websocket route "/ws"
```

GET upgrades to a WebSocket; every text message is echoed back as `echo: <message>`.
Verified end-to-end with a real client (sends "hello from node", receives `echo: hello from node`).


### GraphQL - NOT IMPLEMENTED
**Status:** REST/JSON routes only.

### File Uploads - ✅ DONE
```englishcode
add upload route "/upload" to "public/uploads"
```

Multipart `POST` to the route saves each part to the target directory and returns
`{"uploaded":[{"filename":"note.txt","size":13,"path":"public/uploads/note.txt"}],"status":201}`.
Uploaded files are served back from `/uploads/...`.

### HTTP Cookies - ✅ DONE (basic)
```englishcode
set cookie "session" to "abc123"  # works — sets Set-Cookie response header
```

**Status:** `set cookie` / `set cookie name expires in N days` both work. Cookie values are sent back on subsequent requests and available via the `cookies` variable.
Limitation: no automatic session persistence across requests (each fresh handler context; cross-request session state isn't automatic).

### Middleware / CORS - ✅ DONE
```englishcode
add middleware cors       # adds Access-Control-Allow-* headers to all responses
add middleware logging    # logs [engcode] METHOD /path -> STATUS (Nms) to stdout
```

`add middleware` should appear BEFORE routes. Middleware is applied at server start,
so routes added after `add middleware` are correctly covered.

### Rate Limiting - NOT IMPLEMENTED
**Status:** No rate limiting.

---

## CATEGORY 5: Testing - ✅ VERIFIED WORKING

```englishcode
test "addition works"
 assert 2 plus 2 equalto 4
end
```

**Status:** `test`/`assert`/`expect` work with optional `with message "..."`;
assertion counts are per-test and the process exits non-zero when a test fails.
Rust unit tests (59 passing) also cover the engine.

---

## CATEGORY 6: Package System (0% Done)

- Import/export between `.eng` files (single-file only)
- Package manager (`engpkg install ...`)
- Namespaces (global scope only)

---

## CATEGORY 7: Advanced Language Features

- Classes / OOP (`define class User`), inheritance, interfaces, generics
- Async/await, promises, decorators, pattern matching, enums, type annotations
- Error capture: try/catch works, but `catch ... as e` can't read an error variable, and stack traces are line-number only

---

## CATEGORY 8: IDE Features

- File tree / explorer
- Multi-file support (one file at a time)
- Debugger (breakpoints / step-through)
- Git integration
- Auto-complete / IntelliSense
- Find/replace
- Code formatter
- Plugin system

---

## CATEGORY 9: Deployment (0% Done)

- `engcode deploy to vercel` / `netlify` / docker / k8s
- Serverless functions
- `get env variable "API_KEY"` (env vars not exposed to .eng)
- CI/CD templates

---

## CATEGORY 10: Performance

- Bytecode/JIT compiler (interpreter only)
- Memory/optimization passes
- Parallel execution (single-threaded)
- Profiler (`engcode profile ...`)

---

## CATEGORY 11: Documentation

- Doc comments (`doc "..."`) / API reference generation
- Interactive tutorial / example gallery UI / video tutorials / community forum

---

## MINOR GOTCHAS (working, but be aware)

1. **Grammar is strict:**
 - `create collections in it` (newline + names) works; `create collections
     called "users" in it` does **not** parse. Use the newline form.

 - `create a web server on port 3000` is the server form; `create a server`
     alone isn't a valid statement.

 - `signup "alice" with email "..." and password "..."` (one-word `signup`,
     not `sign up`).

 - Unquoted identifiers in arrays are treated as variables: use `["red"]`,
     not `[red]`.

2. **`create collections in it` resets the collection each run** — intentional,
 so scripts are idempotent. If you want to keep data, don't re-declare the
 collection.
3. **Server binding:** `start the server` blocks until Ctrl+C.
 `start the server for N seconds` auto-stops after N seconds and the program
 exits cleanly.
4. **Static HTML pages**: rendered once to `public/*.html` by `render page "x"`,
 then server-side templating (`{{collection}}`) is applied per-request.
5. **`age: 30` in JSON** renders as `30` (never `30.0`).
6. Interpreter HTML statements `add element`, `add form`, and `set style` parse
 but are currently no-ops.
7. **Web server is single-process, localhost-bound** (Axum + Tokio): great for
 dev/demos; real hosting needs a deployment layer (not built yet).

---

## SUMMARY

Everything shipped so far is **verified working**:

| Category | Status |
|----------|--------|
| Core language (vars, control flow, functions, try/catch) | Working |
| Method calls, chaining, comments, interpolation | Working |
| Objects, arrays, indexing, negative numbers, else-if | Working |
| SQLite CRUD + WHERE filters | Working |
| Auth (signup/login/logout) persisted in .eng | Working |
| Web server, JSON routes, data routes, templated pages | Working |
| Idempotent DB runs / clean server exit | Working |
| Rust test suite | 54/54 passing |
| Stripe, SendGrid, OAuth, S3, WebSockets, GraphQL | Not started |
| Packages, classes/OOP, async, testing framework | Not started |
| Mobile end-to-end build, deployment, IDE extras | Partial / not started |

**Remaining big-ticket gaps:** external services (payments/email/SMS), mobile
end-to-end builds, JavaScript generation, WebSockets, testing framework, and a
deployment story.

**Current pace:** core is solid; the quickest wins next are (1) wiring the
parsed-but-no-op HTML statements, (2) form submissions, and (3) JS generation.
