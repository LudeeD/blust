mod themes;

use anyhow::{anyhow, Result};
use glob::glob;
use handlebars::{Handlebars, JsonValue};
use pulldown_cmark::Tag::Heading;
use pulldown_cmark::{html, BrokenLink, Event, HeadingLevel, Options, Parser};
use serde_json::json;
use std::path::Path;
use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use themes::{CSS, TEMPLATE_INDEX, TEMPLATE_NOTA};

fn parse_to_html(in_path: PathBuf) -> Result<String> {
    let buffer: String = fs::read_to_string(&in_path)?
        .parse()
        .expect("TODO remove expects");
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&buffer, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    Ok(html_output)
}

pub fn parse(in_file: &Path) -> Result<String> {
    println!("Parsing...");

    let mut title: Option<String> = None;
    let mut last_event = None;
    let mut buffer = String::new();
    let mut in_file = File::open(in_file)?;
    in_file.read_to_string(&mut buffer).expect("#TODO change");
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    // Setup callback that sets the URL and title when it encounters
    // a reference to our home page.
    let callback = &mut |_broken_link: BrokenLink| {
        println!("#TODO, need to handle links");
        None
    };
    let parser = Parser::new_with_broken_link_callback(&buffer, options, Some(callback));
    for element in parser {
        match &element {
            Event::Start(Heading(HeadingLevel::H1, _, _)) => {
                last_event = Some(element);
            }
            Event::Text(t) => {
                if let Some(Event::Start(Heading(HeadingLevel::H1, _, _))) = last_event {
                    if title.is_none() {
                        title = Some(t.to_string());
                    }
                }
            }
            _ => (),
        }
    }
    println!("title => {:?}", title);
    title.ok_or_else(|| anyhow!("could not find title"))
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let outpath = args.get(1);
    if outpath.is_none() {
        println!("No outpath provided");
        return Err(anyhow!("oh no! no outpath provided"));
    }
    let outpath = PathBuf::from(outpath.unwrap());
    let path = env::current_dir()?;

    let mut hbs = Handlebars::new();
    hbs.register_template_string(
        "nota",
        String::from_utf8(TEMPLATE_NOTA.to_vec()).expect("TODO"),
    )
    .expect("TODO");
    hbs.register_template_string(
        "index",
        String::from_utf8(TEMPLATE_INDEX.to_vec()).expect("TODO"),
    )
    .expect("TODO");

    let mut static_folder = outpath.clone();
    static_folder.push("static");
    fs::create_dir_all(&static_folder).expect("TODO");

    static_folder.push("style.css");

    let mut file = File::create(static_folder).expect("TODO");
    file.write_all(CSS).expect("TODO");

    let g = format!("{}/**/*.md", path.display());

    let mut index_files = Vec::new();

    for path in glob(&g).unwrap().filter_map(Result::ok) {
        let mut output_file = outpath.clone();
        let file_name = path.file_name().expect("TODO");
        print!("Found {:?} ", file_name);
        let title = parse(&path)?;
        let metadata = fs::metadata(file_name)?;
        let time = metadata.modified().unwrap();

        let filename = path.clone();
        let filename = filename.file_name().unwrap().to_owned();

        let html_output = parse_to_html(path).unwrap();
        let render = hbs
            .render("nota", &json!({ "content": html_output }))
            .expect("TODO");

        output_file.push(&filename);
        output_file.set_extension("html");
        let mut file = File::create(output_file.clone()).unwrap();
        file.write_all(render.as_bytes()).expect("TODO");

        let demo = output_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        index_files.push((demo, title, time));
    }

    index_files.sort_by(|(_, _, a), (_, _, b)| a.partial_cmp(b).unwrap());

    let mut index_file = outpath.clone();
    index_file.push("index");
    index_file.set_extension("html");

    let demo: Vec<JsonValue> = index_files
        .iter()
        .map(|nota| json!({"title": nota.1, "link": nota.0}))
        .collect();

    println!("{:?}", demo);

    let render = hbs
        .render("index", &json!({ "people": demo }))
        .expect("TODO");

    let mut file = File::create(index_file).unwrap();
    file.write_all(&render.as_bytes()).expect("TODO");

    Ok(())
}
