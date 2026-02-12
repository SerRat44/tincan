//! Basic signal example

use tincan::{create_effect, create_signal};

fn main() {
    println!("=== Basic Signal Example ===\n");

    // Create a signal with initial value
    let (count, set_count) = create_signal(0);

    // Create an effect that runs when count changes
    create_effect({
        let count = count.clone();
        move || {
            println!("Count changed to: {}", count.get());
        }
    });

    // Update the signal
    println!("Setting count to 5...");
    set_count.set(5);

    println!("Setting count to 10...");
    set_count.set(10);

    println!("Updating count by adding 3...");
    set_count.update(|n| *n += 3);
}
