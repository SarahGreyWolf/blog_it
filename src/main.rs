use std::{cmp::Ordering, fmt::Display, io};

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
                let post_string = match std::fs::read_to_string(file.path()) {
                    Ok(pt) => pt,
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
        match std::fs::write(format!("./site/posts/{}.html", file_name), output) {
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

    generate_home(latest_posts)?;
    generate_posts_list(&posts)?;

    Ok(())
}

fn generate_home(posts: &[Post]) -> io::Result<()> {
    let home_template = match std::fs::read_to_string("./templates/home.html") {
        Ok(pt) => pt,
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
    match std::fs::write("./site/index.html", output) {
        Ok(_) => {}
        Err(e) => panic!("Couldn't write to site/index.html: {e}"),
    };
    Ok(())
}

fn generate_posts_list(posts: &[Post]) -> io::Result<()> {
    let posts_template = match std::fs::read_to_string("./templates/posts.html") {
        Ok(pt) => pt,
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
    match std::fs::write("./site/posts/index.html", output) {
        Ok(_) => {}
        Err(e) => panic!("Couldn't write to site/index.html: {e}"),
    };

    Ok(())
}
