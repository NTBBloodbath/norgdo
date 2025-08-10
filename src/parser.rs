use crate::task::{Task, TodoItem, TodoState};
use color_eyre::Result;
use rust_norg::{
    DetachedModifierExtension, NestableDetachedModifier, NorgAST, NorgASTFlat, ParagraphSegment,
    ParagraphSegmentToken, TodoStatus, parse_tree,
};
use std::fs;
use std::path::Path;

pub struct NorgParser;

impl NorgParser {
    pub fn parse_task_file(file_path: &Path) -> Result<Task> {
        let content = fs::read_to_string(file_path)?;

        // Parse file
        let ast = parse_tree(&content)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to parse Norg file: {:?}", e))?;

        // Extract title from the first heading
        let title = Self::extract_title_from_ast(&ast);
        let mut task = Task::new(title, file_path.to_path_buf());

        // Extract description and todos from AST
        let (description, todos) = Self::extract_content_from_ast(&ast)?;
        task.description = description;
        task.todos = todos;

        Ok(task)
    }

    fn extract_title_from_ast(ast: &[NorgAST]) -> String {
        for node in ast {
            if let NorgAST::Heading { title, .. } = node {
                return Self::paragraph_to_string(title);
            }
        }
        "Untitled Task".to_string()
    }

    fn extract_content_from_ast(ast: &[NorgAST]) -> Result<(String, Vec<TodoItem>)> {
        let mut description_parts = Vec::new();
        let mut todos = Vec::new();
        let mut todo_id_counter = 0;
        let mut found_heading = false;
        let mut in_description = true;

        for node in ast {
            match node {
                NorgAST::Heading { title, content, .. } => {
                    found_heading = true;
                    in_description = true;

                    // Extract title text for description
                    let title_text = Self::paragraph_to_string(title);
                    description_parts.push(title_text);

                    // Process content within heading for todos
                    let (content_desc, content_todos) = Self::extract_content_from_ast(content)?;
                    if !content_desc.is_empty() {
                        description_parts.push(content_desc);
                    }
                    todos.extend(content_todos);
                }
                NorgAST::Paragraph(segments) => {
                    if found_heading && in_description {
                        let text = Self::paragraph_to_string(segments);
                        if !text.trim().is_empty() {
                            description_parts.push(text);
                        }
                    }
                }
                NorgAST::NestableDetachedModifier {
                    modifier_type: NestableDetachedModifier::UnorderedList,
                    level,
                    text,
                    content,
                    extensions,
                    ..
                } => {
                    in_description = false; // Stop collecting description once we hit todos

                    // Extract todo from the extensions and text
                    if let Some(todo) = Self::extract_todo_from_modifier(
                        text,
                        *level,
                        extensions,
                        &mut todo_id_counter,
                    )? {
                        todos.push(todo);
                    }

                    // Recursively process nested todos
                    let (_, nested_todos) = Self::extract_content_from_ast(content)?;
                    todos.extend(nested_todos);
                }
                _ => {
                    // Ignore other AST nodes
                }
            }
        }

        let description = description_parts.join("\n\n").trim().to_string();
        Ok((description, todos))
    }

    fn extract_todo_from_modifier(
        text: &Box<NorgASTFlat>,
        level: u16,
        extensions: &Vec<DetachedModifierExtension>,
        todo_id_counter: &mut usize,
    ) -> Result<Option<TodoItem>> {
        if let NorgASTFlat::Paragraph(segments) = text.as_ref() {
            let text_content = Self::paragraph_to_string(segments);

            // Look for Todo extension in the extensions array
            for extension in extensions {
                // Check if this extension indicates a todo item
                if let Some(state) = Self::extract_todo_state_from_extension(extension) {
                    *todo_id_counter += 1;

                    return Ok(Some(TodoItem {
                        id: format!("todo_{}", todo_id_counter),
                        text: text_content,
                        state,
                        level: level as usize,
                        line_number: 0, // We don't have line numbers from AST
                    }));
                }
            }
        }
        Ok(None)
    }

    fn extract_todo_state_from_extension(
        extension: &DetachedModifierExtension,
    ) -> Option<TodoState> {
        // For now, let's match based on debug output patterns we saw
        match extension {
            DetachedModifierExtension::Todo(todo_status) => {
                // TODO: handle recurring Option<String> for dates
                match todo_status {
                    TodoStatus::Done => Some(TodoState::Done),
                    TodoStatus::Pending => Some(TodoState::Pending),
                    TodoStatus::Urgent => Some(TodoState::Urgent),
                    TodoStatus::Undone => Some(TodoState::Undone),
                    TodoStatus::Paused => Some(TodoState::OnHold),
                    TodoStatus::Canceled => Some(TodoState::Cancelled),
                    TodoStatus::NeedsClarification => Some(TodoState::Uncertain),
                    TodoStatus::Recurring(_) => Some(TodoState::Recurring),
                }
            }
            _ => {
                // Ignore other extensions
                None
            }
        }
    }

    fn paragraph_to_string(segments: &[ParagraphSegment]) -> String {
        let mut result = String::new();

        for segment in segments {
            match segment {
                ParagraphSegment::Token(token) => match token {
                    ParagraphSegmentToken::Text(text) => result.push_str(text),
                    ParagraphSegmentToken::Whitespace => result.push(' '),
                    ParagraphSegmentToken::Special(c) | ParagraphSegmentToken::Escape(c) => {
                        result.push(*c);
                    }
                },
                ParagraphSegment::AttachedModifier { content, .. } => {
                    // For simplicity, just extract the text content from modifiers
                    result.push_str(&Self::paragraph_to_string(content));
                }
                ParagraphSegment::InlineVerbatim(tokens) => {
                    for token in tokens {
                        match token {
                            ParagraphSegmentToken::Text(text) => result.push_str(text),
                            ParagraphSegmentToken::Whitespace => result.push(' '),
                            ParagraphSegmentToken::Special(c)
                            | ParagraphSegmentToken::Escape(c) => {
                                result.push(*c);
                            }
                        }
                    }
                }
                ParagraphSegment::Link { description, .. } => {
                    if let Some(desc) = description {
                        result.push_str(&Self::paragraph_to_string(desc));
                    }
                }
                _ => {
                    // For unhandled segment types, continue without adding content
                }
            }
        }

        result
    }

    pub fn write_task_file(task: &Task) -> Result<()> {
        let mut content = String::new();

        // Write title
        content.push_str(&format!("* {}\n\n", task.title));

        // Write description
        if !task.description.is_empty() {
            content.push_str(&task.description);
            content.push_str("\n\n");
        }

        // Write todos
        for todo in &task.todos {
            let list_prefix = "-".repeat(todo.level.max(1)); // At least one hyphen
            content.push_str(&format!(
                "{} ({}) {}\n",
                list_prefix,
                todo.state.to_norg_char(),
                todo.text
            ));
        }

        fs::write(&task.file_path, content)?;
        Ok(())
    }
}
