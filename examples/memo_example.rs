//! Derived values with memos

use tincan::{create_memo, create_signal};

fn main() {
    println!("=== Memo Example ===\n");

    let (first_name, set_first_name) = create_signal("John".to_string());
    let (last_name, set_last_name) = create_signal("Doe".to_string());

    // Create a derived value that combines first and last name
    let full_name = create_memo({
        let first_name = first_name.clone();
        let last_name = last_name.clone();
        move || {
            let full = format!("{} {}", first_name.get(), last_name.get());
            println!("  (Computing full name...)");
            full
        }
    });

    println!("Initial full name: {}", full_name.get());

    println!("\nUpdating first name...");
    set_first_name.set("Jane".to_string());
    println!("New full name: {}", full_name.get());

    println!("\nUpdating last name...");
    set_last_name.set("Smith".to_string());
    println!("New full name: {}", full_name.get());
}
