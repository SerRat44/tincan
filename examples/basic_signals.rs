//! Basic signal operations demonstration

use tincan::Signal;

fn main() {
    println!("=== Basic Signals Example ===\n");

    // Create a signal
    println!("1. Creating a signal with initial value 0");
    let count = Signal::new(0);
    println!("   count.get() = {}\n", count.get());

    // Update the signal
    println!("2. Setting value to 42");
    count.set(42);
    println!("   count.get() = {}\n", count.get());

    // Update using a function
    println!("3. Updating with a function (adding 10)");
    count.update(|n| *n += 10);
    println!("   count.get() = {}\n", count.get());

    // Using with() to read without cloning
    println!("4. Reading with a function (checking if even)");
    let is_even = count.with(|n| n % 2 == 0);
    println!("   Is even? {}\n", is_even);

    // Watching for changes
    println!("5. Setting up a watcher");
    let _guard = count.watch(|value| {
        println!("   -> Count changed to: {}", value);
    });

    println!("\n6. Making changes (watcher will trigger)");
    count.set(100);
    count.set(200);
    count.update(|n| *n *= 2);

    println!("\n✓ Example complete!");
}
