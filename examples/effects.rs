//! Demonstration of reactive effects

use std::thread;
use std::time::Duration;
use tincan::{Effect, Signal};

fn main() {
    println!("=== Effects Example ===\n");

    // Effects run automatically when dependencies change
    println!("1. Creating a signal and an effect");
    let count = Signal::new(0);

    let _effect = Effect::new({
        let count = count.clone();
        move || {
            println!("   [Effect] Count is now: {}", count.get());
        }
    });

    println!("\n2. Effect runs immediately on creation (printed above)");

    println!("\n3. Updating the signal triggers the effect");
    count.set(5);
    thread::sleep(Duration::from_millis(50));

    count.set(10);
    thread::sleep(Duration::from_millis(50));

    count.update(|n| *n += 5);
    thread::sleep(Duration::from_millis(50));

    // Multiple dependencies
    println!("\n4. Effect with multiple dependencies");
    let first_name = Signal::new("John".to_string());
    let last_name = Signal::new("Doe".to_string());

    let _name_effect = Effect::new({
        let first = first_name.clone();
        let last = last_name.clone();
        move || {
            println!("   [Effect] Full name: {} {}", first.get(), last.get());
        }
    });

    println!("\n5. Changing first name");
    first_name.set("Jane".to_string());
    thread::sleep(Duration::from_millis(50));

    println!("\n6. Changing last name");
    last_name.set("Smith".to_string());
    thread::sleep(Duration::from_millis(50));

    // Effect cleanup
    println!("\n7. Effects cleanup when dropped");
    {
        let temp = Signal::new(20);
        let _temp_effect = Effect::new({
            let temp = temp.clone();
            move || {
                println!("   [Effect] Temperature: {}°C", temp.get());
            }
        });

        temp.set(25);
        thread::sleep(Duration::from_millis(50));
        println!("   Scope ending, effect will be cleaned up...");
    }
    println!("   Effect dropped and cleaned up!");

    // Conditional effects
    println!("\n8. Effect with conditional logic");
    let value = Signal::new(5);

    let _conditional_effect = Effect::new({
        let value = value.clone();
        move || {
            let val = value.get();
            if val > 10 {
                println!("   [Effect] Value {} is HIGH", val);
            } else {
                println!("   [Effect] Value {} is LOW", val);
            }
        }
    });

    println!("\n9. Testing conditional effect");
    value.set(3);
    thread::sleep(Duration::from_millis(50));

    value.set(15);
    thread::sleep(Duration::from_millis(50));

    value.set(7);
    thread::sleep(Duration::from_millis(50));

    println!("\n✓ Example complete!");
}
