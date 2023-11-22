#![feature(path_file_prefix)]

use std::{
    cmp::Ordering,
    error::Error,
    fmt::Display,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use chrono::{DateTime, NaiveDate, Utc};

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
    fn produced_html(&self) -> Result<String, Box<dyn Error>> {
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
            let linked_paragraph = parser::convert_links(&paragraph)?;
            // TODO: Handle markdown links
            if linked_paragraph.is_empty() {
                block = block.trim_end_matches("<br>").to_owned();
                block.push_str("</p>");
                main.push_str(&block);
                block = String::from("<p>");
                continue;
            }
            block.push_str(&format!("{linked_paragraph}"));
            block.push_str("<br>");
        }
        block = block.trim_end_matches("<br>").to_owned();
        block.push_str("</p>");
        main.push_str(&block);

        Ok(main)
    }
}

struct Details {
    name: String,
    username: String,
    age: String,
    email: String,
    pronouns: String,
}

impl Details {
    pub fn new() -> Details {
        Details {
            name: String::from("Sarah"),
            username: String::from("SarahGreyWolf"),
            age: generate_age(),
            email: String::from("m.sarahgreywolf@outlook.com"),
            pronouns: String::from("She/Her"),
        }
    }

    pub fn modify_text(&self, input: &mut String) {
        *input = input.replace("{{% NAME %}}", &self.name);
        *input = input.replace("{{% USERNAME %}}", &self.username);
        *input = input.replace("{{% AGE %}}", &self.age);
        *input = input.replace("{{% EMAIL %}}", &self.email);
        *input = input.replace("{{% PRONOUNS %}}", &self.pronouns);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut options = OpenOptions::new();
    options.create(true).write(true).read(true);
    let post_template = match std::fs::read_to_string("./templates/post.html") {
        Ok(pt) => pt,
        Err(e) => panic!("Couldn't get post template: {e}"),
    };

    let details = Details::new();

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
        let mut output = post_template.replace("{{% POST %}}", &post.produced_html()?);
        output = output.replace("{{% POST_TITLE %}}", &post.title);
        details.modify_text(&mut output);

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

    generate_home(&options, &details, latest_posts)?;
    generate_posts_list(&options, &details, &posts)?;

    Ok(())
}

struct Template {
    file_name: String,
    file: Option<File>,
    content: String,
}

impl Template {
    pub fn new(open_opts: &OpenOptions, file_path: &Path) -> std::io::Result<Template> {
        let Some(os_file_name) = file_path.file_prefix() else {
            panic!(
                "Could not get file prefix from path: {}",
                file_path.display()
            );
        };
        let Some(file_name) = os_file_name.to_str() else {
            panic!("Could not convert OsStr to str: {:?}", os_file_name);
        };
        let file = match open_opts.open(file_path) {
            Ok(f) => f,
            Err(e) => panic!("Could not open file {}: {e}", file_path.display()),
        };
        Ok(Template {
            file_name: file_name.to_string(),
            file: Some(file),
            content: String::new(),
        })
    }
    pub fn load(&mut self) {
        let Some(file) = &mut self.file else {
            panic!("File for {} was None", self.file_name);
        };
        match file.read_to_string(&mut self.content) {
            Ok(_) => {}
            Err(e) => panic!("Could not read template {}: {e}", self.file_name),
        }
    }
}

fn generate_home(
    options: &OpenOptions,
    details: &Details,
    posts: &[Post],
) -> Result<(), Box<dyn Error>> {
    let mut template = Template::new(&options, &PathBuf::from("./templates/home.html"))?;
    template.load();
    let mut output = String::new();
    for post in posts {
        if post.is_draft {
            continue;
        }
        let file_name = post.title.replace(' ', "_");
        output.push_str(&format!(
            "<li><a href=\"/posts/{}.html\">{}</a></li>",
            file_name, post.title
        ));
    }
    output = template.content.replace("{{% LATEST %}}", &output);
    details.modify_text(&mut output);
    let complete = parser::convert_links(&output)?;
    let mut file = match options.open("./site/index.html") {
        Ok(f) => f,
        Err(e) => panic!("Could not open ./site/index.html: {e}"),
    };
    file.write_all(complete.as_bytes())?;
    Ok(())
}

fn generate_posts_list(
    options: &OpenOptions,
    details: &Details,
    posts: &[Post],
) -> Result<(), Box<dyn Error>> {
    let mut template = Template::new(&options, &PathBuf::from("./templates/posts.html"))?;
    template.load();
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
    output = template.content.replace("{{% POSTS %}}", &output);
    details.modify_text(&mut output);
    let complete = parser::convert_links(&output)?;
    let mut file = match options.open("./site/posts/index.html") {
        Ok(f) => f,
        Err(e) => panic!("Could not open ./site/posts/index.html: {e}"),
    };
    match file.write(complete.as_bytes()) {
        Ok(_) => {}
        Err(e) => panic!("Couldn't write to ./site/posts/index.html: {e}"),
    };

    Ok(())
}

fn generate_age() -> String {
    let current_utc: DateTime<Utc> = Utc::now();
    let naive = current_utc.date_naive();
    let Some(birth) = NaiveDate::from_ymd_opt(1995, 10, 29) else {
        panic!("Could not get date from from_ymd_opt(1995, 10, 29)");
    };
    let Some(age) = naive.years_since(birth) else {
        panic!("Could not get age")
    };

    age.to_string()
}
