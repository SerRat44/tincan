//! Store example with complex state

use tincan::Store;

#[derive(Clone, Debug)]
struct TodoItem {
    id: usize,
    text: String,
    completed: bool,
}

#[derive(Clone, Debug)]
struct AppState {
    todos: Vec<TodoItem>,
    filter: String,
}

fn main() {
    println!("=== Store Example ===\n");

    // Create a store with initial state
    let store = Store::new(AppState {
        todos: vec![],
        filter: "all".to_string(),
    });

    // Subscribe to state changes
    store.subscribe(|state| {
        println!(
            "State updated! Active todos: {}",
            state.todos.iter().filter(|t| !t.completed).count()
        );
    });

    // Add a todo
    println!("Adding todo...");
    store.update(|state| {
        state.todos.push(TodoItem {
            id: 1,
            text: "Learn Chronicle".to_string(),
            completed: false,
        });
    });

    // Complete the todo
    println!("\nCompleting todo...");
    store.update(|state| {
        if let Some(todo) = state.todos.first_mut() {
            todo.completed = true;
        }
    });

    // Read final state
    println!("\nFinal state: {:#?}", store.get());
}
