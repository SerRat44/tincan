//! Demonstration of memoized computations

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tincan::{Memo, Signal};

fn main() {
    println!("=== Memos Example ===\n");

    // Memos cache expensive computations
    println!("1. Creating a memo that tracks computation count");
    let count = Signal::new(5);
    let compute_counter = Arc::new(AtomicUsize::new(0));

    let expensive_double = Memo::new({
        let count = count.clone();
        let counter = Arc::clone(&compute_counter);
        move || {
            counter.fetch_add(1, Ordering::SeqCst);
            println!("   [Computing] Doubling {}...", count.get());
            count.get() * 2
        }
    });

    println!("\n2. First access - will compute");
    println!("   Result: {}", expensive_double.get());
    println!(
        "   Computation count: {}",
        compute_counter.load(Ordering::SeqCst)
    );

    println!("\n3. Second access - uses cached value");
    println!("   Result: {}", expensive_double.get());
    println!(
        "   Computation count: {}",
        compute_counter.load(Ordering::SeqCst)
    );

    println!("\n4. Third access - still cached");
    println!("   Result: {}", expensive_double.get());
    println!(
        "   Computation count: {}",
        compute_counter.load(Ordering::SeqCst)
    );

    println!("\n5. Updating source signal");
    count.set(10);

    println!("\n6. Accessing after change - recomputes");
    println!("   Result: {}", expensive_double.get());
    println!(
        "   Computation count: {}",
        compute_counter.load(Ordering::SeqCst)
    );

    println!("\n7. Another access - cached again");
    println!("   Result: {}", expensive_double.get());
    println!(
        "   Computation count: {}",
        compute_counter.load(Ordering::SeqCst)
    );

    // Multiple dependencies
    println!("\n8. Memo with multiple dependencies");
    let a = Signal::new(3);
    let b = Signal::new(4);
    let c = Signal::new(5);

    let sum = Memo::new({
        let a = a.clone();
        let b = b.clone();
        let c = c.clone();
        move || {
            println!("   [Computing] Sum...");
            a.get() + b.get() + c.get()
        }
    });

    println!("   Sum: {}", sum.get());

    println!("\n9. Changing one dependency");
    a.set(10);
    println!("   Sum: {}", sum.get());

    println!("\n10. Reading again (cached)");
    println!("   Sum: {}", sum.get());

    println!("\n✓ Example complete!");
    println!(
        "   Total computations of expensive_double: {}",
        compute_counter.load(Ordering::SeqCst)
    );
}
