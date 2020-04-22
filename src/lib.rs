#![feature(trace_macros)]
#![allow(dead_code)]
#![allow(unused_imports)]
/// Re-export of console::Key
///
pub use console::Key;
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirmation, Input, Select};
use std::fmt;
use std::io;
use std::io::Read;
use std::sync;

pub fn _redir_stdout() -> (sync::mpsc::Sender<bool>, sync::mpsc::Receiver<String>) {
    let (tx, rx) = sync::mpsc::channel();
    let (tk, rk) = sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut shh = shh::stdout().unwrap();
        while !rk.try_recv().is_ok() {
            let mut buf = String::new();
            shh.read_to_string(&mut buf).ok();
            tx.send(buf).ok();
        }
    });
    (tk, rx)
}

pub fn _unredir_stdout(arg: (sync::mpsc::Sender<bool>, sync::mpsc::Receiver<String>)) -> String {
    let (tx, rx) = arg;
    tx.send(true).unwrap();
    println!("");
    let mut buf = String::new();
    if let Ok(s) = rx.recv() {
        buf.push_str(&s);
    }
    buf
}

/// Wrapper around console::Term that counts lines for easy resetting.
///
/// A diralogue has a "Log" section above that logs each input,
/// and a "Display" section below that shows the current menuitem.
/// While implementing this pattern, I found that i was manually counting
/// how many lines i had written to the screen so that i could clear them
/// and get back to the Log once i was done. This wrapper around Term
/// simplifies that process.
///
/// Rather than using term directly, use TrackedTerm.write_line to output
/// to the screen. It works just like Term.write_line, but increments the
/// line count for every line of input. You can even give multi-line
/// &strs to TrackedTerm.write_line, and it will split it up and increment
/// the count correctly.
///
/// Once your menuitem has completed its task (or completed a loop iteration
/// and is going to ask for input, etc etc), call TrackedTerm.reset() to
/// clear all written lines and return to the log.
///
/// # Examples
///
/// Please note the usage of two TrackedTerm.line_break calls before anything
/// else is printed to the screen. Your implementation of MenuItem.exec
/// should also do this.
/// ### Code
/// ```
/// use diraloguer::{TrackedTerm, Key};
/// # struct Dummy();
/// # impl Dummy {
/// // Within MenuItem impl
/// fn exec(&self, term: &mut TrackedTerm) {
///     loop {
///         term.line_break();
///         term.line_break();
///         term.write_line("This is my cool informational page!");
///         term.line_break();
///         term.write_line("Please feel free to press 'q' to exit my cool page!");
///         let key = term.read_key();
///         term.reset();
///         match key {
///             # // We're doing a little bit of a lie.
///             # // This test doesn't actually work as shown, since no keys are ever pressed.
///             # // Luckily, since test isn't user-accessible, Term.read_key returns Key::Unknown,
///             # // so we can use that to pass the test
///             # Key::Unknown => break,
///             Key::Char('q') => break,
///             // Notice how we use println! now that we've reset.
///             // We're writing to the log here, so we don't want to track this line.
///             // If we did, it would get erased the next time we reset!
///             _ => println!("That isn't 'q' silly!")
///         }
///     }
/// }
/// # }
/// # fn main() {
/// #     Dummy().exec(&mut TrackedTerm::stdout());
/// # }
/// ```
/// ### Output:
/// ```text
/// ------Log------
///
///
/// This is my cool informational page!
///
/// Please feel free to press 'q' to exit my cool page!
/// ```
/// (After pressing 'Escape')
/// ```text
/// ------Log------
/// That isn't 'q' silly!
///
///
/// This is my cool informational page!
///
/// Please feel free to press 'q' to exit my cool page!
/// ```
/// (After pressing 'q')
/// ```text
/// ------Log------
/// That isn't 'q' silly!
///
///
/// <Parent Menu Displayed>
/// ```
pub struct TrackedTerm(Term, usize, usize);

impl TrackedTerm {
    /// Get a new TrackedTerm for stdout.
    pub fn stdout() -> Self {
        Self(Term::stdout(), 0, 0)
    }
    /// Get a reference to the underlying console::Term
    pub fn unwrap(&self) -> &Term {
        &self.0
    }
    /// Writes an empty line to the screen, incrementing the line count.
    pub fn line_break(&mut self) {
        self.write_line("\n");
    }
    /// Writes a line to the screen and increments the line count.
    ///
    /// Writes the given string to stdout and increments the line count.
    /// This method also handles multi-line strings.
    pub fn write_line(&mut self, s: &str) {
        for line in s.lines() {
            self.0.write_line(line).ok();
            self.1 += 1;
        }
    }
    /// Clears the written lines and resets the line count.
    pub fn reset(&mut self) {
        if self.2 > 0 {
            self.move_cursor_down(self.2);
        }
        self.0.clear_last_lines(self.1 + 1).ok();
        self.1 = 0;
    }
    /// Clear the last line without modifying the line count.
    ///
    /// This method should only be used when a line was written to the
    /// screen without going through this wrapper.
    pub fn force_clear_line(&mut self) {
        self.0.clear_last_lines(1).ok();
    }
    /// Works just like Term.clear_last_lines, but also tracks the reduced line count.
    pub fn clear_last_lines(&mut self, n: usize) {
        if n > (self.1 - self.2) {
            self.0.clear_last_lines(self.1 - self.2).ok();
            self.2 += self.1 - self.2;
        } else {
            self.0.clear_last_lines(n).ok();
            self.2 += n;
        }
    }
    /// Works just like Term.move_cursor_up, but also tracks that movement.
    ///
    /// For more information, see [move_cursor_down](#method.move_cursor_down).
    pub fn move_cursor_up(&mut self, n: usize) {
        self.2 += n;
        self.0.move_cursor_up(n).ok();
    }
    /// Works just like Term.move_cursor_down, but also tracks that movement.
    ///
    /// Cursor position is tracked with TrackedTerm.2.
    /// Every movement upwards increments self.2 by 1,
    /// and every movement downwards decrements self.2.
    /// Once self.2 reaches zero, move_cursor_down is
    /// basically the same as line_break, so further movement
    /// downwards instead increments self.1
    pub fn move_cursor_down(&mut self, n: usize) {
        match self.2.checked_sub(n) {
            Some(r) => self.2 = r,
            None => {
                self.1 += match n.checked_sub(self.2) {
                    Some(r) => r,
                    None => panic!("This can't happen"),
                };
                self.2 = 0;
            }
        };
        self.0.move_cursor_down(n).ok();
    }
    /// Convienience function to access the underlying Term.read_key method.
    pub fn read_key(&self) -> Key {
        self.0.read_key().unwrap()
    }
}

/// The menuitem trait.
/// Implement this to make your own menu items.
///
/// For details on implementing your own menuitems,
/// please see the [Information Documentation]()
pub trait MenuItem {
    /// Returns the title of the menuitem.
    ///
    /// The title is what is displayed in a parent menu.
    fn name(&self) -> &str;
    /// Called by the parent menu when the item is activated.
    ///
    /// It should implement all the logic of displaying and interacting with the page.
    /// On return, the user is taken back to the parent menu.
    fn exec(&mut self, term: &mut TrackedTerm);
    /// Returns whether the menu item is enabled.
    ///
    /// An "enabled" menu item (indicated by this method returning true) can be selected and activated by a user.
    /// A "disabled" menu item (indicated by this method returning false) cannot, instead the user
    /// will skip over it.
    ///
    /// Given this example menu, successive presses of the down arrow will change the display as shown:
    /// ```text
    /// Main Menu
    ///  > This item returns true when is_enabled is called
    ///    So does this one
    ///    This one returns false!
    ///    This one is back to true.
    /// ```
    /// ```text
    /// Main Menu
    ///    This item returns true when is_enabled is called
    ///  > So does this one
    ///    This one returns false!
    ///    This one is back to true.
    /// ```
    /// ```text
    /// Main Menu
    ///    This item returns true when is_enabled is called
    ///    So does this one
    ///    This one returns false!
    ///  > This one is back to true.
    /// ```
    fn is_enabled(&self) -> bool;
}

/// A menu
///
/// These are automatically generated by the diraloguer! macro.
///
/// # Example
/// ```rust
/// # use diraloguer::diralogue;
/// # use diraloguer::{_redir_stdout, _unredir_stdout};
/// # let arg = _redir_stdout();
/// println!("test");
/// diralogue!("Title", "Prompt" => (default = 1; confirmation = "Are you sure you want to quit?";) [
///     "Title" => println!("You just selected the first item!");
/// ]);
/// # println!("{:?}", _unredir_stdout(arg));
/// ```
/// ```text
/// Prompt (defaults to Title)
///     Title
///   > Title
/// ```
pub struct Directory {
    title: String,
    prompt: String,
    items: Vec<Box<dyn MenuItem>>,
    selected: usize,
    exit_confirmation: Option<String>,
}

impl Directory {
    pub fn new(title: &str) -> Directory {
        Directory {
            title: String::from(title),
            prompt: String::from(title),
            items: Vec::new(),
            selected: 0,
            exit_confirmation: None,
        }
    }
    /// Set a custom prompt to display when the menu executes
    ///
    /// # Examples
    ///
    pub fn prompt(&mut self, prompt: &str) {
        self.prompt = String::from(prompt);
    }
    pub fn item(&mut self, item: Box<dyn MenuItem>) {
        self.items.push(item);
    }
    pub fn default(&mut self, def: usize) {
        self.selected = def;
    }
    pub fn confirmation(&mut self, prompt: &str) {
        self.exit_confirmation = Some(String::from(prompt));
    }
    pub fn run(&mut self) {
        println!("------Log------");
        self.exec(&mut TrackedTerm::stdout());
    }
}

impl MenuItem for Directory {
    fn name(&self) -> &str {
        &self.title
    }
    fn exec(&mut self, term: &mut TrackedTerm) {
        loop {
            term.line_break();
            term.line_break();
            let items: Vec<&str> = self.items.iter().map(|mi| mi.name()).collect();
            let rv = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(&self.prompt)
                .items(&items)
                .default(self.selected)
                .clear(true)
                .interact_opt()
                .unwrap();
            // term.force_clear_line();
            match rv {
                Some(v) => {
                    term.reset();
                    self.selected = v;
                    println!("{}: {}", self.prompt, style(items[v]).green());
                    self.items[v].exec(term);
                }
                None => {
                    term.reset();
                    if let Some(ec) = self.exit_confirmation.as_ref() {
                        if Confirmation::new().with_text(&ec).interact().unwrap() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

pub struct Text {
    content: String,
}

impl MenuItem for Text {
    fn name(&self) -> &str {
        &self.content
    }
    fn exec(&mut self, _term: &mut TrackedTerm) {}
    fn is_enabled(&self) -> bool {
        false
    }
}

pub struct Function {
    pub title: String,
    pub func: Box<dyn Fn() -> ()>,
}

impl MenuItem for Function {
    fn name(&self) -> &str {
        &self.title
    }
    fn exec(&mut self, _term: &mut TrackedTerm) {
        (self.func)();
    }
    fn is_enabled(&self) -> bool {
        true
    }
}

pub struct Toggle {
    title: String,
    content: String,
    value: bool,
    true_text: String,
    false_text: String,
}

impl Toggle {
    pub fn new(text: &str) -> Toggle {
        let mut t = Toggle {
            title: String::from(text),
            content: String::from(text),
            value: false,
            true_text: String::from("true"),
            false_text: String::from("false"),
        };
        t.update_content();
        t
    }
    pub fn value(mut self, value: bool) -> Self {
        self.value = value;
        self.update_content();
        self
    }
    pub fn true_text(mut self, text: &str) -> Self {
        self.true_text = String::from(text);
        self.update_content();
        self
    }
    pub fn false_text(mut self, text: &str) -> Self {
        self.false_text = String::from(text);
        self.update_content();
        self
    }
    fn update_content(&mut self) {
        let text = if self.value {
            &self.true_text
        } else {
            &self.false_text
        };
        self.content = format!("{}: {}", &self.title, text);
    }
}

impl MenuItem for Toggle {
    fn name(&self) -> &str {
        &self.content
    }
    fn exec(&mut self, _: &mut TrackedTerm) {
        self.value = !self.value;
        self.update_content();
    }
    fn is_enabled(&self) -> bool {
        true
    }
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! diralogue {
    ($title:expr $(,$prompt:expr)? => $(($($method:ident $(= $args:expr)?;)+))?[ $($inner:tt)+ ]) => {{
        let mut dir = diraloguer::Directory::new($title);
        $(dir.prompt($prompt);)?
        $(
            $(dir.$method($($args)*);)+
        )?
        let mut dirs = Vec::new();
        dirs.push(dir);
        diralogue!(item dirs 0, $($inner)+);
        dirs[0].run();
        // $(dir.item(diralogue!($($inner)+;)))+;
    }};

    (item $outer:ident $id:expr, $title:expr $(,$prompt:expr)? => $(($($method:ident $(= $args:expr)?;)+))?[ $($inner:tt)+ ]; $($rest:tt)*) => {{
        $outer.push(diraloguer::Directory::new($title));
        diralogue!(item $outer $id+1, $($inner)+);
        let mut this = $outer.pop().unwrap();
        $(
            $(this.$method($($args)*))+;
        )?
        $outer[$id].item(Box::new(this));
        diralogue!(item $outer $id, $($rest)*);
    }};

    (item $outer:ident $id:expr, $inner:expr; $($rest:tt)*) => {{
        let mut parent = $outer.pop().unwrap();
        parent.item(Box::new($inner));
        $outer.push(parent);
        diralogue!(item $outer $id, $($rest)*);
    }};

    (item $outer:ident $id:expr, $title:expr => $inner:expr; $($rest:tt)*) => {{
        let mut parent = $outer.pop().unwrap();
        let item = diraloguer::Function {
            title: String::from($title),
            func: Box::new(|| $inner),
        };
        parent.item(Box::new(item));
        $outer.push(parent);
        diralogue!(item $outer $id, $($rest)*);
    }};

    (item $outer:ident $id:expr, ) => {()};
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {}
}
