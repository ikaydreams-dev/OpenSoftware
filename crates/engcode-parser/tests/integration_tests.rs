use engcode_parser::{Parser, Program, Statement, Expression};
use engcode_lexer::{Lexer, Token};

#[test]
fn test_complete_crud_workflow() {
    let code = r#"
create a database called "TestDB"
insert into users with name "Alice" and age 30
select all from users
    "#;

    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    assert_eq!(program.statements.len(), 3);
}

#[test]
fn test_variable_assignment() {
    let code = "set name to \"Bob\"";
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Statement::Assignment { variable, value } => {
            assert_eq!(variable, "name");
            assert_eq!(*value, Expression::String("Bob".to_string()));
        }
        _ => panic!("Expected assignment"),
    }
}

#[test]
fn test_insert_multiple_fields() {
    let code = r#"insert into products with title "Laptop" and price 999.99 and stock 50"#;
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Statement::Insert { collection, data } => {
            assert_eq!(collection, "products");
            assert_eq!(data.len(), 3);
            assert_eq!(data[0].0, "title");
            assert_eq!(data[1].0, "price");
            assert_eq!(data[2].0, "stock");
        }
        _ => panic!("Expected insert"),
    }
}

#[test]
fn test_select_statement() {
    let code = "select all from users";
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Statement::Select { collection, .. } => {
            assert_eq!(collection, "users");
        }
        _ => panic!("Expected select"),
    }
}

#[test]
fn test_update_statement() {
    let code = r#"update users with status "active""#;
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Statement::Update { collection, data, .. } => {
            assert_eq!(collection, "users");
            assert_eq!(data.len(), 1);
        }
        _ => panic!("Expected update"),
    }
}

#[test]
fn test_delete_statement() {
    let code = "delete from users";
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Statement::Delete { collection, .. } => {
            assert_eq!(collection, "users");
        }
        _ => panic!("Expected delete"),
    }
}

#[test]
fn test_multiple_variable_syntaxes() {
    let tests = vec![
        "set x to 5",
        "let x = 5",
        "make x to 5",
        "store x = 5",
    ];

    for code in tests {
        let mut lexer = Lexer::new(code.to_string());
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        match &program.statements[0] {
            Statement::Assignment { variable, value } => {
                assert_eq!(variable, "x");
                assert_eq!(*value, Expression::Number(5.0));
            }
            _ => panic!("Expected assignment for: {}", code),
        }
    }
}

#[test]
fn test_boolean_values() {
    let code = "set active to true";
    let mut lexer = Lexer::new(code.to_string());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    match &program.statements[0] {
        Statement::Assignment { value, .. } => {
            assert_eq!(*value, Expression::Boolean(true));
        }
        _ => panic!("Expected assignment"),
    }
}
