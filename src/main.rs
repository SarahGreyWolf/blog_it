use std::{
    cmp::Ordering,
    fmt::Display,
    fs::OpenOptions,
    io::{self, Read, Write},
};

mod parser;

#[derive(Default, Debug, PartialEq, Eq, Ord)]
struct Date {
    day: u32,
    month: u32,
    year: u32,
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.year == other.year && self.month == other.month && self.day == other.day {
            return Some(core::cmp::Ordering::Equal);
        }
        if self.year > other.year {
            return Some(core::cmp::Ordering::Greater);
        }
        if self.month > other.month && self.year == other.year {
            return Some(core::cmp::Ordering::Greater);
        }
        if self.day > other.day && self.month == other.month && self.year == other.year {
            return Some(core::cmp::Ordering::Greater);
        }
        Some(core::cmp::Ordering::Less)
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        output.push_str(&format!("{:2}/", self.day));
        output.push_str(&format!("{:2}/", self.month));
        output.push_str(&format!("{:4}", self.year));
        write!(f, "{}", output)
    }
}

#[derive(Default, Debug, Eq)]
struct Post {
    title: String,
    short_desc: String,
    content: String,
    date: Date,
    author: String,
    is_draft: bool,
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool { self.title == other.title && self.date == other.date }
}

impl PartialOrd for Post {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.date.partial_cmp(&other.date) }
}

impl Ord for Post {
    fn cmp(&self, other: &Self) -> Ordering { self.date.cmp(&other.date) }
}

impl Post {
    fn produced_html(&self) -> String {
        let mut main = format!(
            "
            <header>
                <h1>{}</h1>
                <h3>{}</h3>
                <h2>{}</h2>
            </header>",
            self.title, self.date, self.short_desc
        );

        let mut block = String::from("<p>");
        for paragraph in self.content.split('\n') {
            // TODO: Handle markdown links
            if paragraph.is_empty() {
                block = block.trim_end_matches("<br>").to_owned();
                block.push_str("</p>");
                main.push_str(&block);
                block = String::from("<p>");
                continue;
            }
            block.push_str(&format!("{paragraph}"));
            block.push_str("<br>");
        }
        block = block.trim_end_matches("<br>").to_owned();
        block.push_str("</p>");
        main.push_str(&block);

        main
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = OpenOptions::new();
    options.create(true).write(true).read(true);
    let post_template = match std::fs::read_to_string("./templates/post.html") {
        Ok(pt) => pt,
        Err(e) => panic!("Couldn't get post template: {e}"),
    };

    let mut posts = vec![];
    let files = match std::fs::read_dir("./posts") {
        Ok(pt) => pt,
        Err(e) => panic!("Couldn't get posts: {e}"),
    };
    for entry in files {
        let file = entry?;
        if let Some(ext) = file.path().extension() {
            if ext == "md" {
                let mut open_file = match options.open(file.path()) {
                    Ok(f) => f,
                    Err(e) => panic!("Couldn't open file for post {:?}: {e}", file.path()),
                };
                let mut post_string = String::new();
                match open_file.read_to_string(&mut post_string) {
                    Ok(_) => {}
                    Err(e) => panic!("Couldn't read from post {:?}: {e}", file.path()),
                };
                posts.push(Post::from(post_string));
            }
        }
    }

    posts.sort();
    posts.reverse();

    for post in &posts {
        if post.is_draft {
            continue;
        }
        let output = post_template.replace("{{% POST %}}", &post.produced_html());
        let output = output.replace("{{% POST_TITLE %}}", &post.title);
        let file_name = post.title.replace(' ', "_");
        let mut open_file = match options.open(format!("./site/posts/{}.html", file_name)) {
            Ok(f) => f,
            Err(e) => panic!(
                "Could not open or create file {}: {e}",
                format!("./site/posts/{}.html", file_name)
            ),
        };
        match open_file.write(output.as_bytes()) {
            Ok(_) => {}
            Err(e) => panic!(
                "Couldn't write to {}: {e}",
                format!("./site/posts/{}.html", file_name)
            ),
        };
    }

    let range = if posts.len() > 4 {
        0..5
    } else {
        0..posts.len()
    };

    let latest_posts = &posts[range];

    generate_home(&options, latest_posts)?;
    generate_posts_list(&options, &posts)?;

    Ok(())
}

fn generate_home(options: &OpenOptions, posts: &[Post]) -> io::Result<()> {
    let mut home_file = match options.open("./templates/home.html") {
        Ok(f) => f,
        Err(e) => panic!("Could not open file ./templates/home.html: {e}"),
    };
    let mut home_template = String::new();
    match home_file.read_to_string(&mut home_template) {
        Ok(_) => {}
        Err(e) => panic!("Couldn't get home template: {e}"),
    };
    let mut output = String::new();
    for post in posts {
        let file_name = post.title.replace(' ', "_");
        output.push_str(&format!(
            "<li><a href=\"/posts/{}.html\">{}</a></li>",
            file_name, post.title
        ));
    }
    let output = home_template.replace("{{% LATEST %}}", &output);
    let mut file = match options.open("./site/index.html") {
        Ok(f) => f,
        Err(e) => panic!("Could not open ./site/index.html: {e}"),
    };
    file.write_all(output.as_bytes())?;
    Ok(())
}

fn generate_posts_list(options: &OpenOptions, posts: &[Post]) -> io::Result<()> {
    let mut posts_file = match options.open("./templates/posts.html") {
        Ok(f) => f,
        Err(e) => panic!("Could not open file ./templates/posts.html: {e}"),
    };
    let mut posts_template = String::new();
    match posts_file.read_to_string(&mut posts_template) {
        Ok(_) => {}
        Err(e) => panic!("Couldn't get posts template: {e}"),
    };
    let mut output = String::from("<div class=\"posts\">");

    for post in posts {
        if post.is_draft {
            continue;
        }
        let file_name = post.title.replace(' ', "_");
        output.push_str(&format!(
            "<a href=\"/posts/{}.html\">{}</a>",
            file_name, post.title
        ));
    }
    output.push_str("</div>");
    let output = posts_template.replace("{{% POSTS %}}", &output);
    let mut file = match options.open("./site/posts/index.html") {
        Ok(f) => f,
        Err(e) => panic!("Could not open ./site/posts/index.html: {e}"),
    };
    match file.write(output.as_bytes()) {
        Ok(_) => {}
        Err(e) => panic!("Couldn't write to ./site/posts/index.html: {e}"),
    };

    Ok(())
}
