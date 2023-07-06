#![allow(clippy::too_many_arguments)]
use pulldown_cmark::{html, Options, Parser};

wit_bindgen::generate!(in "markdown-html.wit");

pub struct PulldownCmark;

export_commonmark!(PulldownCmark);

impl Commonmark for PulldownCmark {
    fn commonmark_to_html(
        md: wit_bindgen::rt::string::String,
        ext: ExtensionOptions,
    ) -> wit_bindgen::rt::string::String {
        let opt = {
            let mut opt = Options::empty();
            opt.set(Options::ENABLE_TABLES, ext.tables);
            opt.set(Options::ENABLE_FOOTNOTES, ext.footnotes);
            opt.set(Options::ENABLE_STRIKETHROUGH, ext.strikethrough);
            opt.set(Options::ENABLE_TASKLISTS, ext.tasklist);
            opt.set(Options::ENABLE_SMART_PUNCTUATION, ext.smart_punctuation);
            opt.set(Options::ENABLE_HEADING_ATTRIBUTES, ext.heading_attributes);
            opt
        };

        let parser = Parser::new_ext(&md, opt);

        let mut html = String::new();
        html::push_html(&mut html, parser);

        html
    }
}
