//! Contains structs and functions to parse Debian-styled RFC 822 files.
use core::iter::Iterator;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
/// The result of a parsing error.
pub struct ParserError {
	pub msg: String,
	pub line: Option<usize>,
}

impl fmt::Display for ParserError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let Some(num) = self.line {
			write!(f, "{} at line '{num}'", self.msg)?
		} else {
			write!(f, "{}", self.msg)?
		}
		Ok(())
	}
}

impl std::error::Error for ParserError {}

/// A section in a TagFile. A TagFile is made up of double-newline (`\n\n`)
/// separated paragraphs, each of which make up one of these sections.
#[derive(Debug)]
pub struct TagSection {
	data: HashMap<String, String>,
}

impl From<TagSection> for HashMap<String, String> {
	fn from(value: TagSection) -> Self { value.data }
}

impl TagSection {
	fn error(msg: &str, line: Option<usize>) -> Result<Self, ParserError> {
		Err(ParserError {
			msg: "E:".to_owned() + msg,
			line,
		})
	}

	fn line_is_key(line: &str) -> bool { !line.starts_with(' ') && !line.starts_with('\t') }

	fn next_line_extends_value(lines: &[&str], current_line: usize) -> bool {
		if let Some(next_line) = lines.get(current_line + 1) {
			!Self::line_is_key(next_line)
		} else {
			false
		}
	}

	/// Create a new [`TagSection`] instance.
	/// # Returns
	/// * A [`Result`]: The [`Ok`] variant if there was no issue parsing the
	///   section, and the [`Err`] variant if there was.
	pub fn new(section: &str) -> Result<Self, ParserError> {
		// Make sure the string doesn't contain multiple sections.
		if section.contains("\n\n") {
			return Self::error("More than one section was found", None);
		}

		// Make sure the user didn't pass an empty string.
		if section.is_empty() {
			return Self::error("An empty string was passed", None);
		}

		// Start building up the HashMap.
		let mut data = HashMap::new();
		let lines = section.lines().collect::<Vec<&str>>();

		// Variables used while parsing.
		let mut current_key: Option<String> = None;
		let mut current_value = String::new();

		for (index, line) in lines.iter().enumerate() {
			// Indexes start at 0, so increase by 1 to get the line number.
			let line_number = index + 1;

			// If this line starts with a comment ignore it.
			if line.starts_with('#') {
				continue;
			}

			// If this line is a new key, split the line into the key and its value.
			if Self::line_is_key(line) {
				let (key, value) = match line.split_once(':') {
					Some((key, value)) => {
						(key.to_string(), value.strip_prefix(' ').unwrap_or(value))
					},
					None => {
						return Self::error(
							"Line doesn't contain a ':' separator",
							Some(line_number),
						);
					},
				};

				// Set the current key and value.
				// If the value is empty, then this is a multiline field, and it's going to be
				// one of these things:
				// 1. A multiline field, in which case we want to add a
				// newline to reflect such.
				// 2. A key with an empty value, in which case it will
				// be removed post-processing.
				current_key = Some(key);

				if value.is_empty() {
					current_value = "\n".to_string();
				} else {
					current_value = value.to_string();

					// If the next extends the value, add the newline before it.
					if Self::next_line_extends_value(&lines, index) {
						current_value += "\n";
					}
				}
			}

			// If this line is indented with spaces or tabs, add it to the current value.
			// This should never end up running in conjunction with the above `if` block.
			if line.starts_with(' ') || line.starts_with('\t') {
				current_value += line;

				// If the next line extends the value, add the newline. `line_number`
				// conveniently is the next index, so use that to our advantage.
				if Self::next_line_extends_value(&lines, index) {
					current_value += "\n";
				}
			}

			// If the next line is a new key or this is the last line, add the current key
			// and value to the HashMap. `line_number` conveniently is the next index, so
			// use that to our advantage.
			if !Self::next_line_extends_value(&lines, index) {
				// If no key exists, we've defined a paragraph (at the beginning of the control
				// file) with no key. This would be parsed at the very beginning, but the file
				// may have an unknown amount of comment lines, so we just do this here as a
				// normal step of the parsing stage.
				if current_key.is_none() {
					return Self::error(
						"No key defined for the currently indented line",
						Some(line_number),
					);
				}

				// Add the key and reset the `current_key` and `current_value` counters.
				data.insert(current_key.unwrap(), current_value);
				current_key = None;
				current_value = String::new();
			}
		}

		Ok(Self { data })
	}

	/// Get the underlying [`HashMap`] used in the generated [`TagSection`].
	pub fn hashmap(&self) -> &HashMap<String, String> { &self.data }

	/// Get the value of the specified key.
	pub fn get(&self, key: &str) -> Option<&String> { self.data.get(key) }

	/// Get the value of the specified key,
	///
	/// Returns specified default on failure.
	pub fn get_default<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
		if let Some(value) = self.data.get(key) {
			return value;
		}
		default
	}
}

/// Parses a TagFile: these are files such as Debian `control` and `Packages`
/// files.
///
/// # Returns
/// * A [`Result`]: The [`Ok`] variant containing the vector of [`TagSection`]
///   objects if there was no issue parsing the file, and the [`Err`] variant if
///   there was.
pub fn parse_tagfile(content: &str) -> Result<Vec<TagSection>, ParserError> {
	let mut sections = vec![];
	let section_strings = content.split("\n\n");

	for (iter, section) in section_strings.clone().enumerate() {
		// If this section is empty (i.e. more than one empty line was placed between
		// each section), then ignore this section.
		if section.is_empty() || section.chars().all(|c| c == '\n') {
			break;
		}

		match TagSection::new(section) {
			Ok(section) => sections.push(section),
			Err(mut err) => {
				// If an error line was provided, add the number of lines in the sections before
				// this one. Otherwise no line was specified, and we'll just specify the number
				// of lines in the section before this one so we know which section the line is
				// in.
				let mut line_count = 0;

				for _ in 0..iter {
					// Add one for the line separation between each section.
					line_count += 1;

					// Add the line count in this section.
					line_count += section_strings.clone().count();
				}

				if let Some(line) = err.line {
					err.line = Some(line_count + line);
				} else {
					err.line = Some(line_count);
				}
			},
		}
	}

	Ok(sections)
}
