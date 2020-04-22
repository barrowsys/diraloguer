//! # Implementing a MenuItem
//!
//! This is going to be a bit of a long one.
//! We are going to focus on creating a "Information" type,
//! which will utilize the builder pattern in order to be explicit.
//! Notice: much of the given example code is untested.
//! All of the examples given here are included in the
//!
//! When activated, this menuitem will display a number of paragraphs of text.
//! After that, it will display instructions to the user,
//! indicating options to "save to file" and "exit",
//! controlled by the keys "s" and "q".
//!
//! ```rust,ignore
//! diralogue! {
//! "Main Menu" {
//!     Paragraph::new("Lorem Ipsum")
//!         .paragraph("Lorem Ipsum is a dummy text used in the printing and typesetting industry. It
//!         has been the industry standard dummy text since the 1500s. It was popularized in the 1960s
//!         with the release of Letraset sheets containing Lorem Ipsum passages.")
//!         .paragraph("It is a long established fact that the reader will be distracted by the content
//!         of a page when looking at its layout. The point of reading lorem ipsum is that it has a
//!         more-or-less normal distribution of letters, as opposed to reading 'Content here, content
//!         here', which looks like readable english.")
//!         .paragraph("There are many variations of lorem ipsum available, but the majority have been
//!         altered in some form, by injected humor or randomized words. If you are going to use a
//!         passage of lorem ipsum, be sure there isn't anything embarrassing hidden in the middle of
//!         the text."),
//!     "Exit" => exit!
//! }
//! ```
//! The above code will generate the following menu.
//! ```text
//! Main Menu
//!  > Lorem Ipsum
//!    Exit
//! ```
//! and if the enter key is pressed
//! ```text
//! Lorem Ipsum
//!
//! Lorem Ipsum is a dummy text used in the printing and typesetting industry. It
//! has been the industry standard dummy text since the 1500s. It was popularized in
//! the 1960s with the release of Letraset sheets containing Lorem Ipsum passages.
//!
//! It is a long established fact that the reader will be distracted by the content
//! of a page when looking at its layout. The point of reading lorem ipsum is that it
//! has a more-or-less normal distribution of letters, as opposed to reading 'Content here,
//! content here', which looks like readable english.
//!
//! There are many variations of lorem ipsum available, but the majority have been
//! altered in some form, by injected humor or randomized words. If you are going to
//! use a passage of lorem ipsum, be sure there isn't anything embarrassing hidden in the
//! middle of the text.
//!
//!   Press 's' to save to a file, or 'q' to go back.
//! ```
#![allow(dead_code)]
use diraloguer::{MenuItem, TrackedTerm};

pub struct Paragraph {
    title: String,
    paragraphs: Vec<String>,
}

impl Paragraph {
    fn new(title: &str) -> Paragraph {
        Paragraph {
            title: String::from(title),
            paragraphs: Vec::new(),
        }
    }
    fn paragraph(&mut self, text: &str) {
        self.paragraphs.push(String::from(text));
    }
}

impl MenuItem for Paragraph {
    fn name(&self) -> &str {
        &self.title
    }
    fn exec(&mut self, term: &mut TrackedTerm) {
        term.write_line(&self.title);
        term.write_line("");
        for p in self.paragraphs.iter() {
            term.write_line(&p);
            term.write_line("");
        }
        term.write_line("");
        term.write_line("  Press 's' to save to a file, or 'q' to go back");
    }
    fn is_enabled(&self) -> bool {
        true
    }
}
