use std::error::Error;
use std::io::BufRead;

/// Setup rayon (initialize threadpools according to concurrency).
pub fn setup_rayon(concurrency: usize) -> Result<(), Box<dyn Error>> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .build_global()?;
    Ok(())
}

/// If items contain a single string "-", read items from stdin, otherwise return as-is.
pub fn items_from_opt(items: Vec<String>) -> Result<Vec<String>, std::io::Error> {
    Ok(if items.len() == 1 && items[0] == "-" {
        read_items_from_stdin()?
    } else {
        items
    })
}

/// Read items from stdin, one item per line.
pub fn read_items_from_stdin() -> Result<Vec<String>, std::io::Error> {
    let mut items: Vec<String> = vec![];
    for line in std::io::stdin().lock().lines() {
        items.push(line?);
    }
    Ok(items)
}
