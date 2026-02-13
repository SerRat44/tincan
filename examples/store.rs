//! Demonstration of Store for managing complex state

use tincan::Store;

#[derive(Clone, Debug)]
struct TodoItem {
    id: usize,
    title: String,
    completed: bool,
}

#[derive(Clone, Debug)]
struct AppState {
    todos: Vec<TodoItem>,
    filter: TodoFilter,
}

#[derive(Clone, Debug, PartialEq)]
enum TodoFilter {
    All,
    Active,
    Completed,
}

impl AppState {
    fn new() -> Self {
        Self {
            todos: Vec::new(),
            filter: TodoFilter::All,
        }
    }

    fn add_todo(&mut self, title: String) {
        let id = self.todos.len();
        self.todos.push(TodoItem {
            id,
            title,
            completed: false,
        });
    }

    fn toggle_todo(&mut self, id: usize) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
        }
    }

    fn filtered_todos(&self) -> Vec<&TodoItem> {
        match self.filter {
            TodoFilter::All => self.todos.iter().collect(),
            TodoFilter::Active => self.todos.iter().filter(|t| !t.completed).collect(),
            TodoFilter::Completed => self.todos.iter().filter(|t| t.completed).collect(),
        }
    }

    fn stats(&self) -> (usize, usize, usize) {
        let total = self.todos.len();
        let completed = self.todos.iter().filter(|t| t.completed).count();
        let active = total - completed;
        (total, active, completed)
    }
}

fn main() {
    println!("=== Store Example: Todo App ===\n");

    // Create store with initial state
    let store = Store::new(AppState::new());

    // Subscribe to state changes
    println!("1. Setting up subscriber");
    store.subscribe(|state| {
        let (total, active, completed) = state.stats();
        println!(
            "   [Store Update] Total: {}, Active: {}, Completed: {}",
            total, active, completed
        );
    });

    // Add todos
    println!("\n2. Adding todos");
    store.update(|state| {
        state.add_todo("Learn Rust".to_string());
    });

    store.update(|state| {
        state.add_todo("Build reactive library".to_string());
    });

    store.update(|state| {
        state.add_todo("Write documentation".to_string());
    });

    // Display current todos
    println!("\n3. Current todos:");
    store.read(|state| {
        for todo in &state.todos {
            let status = if todo.completed { "✓" } else { " " };
            println!("   [{}] {}", status, todo.title);
        }
    });

    // Complete a todo
    println!("\n4. Completing first todo");
    store.update(|state| {
        state.toggle_todo(0);
    });

    // Display updated todos
    println!("\n5. Current todos:");
    store.read(|state| {
        for todo in &state.todos {
            let status = if todo.completed { "✓" } else { " " };
            println!("   [{}] {}", status, todo.title);
        }
    });

    // Complete another
    println!("\n6. Completing second todo");
    store.update(|state| {
        state.toggle_todo(1);
    });

    // Change filter
    println!("\n7. Filtering to show only active todos");
    store.update(|state| {
        state.filter = TodoFilter::Active;
    });

    println!("\n8. Active todos:");
    store.read(|state| {
        for todo in state.filtered_todos() {
            println!("   [ ] {}", todo.title);
        }
    });

    // Show completed
    println!("\n9. Filtering to show completed todos");
    store.update(|state| {
        state.filter = TodoFilter::Completed;
    });

    println!("\n10. Completed todos:");
    store.read(|state| {
        for todo in state.filtered_todos() {
            println!("   [✓] {}", todo.title);
        }
    });

    // Final stats
    println!("\n11. Final statistics:");
    let (total, active, completed) = store.read(|state| state.stats());
    println!("   Total: {}", total);
    println!("   Active: {}", active);
    println!("   Completed: {}", completed);

    println!("\n✓ Example complete!");
}
