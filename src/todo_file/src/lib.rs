// LINT-REPLACE-START
// This section is autogenerated, do not modify directly
// nightly sometimes removes/renames lints
#![cfg_attr(allow_unknown_lints, allow(unknown_lints))]
#![cfg_attr(allow_unknown_lints, allow(renamed_and_removed_lints))]
// enable all rustc's built-in lints
#![deny(
	future_incompatible,
	nonstandard_style,
	rust_2018_compatibility,
	rust_2018_idioms,
	rust_2021_compatibility,
	unused,
	warnings
)]
// rustc's additional allowed by default lints
#![deny(
	absolute_paths_not_starting_with_crate,
	deprecated_in_future,
	elided_lifetimes_in_paths,
	explicit_outlives_requirements,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	non_ascii_idents,
	noop_method_call,
	pointer_structural_match,
	rust_2021_incompatible_closure_captures,
	rust_2021_incompatible_or_patterns,
	rust_2021_prefixes_incompatible_syntax,
	rust_2021_prelude_collisions,
	single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unsafe_code,
	unsafe_op_in_unsafe_fn,
	unstable_features,
	unused_crate_dependencies,
	unused_extern_crates,
	unused_import_braces,
	unused_lifetimes,
	unused_macro_rules,
	unused_qualifications,
	unused_results,
	variant_size_differences
)]
// enable all of Clippy's lints
#![deny(clippy::all, clippy::cargo, clippy::pedantic, clippy::restriction)]
#![cfg_attr(include_nightly_lints, deny(clippy::nursery))]
#![allow(
	clippy::arithmetic,
	clippy::blanket_clippy_restriction_lints,
	clippy::default_numeric_fallback,
	clippy::else_if_without_else,
	clippy::expect_used,
	clippy::float_arithmetic,
	clippy::implicit_return,
	clippy::indexing_slicing,
	clippy::integer_arithmetic,
	clippy::map_err_ignore,
	clippy::missing_docs_in_private_items,
	clippy::mod_module_files,
	clippy::module_name_repetitions,
	clippy::new_without_default,
	clippy::option_if_let_else,
	clippy::pub_use,
	clippy::redundant_pub_crate,
	clippy::std_instead_of_alloc,
	clippy::std_instead_of_core,
	clippy::tabs_in_doc_comments,
	clippy::too_many_lines,
	clippy::unwrap_used
)]
#![deny(
	rustdoc::bare_urls,
	rustdoc::broken_intra_doc_links,
	rustdoc::invalid_codeblock_attributes,
	rustdoc::invalid_html_tags,
	rustdoc::missing_crate_level_docs,
	rustdoc::private_doc_tests,
	rustdoc::private_intra_doc_links
)]
#![cfg_attr(
	include_nightly_lints,
	allow(clippy::arithmetic_side_effects, clippy::bool_to_int_with_if)
)]
// LINT-REPLACE-END

//! Git Interactive Rebase Tool - Todo File Module
//!
//! # Description
//! This module is used to handle working with the rebase todo file.

mod action;
mod edit_content;
pub mod errors;
mod history;
mod line;
mod utils;

use std::{
	fs::{read_to_string, File},
	io::Write,
	path::{Path, PathBuf},
	slice::Iter,
};

pub use self::{action::Action, edit_content::EditContext, line::Line};
use self::{
	history::{History, HistoryItem},
	utils::{remove_range, swap_range_down, swap_range_up},
};
use crate::errors::{FileReadErrorCause, IoError};

/// Represents a rebase file.
#[derive(Debug)]
pub struct TodoFile {
	comment_char: String,
	filepath: PathBuf,
	history: History,
	is_noop: bool,
	lines: Vec<Line>,
	selected_line_index: usize,
}

impl TodoFile {
	/// Create a new instance.
	#[must_use]
	#[inline]
	pub fn new<Path: AsRef<std::path::Path>>(path: Path, undo_limit: u32, comment_char: &str) -> Self {
		Self {
			comment_char: String::from(comment_char),
			filepath: PathBuf::from(path.as_ref()),
			history: History::new(undo_limit),
			lines: vec![],
			is_noop: false,
			selected_line_index: 0,
		}
	}

	/// Set the rebase lines.
	#[inline]
	pub fn set_lines(&mut self, lines: Vec<Line>) {
		self.is_noop = !lines.is_empty() && lines[0].get_action() == &Action::Noop;
		self.lines = if self.is_noop {
			vec![]
		}
		else {
			lines.into_iter().filter(|l| l.get_action() != &Action::Noop).collect()
		};
		if self.selected_line_index >= self.lines.len() {
			self.selected_line_index = if self.lines.is_empty() { 0 } else { self.lines.len() - 1 };
		}
		self.history.reset();
	}

	/// Load the rebase file from disk.
	///
	/// # Errors
	///
	/// Returns error if the file cannot be read.
	#[inline]
	pub fn load_file(&mut self) -> Result<(), IoError> {
		let lines: Result<Vec<Line>, IoError> = read_to_string(self.filepath.as_path())
			.map_err(|err| {
				IoError::FileRead {
					file: self.filepath.clone(),
					cause: FileReadErrorCause::from(err),
				}
			})?
			.lines()
			.filter_map(|l| {
				if l.starts_with(self.comment_char.as_str()) || l.is_empty() {
					None
				}
				else {
					Some(Line::new(l).map_err(|err| {
						IoError::FileRead {
							file: self.filepath.clone(),
							cause: FileReadErrorCause::from(err),
						}
					}))
				}
			})
			.collect();
		self.set_lines(lines?);
		Ok(())
	}

	/// Write the rebase file to disk.
	/// # Errors
	///
	/// Returns error if the file cannot be written.
	#[inline]
	pub fn write_file(&self) -> Result<(), IoError> {
		let mut file = File::create(&self.filepath).map_err(|err| {
			IoError::FileRead {
				file: self.filepath.clone(),
				cause: FileReadErrorCause::from(err),
			}
		})?;
		let file_contents = if self.is_noop {
			String::from("noop")
		}
		else {
			self.lines.iter().map(Line::to_text).collect::<Vec<String>>().join("\n")
		};
		writeln!(file, "{file_contents}").map_err(|err| {
			IoError::FileRead {
				file: self.filepath.clone(),
				cause: FileReadErrorCause::from(err),
			}
		})?;
		Ok(())
	}

	/// Set the selected line index.
	#[inline]
	pub fn set_selected_line_index(&mut self, selected_line_index: usize) {
		self.selected_line_index = if self.lines.is_empty() {
			0
		}
		else if selected_line_index >= self.lines.len() {
			self.lines.len() - 1
		}
		else {
			selected_line_index
		}
	}

	/// Swap a range of lines up.
	#[inline]
	pub fn swap_range_up(&mut self, start_index: usize, end_index: usize) -> bool {
		if end_index == 0 || start_index == 0 || self.lines.is_empty() {
			return false;
		}

		let max_index = self.lines.len() - 1;
		let end = if end_index > max_index { max_index } else { end_index };
		let start = if start_index > max_index {
			max_index
		}
		else {
			start_index
		};

		swap_range_up(&mut self.lines, start, end);
		self.history.record(HistoryItem::new_swap_up(start, end));
		true
	}

	/// Swap a range of lines down.
	#[inline]
	pub fn swap_range_down(&mut self, start_index: usize, end_index: usize) -> bool {
		let len = self.lines.len();
		let max_index = if len == 0 { 0 } else { len - 1 };

		if end_index == max_index || start_index == max_index {
			return false;
		}

		swap_range_down(&mut self.lines, start_index, end_index);
		self.history.record(HistoryItem::new_swap_down(start_index, end_index));
		true
	}

	/// Add a new line.
	#[inline]
	pub fn add_line(&mut self, index: usize, line: Line) {
		let i = if index > self.lines.len() {
			self.lines.len()
		}
		else {
			index
		};
		self.lines.insert(i, line);
		self.history.record(HistoryItem::new_add(i, i));
	}

	/// Remove a range of lines.
	#[inline]
	pub fn remove_lines(&mut self, start_index: usize, end_index: usize) {
		if self.lines.is_empty() {
			return;
		}

		let max_index = self.lines.len() - 1;
		let end = if end_index > max_index { max_index } else { end_index };
		let start = if start_index > max_index {
			max_index
		}
		else {
			start_index
		};

		let removed_lines = remove_range(&mut self.lines, start, end);
		self.history.record(HistoryItem::new_remove(start, end, removed_lines));
	}

	/// Update a range of lines.
	#[inline]
	pub fn update_range(&mut self, start_index: usize, end_index: usize, edit_context: &EditContext) {
		if self.lines.is_empty() {
			return;
		}

		let max_index = self.lines.len() - 1;
		let end = if end_index > max_index { max_index } else { end_index };
		let start = if start_index > max_index {
			max_index
		}
		else {
			start_index
		};

		let range = if end <= start { end..=start } else { start..=end };

		let mut lines = vec![];
		for index in range {
			let line = &mut self.lines[index];
			lines.push(line.clone());
			if let Some(action) = edit_context.get_action().as_ref() {
				line.set_action(*action);
			}

			if let Some(content) = edit_context.get_content().as_ref() {
				line.edit_content(content);
			}
		}
		self.history.record(HistoryItem::new_modify(start, end, lines));
	}

	/// Undo the last modification.
	#[inline]
	pub fn undo(&mut self) -> Option<(usize, usize)> {
		self.history.undo(&mut self.lines)
	}

	/// Redo the last undone modification.
	#[inline]
	pub fn redo(&mut self) -> Option<(usize, usize)> {
		self.history.redo(&mut self.lines)
	}

	/// Get the selected line.
	#[must_use]
	#[inline]
	pub fn get_selected_line(&self) -> Option<&Line> {
		self.lines.get(self.selected_line_index)
	}

	/// Get the index of the last line that can be selected.
	#[must_use]
	#[inline]
	pub fn get_max_selected_line_index(&self) -> usize {
		let len = self.lines.len();
		if len == 0 { 0 } else { len - 1 }
	}

	/// Get the selected line index
	#[must_use]
	#[inline]
	pub const fn get_selected_line_index(&self) -> usize {
		self.selected_line_index
	}

	/// Get the file path to the rebase file.
	#[must_use]
	#[inline]
	pub fn get_filepath(&self) -> &Path {
		self.filepath.as_path()
	}

	/// Get a line by index.
	#[must_use]
	#[inline]
	pub fn get_line(&self, index: usize) -> Option<&Line> {
		self.lines.get(index)
	}

	/// Get an owned copy of the lines.
	#[must_use]
	#[inline]
	pub fn get_lines_owned(&self) -> Vec<Line> {
		self.lines.clone()
	}

	/// Is the rebase file a noop.
	#[must_use]
	#[inline]
	pub const fn is_noop(&self) -> bool {
		self.is_noop
	}

	/// Get an iterator over the lines.
	#[inline]
	pub fn lines_iter(&self) -> Iter<'_, Line> {
		self.lines.iter()
	}

	/// Does the rebase file contain no lines.
	#[must_use]
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.lines.is_empty()
	}
}

#[cfg(test)]
mod tests {
	use claim::{assert_none, assert_some_eq};
	use tempfile::{Builder, NamedTempFile};
	use testutils::{assert_empty, assert_not_empty};

	use super::*;

	fn create_line(line: &str) -> Line {
		Line::new(line).unwrap()
	}

	fn create_and_load_todo_file(file_contents: &[&str]) -> (TodoFile, NamedTempFile) {
		let todo_file_path = Builder::new()
			.prefix("git-rebase-todo-scratch")
			.suffix("")
			.tempfile()
			.unwrap();
		write!(todo_file_path.as_file(), "{}", file_contents.join("\n")).unwrap();
		let mut todo_file = TodoFile::new(todo_file_path.path().to_str().unwrap(), 1, "#");
		todo_file.load_file().unwrap();
		(todo_file, todo_file_path)
	}

	macro_rules! assert_read_todo_file {
		($todo_file_path:expr, $($arg:expr),*) => {
			let expected = vec![$( $arg, )*];
			let content = read_to_string(Path::new($todo_file_path)).unwrap();
			pretty_assertions::assert_str_eq!(content, format!("{}\n", expected.join("\n")));
		};
	}

	macro_rules! assert_todo_lines {
		($todo_file_path:expr, $($arg:expr),*) => {
			let actual_lines = $todo_file_path.get_lines_owned();

			let expected = vec![$( create_line($arg), )*];
			pretty_assertions::assert_str_eq!(
				actual_lines.iter().map(Line::to_text).collect::<Vec<String>>().join("\n"),
				expected.iter().map(Line::to_text).collect::<Vec<String>>().join("\n")
			);
		};
	}

	#[test]
	fn load_file() {
		let (todo_file, _) = create_and_load_todo_file(&["pick aaa foobar"]);
		assert_todo_lines!(todo_file, "pick aaa foobar");
	}

	#[test]
	fn load_noop_file() {
		let (todo_file, _) = create_and_load_todo_file(&["noop"]);
		assert_empty!(todo_file);
		assert!(todo_file.is_noop());
	}

	#[test]
	fn load_ignore_comments() {
		let (todo_file, _) = create_and_load_todo_file(&["# pick aaa comment", "pick aaa foo", "# pick aaa comment"]);
		assert_todo_lines!(todo_file, "pick aaa foo");
	}

	#[test]
	fn load_ignore_newlines() {
		let (todo_file, _) = create_and_load_todo_file(&["", "pick aaa foobar", ""]);
		assert_todo_lines!(todo_file, "pick aaa foobar");
	}

	#[test]
	fn set_lines() {
		let (mut todo_file, _) = create_and_load_todo_file(&[]);
		todo_file.set_lines(vec![create_line("pick bbb comment")]);
		assert_todo_lines!(todo_file, "pick bbb comment");
	}

	#[test]
	fn set_lines_reset_history() {
		let (mut todo_file, _) = create_and_load_todo_file(&[]);
		todo_file.history.record(HistoryItem::new_add(1, 1));
		todo_file.set_lines(vec![create_line("pick bbb comment")]);
		assert_none!(todo_file.undo());
	}

	#[test]
	fn set_lines_reset_selected_index() {
		let (mut todo_file, _) = create_and_load_todo_file(&["pick a a", "pick b b", "pick c c"]);
		todo_file.selected_line_index = 2;
		todo_file.set_lines(vec![create_line("pick a a"), create_line("pick b b")]);
		assert_eq!(todo_file.selected_line_index, 1);
	}

	#[test]
	fn set_lines_reset_selected_index_empty_lis() {
		let (mut todo_file, _) = create_and_load_todo_file(&["pick a a", "pick b b", "pick c c"]);
		todo_file.selected_line_index = 2;
		todo_file.set_lines(vec![]);
		assert_eq!(todo_file.selected_line_index, 0);
	}

	#[test]
	fn write_file() {
		let (mut todo_file, _) = create_and_load_todo_file(&[]);
		todo_file.set_lines(vec![create_line("pick bbb comment")]);
		todo_file.write_file().unwrap();
		assert_todo_lines!(todo_file, "pick bbb comment");
	}

	#[test]
	fn write_file_noop() {
		let (mut todo_file, _) = create_and_load_todo_file(&[]);
		todo_file.set_lines(vec![create_line("noop")]);
		todo_file.write_file().unwrap();
		assert_read_todo_file!(todo_file.get_filepath(), "noop");
	}

	#[test]
	fn add_line_index_miss() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "drop bbb comment", "edit ccc comment"]);
		todo_file.add_line(100, create_line("fixup ddd comment"));
		assert_todo_lines!(
			todo_file,
			"pick aaa comment",
			"drop bbb comment",
			"edit ccc comment",
			"fixup ddd comment"
		);
	}

	#[test]
	fn add_line() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "drop bbb comment", "edit ccc comment"]);
		todo_file.add_line(1, create_line("fixup ddd comment"));
		assert_todo_lines!(
			todo_file,
			"pick aaa comment",
			"fixup ddd comment",
			"drop bbb comment",
			"edit ccc comment"
		);
	}

	#[test]
	fn add_line_record_history() {
		let (mut todo_file, _) = create_and_load_todo_file(&["pick aaa comment"]);
		todo_file.add_line(1, create_line("fixup ddd comment"));
		let _undo_result = todo_file.undo();
		assert_todo_lines!(todo_file, "pick aaa comment");
	}

	#[test]
	fn remove_lines_index_miss_start() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "drop bbb comment", "edit ccc comment"]);
		todo_file.remove_lines(100, 1);
		assert_todo_lines!(todo_file, "pick aaa comment");
	}

	#[test]
	fn remove_lines_index_miss_end() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "drop bbb comment", "edit ccc comment"]);
		todo_file.remove_lines(1, 100);
		assert_todo_lines!(todo_file, "pick aaa comment");
	}

	#[test]
	fn remove_lines_index_miss_start_and_end() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "drop bbb comment", "edit ccc comment"]);
		todo_file.remove_lines(100, 100);
		assert_todo_lines!(todo_file, "pick aaa comment", "drop bbb comment");
	}

	#[test]
	fn remove_lines() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "drop bbb comment", "edit ccc comment"]);
		todo_file.remove_lines(1, 1);
		assert_todo_lines!(todo_file, "pick aaa comment", "edit ccc comment");
	}

	#[test]
	fn remove_lines_empty_list() {
		let (mut todo_file, _) = create_and_load_todo_file(&[]);
		todo_file.remove_lines(1, 1);
	}

	#[test]
	fn remove_lines_record_history() {
		let (mut todo_file, _) = create_and_load_todo_file(&["pick aaa comment", "edit ccc comment"]);
		todo_file.remove_lines(1, 1);
		let _undo_result = todo_file.undo();
		assert_todo_lines!(todo_file, "pick aaa comment", "edit ccc comment");
	}

	#[test]
	fn update_range_full_set_action() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "drop bbb comment", "edit ccc comment"]);
		todo_file.update_range(0, 2, &EditContext::new().action(Action::Reword));
		assert_todo_lines!(
			todo_file,
			"reword aaa comment",
			"reword bbb comment",
			"reword ccc comment"
		);
	}

	#[test]
	fn update_range_full_set_content() {
		let (mut todo_file, _) = create_and_load_todo_file(&["exec foo", "exec bar", "exec foobar"]);
		todo_file.update_range(0, 2, &EditContext::new().content("echo"));
		assert_todo_lines!(todo_file, "exec echo", "exec echo", "exec echo");
	}

	#[test]
	fn update_range_edit_action() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "drop bbb comment", "edit ccc comment"]);
		todo_file.update_range(2, 0, &EditContext::new().action(Action::Reword));
		assert_todo_lines!(
			todo_file,
			"reword aaa comment",
			"reword bbb comment",
			"reword ccc comment"
		);
	}

	#[test]
	fn update_range_record_history() {
		let (mut todo_file, _) = create_and_load_todo_file(&["pick aaa comment"]);
		todo_file.update_range(0, 0, &EditContext::new().action(Action::Reword));
		let _undo_result = todo_file.undo();
		assert_todo_lines!(todo_file, "pick aaa comment");
	}

	#[test]
	fn update_range_empty_list() {
		let (mut todo_file, _) = create_and_load_todo_file(&[]);
		todo_file.update_range(0, 0, &EditContext::new().action(Action::Reword));
	}

	#[test]
	fn update_range_start_index_overflow() {
		let (mut todo_file, _) = create_and_load_todo_file(&["pick aaa comment", "pick bbb comment"]);
		todo_file.update_range(2, 0, &EditContext::new().action(Action::Reword));
		assert_todo_lines!(todo_file, "reword aaa comment", "reword bbb comment");
	}

	#[test]
	fn update_range_end_index_overflow() {
		let (mut todo_file, _) = create_and_load_todo_file(&["pick aaa comment", "pick bbb comment"]);
		todo_file.update_range(0, 2, &EditContext::new().action(Action::Reword));
		assert_todo_lines!(todo_file, "reword aaa comment", "reword bbb comment");
	}

	#[test]
	fn history_undo_redo() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "drop bbb comment", "edit ccc comment"]);
		todo_file.update_range(0, 0, &EditContext::new().action(Action::Drop));
		let _undo_result = todo_file.undo();
		assert_todo_lines!(todo_file, "pick aaa comment", "drop bbb comment", "edit ccc comment");
		let _ = todo_file.redo();
		assert_todo_lines!(todo_file, "drop aaa comment", "drop bbb comment", "edit ccc comment");
	}

	#[test]
	fn swap_up() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(todo_file.swap_range_up(1, 2));
		assert_todo_lines!(todo_file, "pick bbb comment", "pick ccc comment", "pick aaa comment");
	}

	#[test]
	fn swap_up_records_history() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		let _ = todo_file.swap_range_up(1, 2);
		let _undo_result = todo_file.undo();
		assert_todo_lines!(todo_file, "pick aaa comment", "pick bbb comment", "pick ccc comment");
	}

	#[test]
	fn swap_up_reverse_index() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(todo_file.swap_range_up(2, 1));
		assert_todo_lines!(todo_file, "pick bbb comment", "pick ccc comment", "pick aaa comment");
	}

	#[test]
	fn swap_up_single_line() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(todo_file.swap_range_up(1, 1));
		assert_todo_lines!(todo_file, "pick bbb comment", "pick aaa comment", "pick ccc comment");
	}

	#[test]
	fn swap_up_at_top_start_index() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(!todo_file.swap_range_up(0, 1));
		assert_todo_lines!(todo_file, "pick aaa comment", "pick bbb comment", "pick ccc comment");
	}

	#[test]
	fn swap_up_at_top_end_index() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(!todo_file.swap_range_up(1, 0));
		assert_todo_lines!(todo_file, "pick aaa comment", "pick bbb comment", "pick ccc comment");
	}

	#[test]
	fn swap_up_start_index_overflow() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(todo_file.swap_range_up(3, 1));
		assert_todo_lines!(todo_file, "pick bbb comment", "pick ccc comment", "pick aaa comment");
	}

	#[test]
	fn swap_up_end_index_overflow() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(todo_file.swap_range_up(3, 1));
		assert_todo_lines!(todo_file, "pick bbb comment", "pick ccc comment", "pick aaa comment");
	}

	#[test]
	fn swap_up_empty_list_index_out_of_bounds() {
		let (mut todo_file, _) = create_and_load_todo_file(&[]);
		assert!(!todo_file.swap_range_up(1, 1));
	}

	#[test]
	fn swap_down() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(todo_file.swap_range_down(0, 1));
		assert_todo_lines!(todo_file, "pick ccc comment", "pick aaa comment", "pick bbb comment");
	}

	#[test]
	fn swap_down_records_history() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		let _swap_result = todo_file.swap_range_down(0, 1);
		let _undo_result = todo_file.undo();
		assert_todo_lines!(todo_file, "pick aaa comment", "pick bbb comment", "pick ccc comment");
	}

	#[test]
	fn swap_down_reverse_index() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(todo_file.swap_range_down(1, 0));
		assert_todo_lines!(todo_file, "pick ccc comment", "pick aaa comment", "pick bbb comment");
	}

	#[test]
	fn swap_down_single_line() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(todo_file.swap_range_down(0, 0));
		assert_todo_lines!(todo_file, "pick bbb comment", "pick aaa comment", "pick ccc comment");
	}

	#[test]
	fn swap_down_at_bottom_end_index() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(!todo_file.swap_range_down(1, 2));
		assert_todo_lines!(todo_file, "pick aaa comment", "pick bbb comment", "pick ccc comment");
	}

	#[test]
	fn swap_down_at_bottom_start_index() {
		let (mut todo_file, _) =
			create_and_load_todo_file(&["pick aaa comment", "pick bbb comment", "pick ccc comment"]);
		assert!(!todo_file.swap_range_down(2, 1));
		assert_todo_lines!(todo_file, "pick aaa comment", "pick bbb comment", "pick ccc comment");
	}

	#[test]
	fn selected_line_index() {
		let (mut todo_file, _) = create_and_load_todo_file(&["exec foo", "exec bar", "exec foobar"]);
		todo_file.set_selected_line_index(1);
		assert_eq!(todo_file.get_selected_line_index(), 1);
	}

	#[test]
	fn selected_line_index_overflow() {
		let (mut todo_file, _) = create_and_load_todo_file(&["exec foo", "exec bar", "exec foobar"]);
		todo_file.set_selected_line_index(3);
		assert_eq!(todo_file.get_selected_line_index(), 2);
	}

	#[test]
	fn selected_line() {
		let (mut todo_file, _) = create_and_load_todo_file(&["exec foo", "exec bar", "exec foobar"]);
		todo_file.set_selected_line_index(0);
		assert_some_eq!(todo_file.get_selected_line(), &create_line("exec foo"));
	}

	#[test]
	fn selected_line_empty_list() {
		let (mut todo_file, _) = create_and_load_todo_file(&[]);
		todo_file.set_selected_line_index(0);
		assert_none!(todo_file.get_selected_line());
	}

	#[test]
	fn get_max_selected_line() {
		let (todo_file, _) = create_and_load_todo_file(&["exec foo", "exec bar", "exec foobar"]);
		assert_eq!(todo_file.get_max_selected_line_index(), 2);
	}

	#[test]
	fn get_max_selected_line_empty_list() {
		let (todo_file, _) = create_and_load_todo_file(&[]);
		assert_eq!(todo_file.get_max_selected_line_index(), 0);
	}

	#[test]
	fn get_line_miss_high() {
		let (todo_file, _) = create_and_load_todo_file(&["exec foo", "exec bar", "exec foobar"]);
		assert_none!(todo_file.get_line(4));
	}

	#[test]
	fn get_line_hit() {
		let (todo_file, _) = create_and_load_todo_file(&["exec foo", "exec bar", "exec foobar"]);
		assert_some_eq!(todo_file.get_line(1), &create_line("exec bar"));
	}

	#[test]
	fn get_file_path() {
		let (todo_file, filepath) = create_and_load_todo_file(&["exec foo", "exec bar", "exec foobar"]);
		assert_eq!(todo_file.get_filepath(), filepath.path());
	}

	#[test]
	fn iter() {
		let (todo_file, _) = create_and_load_todo_file(&["pick aaa comment"]);
		assert_some_eq!(todo_file.lines_iter().next(), &create_line("pick aaa comment"));
	}

	#[test]
	fn is_empty_true() {
		let (todo_file, _) = create_and_load_todo_file(&[]);
		assert_empty!(todo_file);
	}

	#[test]
	fn is_empty_false() {
		let (todo_file, _) = create_and_load_todo_file(&["pick aaa comment"]);
		assert_not_empty!(todo_file);
	}
}
