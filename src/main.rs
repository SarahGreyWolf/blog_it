mod parser;

#[derive(Default, Debug)]
struct Post {
    title: String,
    short_desc: String,
    content: String,
    date: String,
    author: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
