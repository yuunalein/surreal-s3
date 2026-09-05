use surrealism::surrealism;

#[surrealism(default)]
fn hello() -> String {
    "Hello from Surrealism!".to_string()
}

#[surrealism]
fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[surrealism]
fn add(a: i64, b: i64) -> i64 {
    a + b
}
