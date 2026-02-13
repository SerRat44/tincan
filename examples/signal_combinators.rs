//! Demonstration of signal combinators (map, zip)

use std::thread;
use std::time::Duration;
use tincan::Signal;

fn main() {
    println!("=== Signal Combinators Example ===\n");

    // Map: transform signal values
    println!("1. Creating a signal and mapping it");
    let temperature_celsius = Signal::new(25);
    let (temperature_fahrenheit, _guard) = temperature_celsius.map(|c| c * 9 / 5 + 32);

    println!(
        "   {}°C = {}°F",
        temperature_celsius.get(),
        temperature_fahrenheit.get()
    );

    println!("\n2. Updating source signal");
    temperature_celsius.set(0);
    thread::sleep(Duration::from_millis(50)); // Allow propagation
    println!(
        "   {}°C = {}°F",
        temperature_celsius.get(),
        temperature_fahrenheit.get()
    );

    temperature_celsius.set(100);
    thread::sleep(Duration::from_millis(50));
    println!(
        "   {}°C = {}°F",
        temperature_celsius.get(),
        temperature_fahrenheit.get()
    );

    // Zip: combine two signals
    println!("\n3. Combining two signals with zip");
    let width = Signal::new(10);
    let height = Signal::new(5);
    let (area, _guard2) = width.clone().zip(height.clone()).map(|(w, h)| w * h);

    println!(
        "   Width: {}, Height: {}, Area: {}",
        width.get(),
        height.get(),
        area.get()
    );

    println!("\n4. Updating one dimension");
    width.set(20);
    thread::sleep(Duration::from_millis(50));
    println!(
        "   Width: {}, Height: {}, Area: {}",
        width.get(),
        height.get(),
        area.get()
    );

    println!("\n5. Updating both dimensions");
    width.set(15);
    height.set(8);
    thread::sleep(Duration::from_millis(50));
    println!(
        "   Width: {}, Height: {}, Area: {}",
        width.get(),
        height.get(),
        area.get()
    );

    // Chain multiple transformations
    println!("\n6. Chaining transformations");
    let base = Signal::new(2);
    let (doubled, _guard3) = base.map(|n| n * 2);
    let (squared, _guard4) = doubled.map(|n| n * n);

    println!(
        "   Base: {}, Doubled: {}, Squared: {}",
        base.get(),
        doubled.get(),
        squared.get()
    );

    base.set(3);
    thread::sleep(Duration::from_millis(50));
    println!(
        "   Base: {}, Doubled: {}, Squared: {}",
        base.get(),
        doubled.get(),
        squared.get()
    );

    println!("\n✓ Example complete!");
}
