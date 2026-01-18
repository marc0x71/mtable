#![allow(unused)]

use crate::error::{LexerError, TableError};
use std::{collections::HashSet, fmt::Debug, marker::PhantomData};

#[derive(Debug)]
struct Node<T> {
    children: Vec<Option<usize>>,
    value: Option<T>,
}
impl<T: Clone> Node<T> {
    fn new(capacity: usize) -> Self {
        Self {
            children: vec![None; capacity],
            value: None,
        }
    }

    fn set_children(&mut self, index: usize, child: usize) -> Result<(), TableError<T>> {
        if let Some(c) = self.children.get_mut(index) {
            if let Some(existing) = *c
                && existing != child
            {
                return Err(TableError::AmbiguousPattern(
                    char::from_u32(child as u32).unwrap_or_default(),
                ));
            }
            *c = Some(child);
        }
        Ok(())
    }

    fn get_children(&self, index: usize) -> Option<&usize> {
        let child = self.children.get(index)?;
        child.as_ref()
    }

    fn set_value(&mut self, value: T) -> Result<(), TableError<T>> {
        if self.value.is_some() {
            return Err(TableError::<T>::ValueAlreadyDefined {
                current: self.value.as_ref().unwrap().clone(),
                requested: value.clone(),
            });
        }
        self.value = Some(value);
        Ok(())
    }

    fn get_value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    fn has_value(&self) -> bool {
        self.value.is_some()
    }
}

#[derive(Debug)]
pub struct Table<T> {
    alphabet: String,
    nodes: Vec<Node<T>>,
}

impl<T: Debug + Clone> Table<T> {
    pub fn new(alphabet: String) -> Self {
        let capacity = alphabet.len();
        Self {
            alphabet,
            nodes: vec![Node::new(capacity)],
        }
    }

    fn calculate_position(&self, ch: u8) -> Result<usize, TableError<T>> {
        self.alphabet
            .find(ch as char)
            .ok_or(TableError::<T>::InvalidInput(ch as char))
    }

    fn append_node(&mut self, current: usize, child: usize) -> Result<usize, TableError<T>> {
        match self.nodes[current].get_children(child) {
            Some(next) => Ok(*next),
            None => {
                let new_node = Node::<T>::new(self.alphabet.len());
                self.nodes.push(new_node);
                let new_child = self.nodes.len() - 1;
                self.nodes[current].set_children(child, new_child);
                Ok(new_child)
            }
        }
    }

    fn add_from_range(
        &mut self,
        range: &[usize],
        currents: &[usize],
    ) -> Result<Vec<usize>, TableError<T>> {
        let mut new_currents = vec![];
        for current in currents {
            let created: Result<Vec<usize>, TableError<T>> = range
                .iter()
                .map(|pos| self.append_node(*current, *pos))
                .collect();
            new_currents.extend(created?);
        }
        Ok(new_currents.to_vec())
    }

    pub fn add(&mut self, s: &str, value: T) -> Result<(), TableError<T>> {
        if !s.is_ascii() {
            return Err(TableError::InvalidString(s.to_string()));
        }
        let mut currents = vec![0];
        let mut iter = s.bytes().peekable();
        while let Some(ch) = iter.next() {
            let mut range = Vec::with_capacity(self.alphabet.len());
            match ch {
                b'[' => {
                    while let Some(next) = iter.next_if(|n| *n != b']') {
                        let pos = self.calculate_position(next)?;
                        if !range.contains(&pos) {
                            range.push(pos);
                        }
                    }
                    if Some(b']') != iter.next() || range.is_empty() {
                        return Err(TableError::InvalidRange);
                    }
                    currents = self.add_from_range(&range, &currents)?;
                }
                _ => {
                    range = vec![self.calculate_position(ch)?];
                    currents = self.add_from_range(&range, &currents)?;
                }
            };
            if let Some(b'+') = iter.peek() {
                let _ = iter.next();
                for current in &currents {
                    for pos in &range {
                        self.nodes[*current].set_children(*pos, *current);
                    }
                }
            }
        }
        // remove duplicated
        let unique_currents: HashSet<_> = currents.into_iter().collect();
        for current in unique_currents {
            self.nodes[current].set_value(value.clone())?;
        }
        Ok(())
    }

    pub fn get(&self, s: &str) -> Result<Option<&T>, TableError<T>> {
        if !s.is_ascii() {
            return Err(TableError::InvalidString(s.to_string()));
        }
        let mut current: usize = 0;
        for ch in s.bytes() {
            let pos = self
                .alphabet
                .find(ch as char)
                .ok_or(TableError::<T>::InvalidInput(ch as char))?;

            if let Some(next) = self.nodes[current].get_children(pos) {
                current = *next;
            } else {
                return Ok(None);
            }
        }
        Ok(self.nodes[current].get_value())
    }

    pub fn lexer<'a>(&'a self, s: &'a str) -> Result<TableIterator<'a, T>, LexerError> {
        if !s.is_ascii() {
            return Err(LexerError::InvalidString(s.to_string()));
        }
        Ok(TableIterator {
            table: self,
            input: s,
            index: 0,
            _phantom: PhantomData,
        })
    }
}

pub struct TableIterator<'a, T> {
    table: &'a Table<T>,
    input: &'a str,
    index: usize,
    _phantom: PhantomData<T>,
}

impl<'a, T: Clone> Iterator for TableIterator<'a, T> {
    type Item = Result<(&'a T, &'a str), LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.input.len() {
            return None;
        }
        let mut node_id = 0;
        let mut progress = self.index;
        let mut last_match = vec![];
        loop {
            if progress >= self.input.len() {
                return match last_match.pop() {
                    Some((last_index, value)) => {
                        let content = &self.input[self.index..last_index + 1];
                        self.index = last_index + 1;
                        Some(Ok((value, content)))
                    }
                    None => Some(Err(LexerError::UnexpectedEnd {
                        position: self.index,
                    })),
                };
            }
            let ch = self.input.as_bytes()[progress];
            let pos = match self.table.alphabet.find(ch as char) {
                Some(p) => p,
                None => {
                    return Some(Err(LexerError::UnknownChar {
                        char: ch as char,
                        position: progress,
                    }));
                }
            };
            if let Some(next) = self.table.nodes[node_id].get_children(pos) {
                if self.table.nodes[*next].has_value() {
                    last_match.push((progress, self.table.nodes[*next].get_value().unwrap()));
                }
                progress += 1;
                node_id = *next;
            } else {
                return match last_match.pop() {
                    Some((last_progress, value)) => {
                        let content = &self.input[self.index..last_progress + 1];
                        self.index = last_progress + 1;
                        Some(Ok((value, content)))
                    }
                    None => Some(Err(LexerError::UnexpectedEnd {
                        position: self.index,
                    })),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha() -> Table<&'static str> {
        Table::new("abcdefghijklmnopqrstuvwxyz".to_string())
    }

    fn digits() -> Table<i32> {
        Table::new("0123456789".to_string())
    }

    fn alphanum() -> Table<&'static str> {
        Table::new("abcdefghijklmnopqrstuvwxyz0123456789".to_string())
    }

    // ========================================================================
    // LITERAL STRINGS
    // ========================================================================

    #[test]
    fn literal_simple() {
        let mut t = alpha();
        t.add("hello", "greeting").unwrap();
        assert_eq!(t.get("hello").unwrap(), Some(&"greeting"));
    }

    #[test]
    fn literal_single_char() {
        let mut t = alpha();
        t.add("a", "letter").unwrap();
        assert_eq!(t.get("a").unwrap(), Some(&"letter"));
        assert_eq!(t.get("b").unwrap(), None);
    }

    #[test]
    fn literal_empty_string() {
        let mut t = alpha();
        t.add("", "empty").unwrap();
        assert_eq!(t.get("").unwrap(), Some(&"empty"));
    }

    #[test]
    fn literal_multiple_patterns() {
        let mut t = alpha();
        t.add("cat", "animal").unwrap();
        t.add("car", "vehicle").unwrap();
        t.add("card", "object").unwrap();

        assert_eq!(t.get("cat").unwrap(), Some(&"animal"));
        assert_eq!(t.get("car").unwrap(), Some(&"vehicle"));
        assert_eq!(t.get("card").unwrap(), Some(&"object"));
        assert_eq!(t.get("ca").unwrap(), None);
    }

    #[test]
    fn literal_shared_prefix() {
        let mut t = alpha();
        t.add("test", "v1").unwrap();
        t.add("testing", "v2").unwrap();

        assert_eq!(t.get("test").unwrap(), Some(&"v1"));
        assert_eq!(t.get("testing").unwrap(), Some(&"v2"));
        assert_eq!(t.get("testi").unwrap(), None);
    }

    #[test]
    fn literal_prefix_not_matched() {
        let mut t = alpha();
        t.add("hello", "greeting").unwrap();
        assert_eq!(t.get("hel").unwrap(), None);
        assert_eq!(t.get("helloo").unwrap(), None);
    }

    // ========================================================================
    // CHARACTER CLASSES [abc]
    // ========================================================================

    #[test]
    fn class_simple() {
        let mut t = alpha();
        t.add("[abc]", "first_three").unwrap();

        assert_eq!(t.get("a").unwrap(), Some(&"first_three"));
        assert_eq!(t.get("b").unwrap(), Some(&"first_three"));
        assert_eq!(t.get("c").unwrap(), Some(&"first_three"));
        assert_eq!(t.get("d").unwrap(), None);
    }

    #[test]
    fn class_single_element() {
        let mut t = alpha();
        t.add("[a]", "just_a").unwrap();
        assert_eq!(t.get("a").unwrap(), Some(&"just_a"));
        assert_eq!(t.get("b").unwrap(), None);
    }

    #[test]
    fn class_in_middle() {
        let mut t = alpha();
        t.add("c[aou]t", "words").unwrap();

        assert_eq!(t.get("cat").unwrap(), Some(&"words"));
        assert_eq!(t.get("cot").unwrap(), Some(&"words"));
        assert_eq!(t.get("cut").unwrap(), Some(&"words"));
        assert_eq!(t.get("cet").unwrap(), None);
        assert_eq!(t.get("cit").unwrap(), None);
    }

    #[test]
    fn class_multiple() {
        let mut t = alpha();
        t.add("[ab][xy]", "combo").unwrap();

        assert_eq!(t.get("ax").unwrap(), Some(&"combo"));
        assert_eq!(t.get("ay").unwrap(), Some(&"combo"));
        assert_eq!(t.get("bx").unwrap(), Some(&"combo"));
        assert_eq!(t.get("by").unwrap(), Some(&"combo"));
        assert_eq!(t.get("cx").unwrap(), None);
        assert_eq!(t.get("az").unwrap(), None);
    }

    #[test]
    fn class_consecutive() {
        let mut t = alpha();
        t.add("[a][b][c]", "abc").unwrap();
        assert_eq!(t.get("abc").unwrap(), Some(&"abc"));
        assert_eq!(t.get("aaa").unwrap(), None);
    }

    #[test]
    fn class_with_duplicates() {
        let mut t = alpha();
        // [aaa] should be treated as [a]
        t.add("[aaa]", "triple").unwrap();
        assert_eq!(t.get("a").unwrap(), Some(&"triple"));
        assert_eq!(t.get("aa").unwrap(), None);
    }

    #[test]
    fn class_preserves_order() {
        let mut t = alpha();
        // duplicates should be ignored, keeping first occurrence
        t.add("[abab]", "val").unwrap();
        assert_eq!(t.get("a").unwrap(), Some(&"val"));
        assert_eq!(t.get("b").unwrap(), Some(&"val"));
    }

    // ========================================================================
    // PLUS OPERATOR (+)
    // ========================================================================

    #[test]
    fn plus_single_char() {
        let mut t = alpha();
        t.add("a+", "as").unwrap();

        assert_eq!(t.get("a").unwrap(), Some(&"as"));
        assert_eq!(t.get("aa").unwrap(), Some(&"as"));
        assert_eq!(t.get("aaa").unwrap(), Some(&"as"));
        assert_eq!(t.get("").unwrap(), None);
    }

    #[test]
    fn plus_with_prefix() {
        let mut t = alpha();
        t.add("ba+", "b_then_as").unwrap();

        assert_eq!(t.get("ba").unwrap(), Some(&"b_then_as"));
        assert_eq!(t.get("baa").unwrap(), Some(&"b_then_as"));
        assert_eq!(t.get("baaa").unwrap(), Some(&"b_then_as"));
        assert_eq!(t.get("b").unwrap(), None);
    }

    #[test]
    fn plus_with_suffix() {
        let mut t = alpha();
        t.add("a+b", "as_then_b").unwrap();

        assert_eq!(t.get("ab").unwrap(), Some(&"as_then_b"));
        assert_eq!(t.get("aab").unwrap(), Some(&"as_then_b"));
        assert_eq!(t.get("aaab").unwrap(), Some(&"as_then_b"));
        assert_eq!(t.get("a").unwrap(), None);
        assert_eq!(t.get("b").unwrap(), None);
    }

    #[test]
    fn plus_with_class() {
        let mut t = alpha();
        t.add("[ab]+", "ab_repeated").unwrap();

        assert_eq!(t.get("a").unwrap(), Some(&"ab_repeated"));
        assert_eq!(t.get("b").unwrap(), Some(&"ab_repeated"));
        assert_eq!(t.get("ab").unwrap(), Some(&"ab_repeated"));
        assert_eq!(t.get("ba").unwrap(), Some(&"ab_repeated"));
        assert_eq!(t.get("aabb").unwrap(), Some(&"ab_repeated"));
        assert_eq!(t.get("abab").unwrap(), Some(&"ab_repeated"));
        assert_eq!(t.get("c").unwrap(), None);
        assert_eq!(t.get("").unwrap(), None);
    }

    #[test]
    fn plus_multiple() {
        let mut t = alpha();
        t.add("a+b+", "as_then_bs").unwrap();

        assert_eq!(t.get("ab").unwrap(), Some(&"as_then_bs"));
        assert_eq!(t.get("aab").unwrap(), Some(&"as_then_bs"));
        assert_eq!(t.get("abb").unwrap(), Some(&"as_then_bs"));
        assert_eq!(t.get("aabb").unwrap(), Some(&"as_then_bs"));
        assert_eq!(t.get("aaabbb").unwrap(), Some(&"as_then_bs"));
    }

    #[test]
    fn plus_in_middle() {
        let mut t = alpha();
        t.add("a+bc", "pattern").unwrap();

        assert_eq!(t.get("abc").unwrap(), Some(&"pattern"));
        assert_eq!(t.get("aabc").unwrap(), Some(&"pattern"));
        assert_eq!(t.get("aaabc").unwrap(), Some(&"pattern"));
        assert_eq!(t.get("bc").unwrap(), None);
    }

    #[test]
    fn plus_long_match() {
        let mut t = alpha();
        t.add("a+", "many").unwrap();

        let long = "a".repeat(10000);
        assert_eq!(t.get(&long).unwrap(), Some(&"many"));
    }

    // ========================================================================
    // AMBIGUITY / CONVERGENCE (paths converging to same node)
    // ========================================================================

    #[test]
    fn ambiguity_plus_followed_by_class() {
        let mut t: Table<&str> = Table::new("ab".to_string());
        // [ab]+[ab] creates duplicate currents due to self-loops
        let result = t.add("[ab]+[ab]", "value");
        assert!(result.is_ok());

        // Semantically equivalent to [ab]+
        assert_eq!(t.get("a").unwrap(), Some(&"value"));
        assert_eq!(t.get("b").unwrap(), Some(&"value"));
        assert_eq!(t.get("ab").unwrap(), Some(&"value"));
        assert_eq!(t.get("ba").unwrap(), Some(&"value"));
    }

    #[test]
    fn ambiguity_plus_then_same_char() {
        let mut t: Table<&str> = Table::new("a".to_string());
        // a+a: the second 'a' is absorbed by the self-loop
        // so it's semantically equivalent to a+
        t.add("a+a", "value").unwrap();

        assert_eq!(t.get("a").unwrap(), Some(&"value"));
        assert_eq!(t.get("aa").unwrap(), Some(&"value"));
        assert_eq!(t.get("aaa").unwrap(), Some(&"value"));
    }

    #[test]
    fn ambiguity_class_plus_then_subset() {
        let mut t: Table<&str> = Table::new("ab".to_string());
        // a+[ab]:
        // - 'a' after + loops back, so a+a = a+
        // - 'b' after + creates new final state
        // Result: matches "a", "aa", "ab", "aab", etc.
        t.add("a+[ab]", "value").unwrap();

        assert_eq!(t.get("a").unwrap(), Some(&"value")); // a+ part
        assert_eq!(t.get("aa").unwrap(), Some(&"value")); // a+ looping
        assert_eq!(t.get("ab").unwrap(), Some(&"value")); // a then b
        assert_eq!(t.get("aaa").unwrap(), Some(&"value"));
        assert_eq!(t.get("aab").unwrap(), Some(&"value"));
    }

    // ========================================================================
    // ERROR HANDLING
    // ========================================================================

    #[test]
    fn error_invalid_char_not_in_alphabet() {
        let mut t = alpha();
        let result = t.add("hello1", "with_digit");
        assert!(matches!(result, Err(TableError::InvalidInput('1'))));
    }

    #[test]
    fn error_invalid_char_in_class() {
        let mut t = alpha();
        let result = t.add("[abc1]", "invalid");
        assert!(matches!(result, Err(TableError::InvalidInput('1'))));
    }

    #[test]
    fn error_unclosed_bracket() {
        let mut t = alpha();
        let result = t.add("[abc", "unclosed");
        assert!(matches!(result, Err(TableError::InvalidRange)));
    }

    #[test]
    fn error_empty_class() {
        let mut t = alpha();
        let result = t.add("[]", "empty");
        assert!(matches!(result, Err(TableError::InvalidRange)));
    }

    #[test]
    fn error_empty_class_with_plus() {
        let mut t = alpha();
        let result = t.add("[]+", "empty_plus");
        assert!(matches!(result, Err(TableError::InvalidRange)));
    }

    #[test]
    fn error_non_ascii_add() {
        let mut t = alpha();
        let result = t.add("héllo", "accented");
        assert!(matches!(result, Err(TableError::InvalidString(_))));
    }

    #[test]
    fn error_non_ascii_get() {
        let t = alpha();
        let result = t.get("héllo");
        assert!(matches!(result, Err(TableError::InvalidString(_))));
    }

    #[test]
    fn error_emoji() {
        let mut t = alpha();
        let result = t.add("hello😀", "emoji");
        assert!(matches!(result, Err(TableError::InvalidString(_))));
    }

    #[test]
    fn error_duplicate_value() {
        let mut t = alpha();
        t.add("hello", "first").unwrap();
        let result = t.add("hello", "second");
        assert!(matches!(
            result,
            Err(TableError::ValueAlreadyDefined { .. })
        ));
    }

    #[test]
    fn error_duplicate_via_class_overlap() {
        let mut t = alpha();
        t.add("a", "literal").unwrap();
        let result = t.add("[abc]", "class");
        assert!(matches!(
            result,
            Err(TableError::ValueAlreadyDefined { .. })
        ));
    }

    #[test]
    fn error_duplicate_via_plus_overlap() {
        let mut t = alpha();
        t.add("[ab]+", "pattern1").unwrap();
        let result = t.add("a", "pattern2");
        assert!(matches!(
            result,
            Err(TableError::ValueAlreadyDefined { .. })
        ));
    }

    #[test]
    fn error_get_char_not_in_alphabet() {
        let mut t = alpha();
        t.add("hello", "greeting").unwrap();
        let result = t.get("hello!");
        assert!(matches!(result, Err(TableError::InvalidInput('!'))));
    }

    // ========================================================================
    // EDGE CASES
    // ========================================================================

    #[test]
    fn edge_star_is_literal() {
        let mut t: Table<&str> = Table::new("a*".to_string());
        // * is not an operator, just a literal
        t.add("a*", "star").unwrap();
        assert_eq!(t.get("a*").unwrap(), Some(&"star"));
    }

    #[test]
    fn edge_plus_at_start_is_literal() {
        let mut t: Table<&str> = Table::new("+a".to_string());
        t.add("+a", "plus").unwrap();
        assert_eq!(t.get("+a").unwrap(), Some(&"plus"));
    }

    #[test]
    fn edge_bracket_in_alphabet() {
        let mut t: Table<&str> = Table::new("a[]".to_string());
        // When [ is in alphabet but used as operator, it's still parsed as operator
        let result = t.add("[a]", "test");
        assert!(result.is_ok());
        assert_eq!(t.get("a").unwrap(), Some(&"test"));
    }

    #[test]
    fn edge_nested_brackets() {
        let mut t = alpha();
        // [[ab]] - inner [ is looked up in alphabet, not found
        let result = t.add("[[ab]]", "nested");
        assert!(matches!(result, Err(TableError::InvalidInput('['))));
    }

    #[test]
    fn edge_empty_alphabet() {
        let mut t: Table<&str> = Table::new("".to_string());
        t.add("", "empty").unwrap();
        assert_eq!(t.get("").unwrap(), Some(&"empty"));

        let result = t.add("a", "should_fail");
        assert!(matches!(result, Err(TableError::InvalidInput('a'))));
    }

    #[test]
    fn edge_single_char_alphabet() {
        let mut t: Table<i32> = Table::new("a".to_string());
        t.add("a+", 42).unwrap();

        assert_eq!(t.get("a").unwrap(), Some(&42));
        assert_eq!(t.get("aaa").unwrap(), Some(&42));

        let result = t.get("b");
        assert!(matches!(result, Err(TableError::InvalidInput('b'))));
    }

    #[test]
    fn edge_very_long_pattern() {
        let mut t = alpha();
        let long_pattern = "a".repeat(1000);
        t.add(&long_pattern, "long").unwrap();

        assert_eq!(t.get(&long_pattern).unwrap(), Some(&"long"));
        assert_eq!(t.get(&"a".repeat(999)).unwrap(), None);
    }

    #[test]
    fn edge_special_chars_in_alphabet() {
        let mut t: Table<&str> = Table::new("abc.+-_@#".to_string());
        t.add("a.b", "dot").unwrap();
        t.add("a-b", "dash").unwrap();
        t.add("a@b", "at").unwrap();

        assert_eq!(t.get("a.b").unwrap(), Some(&"dot"));
        assert_eq!(t.get("a-b").unwrap(), Some(&"dash"));
        assert_eq!(t.get("a@b").unwrap(), Some(&"at"));
    }

    // ========================================================================
    // GENERIC TYPES
    // ========================================================================

    #[test]
    fn generic_integer() {
        let mut t: Table<i32> = Table::new("abc".to_string());
        t.add("a", 1).unwrap();
        t.add("b", 2).unwrap();
        t.add("c", 3).unwrap();

        assert_eq!(t.get("a").unwrap(), Some(&1));
        assert_eq!(t.get("b").unwrap(), Some(&2));
        assert_eq!(t.get("c").unwrap(), Some(&3));
    }

    #[test]
    fn generic_struct() {
        #[derive(Debug, Clone, PartialEq)]
        struct Token {
            kind: String,
            priority: u8,
        }

        let mut t: Table<Token> = Table::new("+-*/".to_string());
        t.add(
            "+",
            Token {
                kind: "add".into(),
                priority: 1,
            },
        )
        .unwrap();
        t.add(
            "*",
            Token {
                kind: "mul".into(),
                priority: 2,
            },
        )
        .unwrap();

        let plus = t.get("+").unwrap().unwrap();
        assert_eq!(plus.kind, "add");
        assert_eq!(plus.priority, 1);

        let mul = t.get("*").unwrap().unwrap();
        assert_eq!(mul.kind, "mul");
        assert_eq!(mul.priority, 2);
    }

    #[test]
    fn generic_with_digits() {
        let mut t = digits();
        t.add("[0123456789]+", 42).unwrap();

        assert_eq!(t.get("0").unwrap(), Some(&42));
        assert_eq!(t.get("123").unwrap(), Some(&42));
        assert_eq!(t.get("9876543210").unwrap(), Some(&42));
    }

    // ========================================================================
    // COMPLEX PATTERNS
    // ========================================================================

    #[test]
    fn complex_identifier_like() {
        let mut t = alphanum();
        // First char must be letter, rest can be letter or digit
        t.add(
            "[abcdefghijklmnopqrstuvwxyz][abcdefghijklmnopqrstuvwxyz0123456789]+",
            "id",
        )
        .unwrap();

        assert_eq!(t.get("ab").unwrap(), Some(&"id"));
        assert_eq!(t.get("a1").unwrap(), Some(&"id"));
        assert_eq!(t.get("var123").unwrap(), Some(&"id"));
        assert_eq!(t.get("x9y8z7").unwrap(), Some(&"id"));

        // Single letter doesn't match (needs at least 2 chars due to +)
        assert_eq!(t.get("a").unwrap(), None);
        // Starts with digit - no match
        assert_eq!(t.get("1abc").unwrap(), None);
    }

    #[test]
    fn complex_multiple_non_overlapping() {
        let mut t = alpha();
        t.add("get", "GET").unwrap();
        t.add("put", "PUT").unwrap();
        t.add("post", "POST").unwrap();
        t.add("delete", "DELETE").unwrap();

        assert_eq!(t.get("get").unwrap(), Some(&"GET"));
        assert_eq!(t.get("put").unwrap(), Some(&"PUT"));
        assert_eq!(t.get("post").unwrap(), Some(&"POST"));
        assert_eq!(t.get("delete").unwrap(), Some(&"DELETE"));
        assert_eq!(t.get("patch").unwrap(), None);
    }

    #[test]
    fn complex_class_then_plus_then_literal() {
        let mut t = alpha();
        t.add("[abc]+x", "pattern").unwrap();

        assert_eq!(t.get("ax").unwrap(), Some(&"pattern"));
        assert_eq!(t.get("bx").unwrap(), Some(&"pattern"));
        assert_eq!(t.get("abcx").unwrap(), Some(&"pattern"));
        assert_eq!(t.get("aaabbbcccx").unwrap(), Some(&"pattern"));
        assert_eq!(t.get("x").unwrap(), None);
        assert_eq!(t.get("dx").unwrap(), None);
    }

    #[test]
    fn complex_alternating_classes() {
        let mut t: Table<&str> = Table::new("ab".to_string());
        t.add("[ab][ab][ab]", "three").unwrap();

        assert_eq!(t.get("aaa").unwrap(), Some(&"three"));
        assert_eq!(t.get("bbb").unwrap(), Some(&"three"));
        assert_eq!(t.get("aba").unwrap(), Some(&"three"));
        assert_eq!(t.get("bab").unwrap(), Some(&"three"));
        assert_eq!(t.get("ab").unwrap(), None);
        assert_eq!(t.get("abab").unwrap(), None);
    }

    // ========================================================================
    // NODE REUSE / STRUCTURE
    // ========================================================================

    #[test]
    fn structure_shared_prefix_reuses_nodes() {
        let mut t = alpha();
        t.add("abc", "v1").unwrap();
        let nodes_after_first = t.nodes.len();

        t.add("abd", "v2").unwrap();
        let nodes_after_second = t.nodes.len();

        // "abd" should reuse nodes for "ab" and add only one for 'd'
        assert_eq!(nodes_after_second, nodes_after_first + 1);
    }

    #[test]
    fn structure_plus_creates_self_loop() {
        let mut t = alpha();
        t.add("a+", "loop").unwrap();

        let a_pos = t.alphabet.find('a').unwrap();
        let first_node = t.nodes[0].get_children(a_pos).unwrap();
        let loop_target = t.nodes[*first_node].get_children(a_pos).unwrap();

        // Node should point to itself
        assert_eq!(first_node, loop_target);
    }

    // ========================================================================
    // BASIC TOKENIZATION
    // ========================================================================

    #[test]
    fn lexer_basic_expression() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Number,
            Add,
            Sub,
            Mul,
            Div,
        }

        let mut t = Table::new("0123456789+-*/".to_string());
        t.add("[0123456789]+", Kind::Number).unwrap();
        t.add("+", Kind::Add).unwrap();
        t.add("-", Kind::Sub).unwrap();
        t.add("*", Kind::Mul).unwrap();
        t.add("/", Kind::Div).unwrap();

        let iter = t.lexer("1+2*3").unwrap();
        let tokens: Vec<_> = iter.collect::<Result<_, _>>().unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], (&Kind::Number, "1"));
        assert_eq!(tokens[1], (&Kind::Add, "+"));
        assert_eq!(tokens[2], (&Kind::Number, "2"));
        assert_eq!(tokens[3], (&Kind::Mul, "*"));
        assert_eq!(tokens[4], (&Kind::Number, "3"));
    }

    #[test]
    fn lexer_single_token() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Word,
        }

        let mut t = Table::new("abcdefghijklmnopqrstuvwxyz".to_string());
        t.add("[abcdefghijklmnopqrstuvwxyz]+", Kind::Word).unwrap();

        let tokens: Vec<_> = t.lexer("hello").unwrap().collect::<Result<_, _>>().unwrap();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], (&Kind::Word, "hello"));
    }

    #[test]
    fn lexer_empty_input() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Number,
        }

        let mut t = Table::new("0123456789".to_string());
        t.add("[0123456789]+", Kind::Number).unwrap();

        let tokens: Vec<_> = t.lexer("").unwrap().collect::<Result<_, _>>().unwrap();

        assert!(tokens.is_empty());
    }

    #[test]
    fn lexer_manual_iteration() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            A,
            B,
        }

        let mut t = Table::new("ab".to_string());
        t.add("a", Kind::A).unwrap();
        t.add("b", Kind::B).unwrap();

        let mut iter = t.lexer("aba").unwrap();

        let (kind, content) = iter.next().unwrap().unwrap();
        assert_eq!(kind, &Kind::A);
        assert_eq!(content, "a");

        let (kind, content) = iter.next().unwrap().unwrap();
        assert_eq!(kind, &Kind::B);
        assert_eq!(content, "b");

        let (kind, content) = iter.next().unwrap().unwrap();
        assert_eq!(kind, &Kind::A);
        assert_eq!(content, "a");

        assert!(iter.next().is_none());
    }

    // ========================================================================
    // LONGEST MATCH (MAXIMAL MUNCH)
    // ========================================================================

    #[test]
    fn lexer_longest_match_numbers() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Number,
            Add,
        }

        let mut t = Table::new("0123456789+".to_string());
        t.add("[0123456789]+", Kind::Number).unwrap();
        t.add("+", Kind::Add).unwrap();

        let tokens: Vec<_> = t
            .lexer("123+456")
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], (&Kind::Number, "123"));
        assert_eq!(tokens[1], (&Kind::Add, "+"));
        assert_eq!(tokens[2], (&Kind::Number, "456"));
    }

    #[test]
    fn lexer_longest_match_overlapping_literals() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Eq,
            EqEq,
        }

        let mut t = Table::new("=".to_string());
        t.add("=", Kind::Eq).unwrap();
        t.add("==", Kind::EqEq).unwrap();

        // "==" should match as EqEq
        let tokens: Vec<_> = t.lexer("==").unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], (&Kind::EqEq, "=="));

        // "=" should match as Eq
        let tokens: Vec<_> = t.lexer("=").unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], (&Kind::Eq, "="));

        // "===" should be EqEq + Eq
        let tokens: Vec<_> = t.lexer("===").unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], (&Kind::EqEq, "=="));
        assert_eq!(tokens[1], (&Kind::Eq, "="));
    }

    #[test]
    fn lexer_longest_match_backtrack() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Ab,
            Abc,
            X,
        }

        let mut t = Table::new("abcx".to_string());
        t.add("ab", Kind::Ab).unwrap();
        t.add("abc", Kind::Abc).unwrap();
        t.add("x", Kind::X).unwrap();

        // "abx" - matches "ab" then "x" (not "abc" partial)
        let tokens: Vec<_> = t.lexer("abx").unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], (&Kind::Ab, "ab"));
        assert_eq!(tokens[1], (&Kind::X, "x"));
    }

    #[test]
    fn lexer_longest_match_greedy() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            A,
            Aa,
            Aaa,
        }

        let mut t = Table::new("a".to_string());
        t.add("a", Kind::A).unwrap();
        t.add("aa", Kind::Aa).unwrap();
        t.add("aaa", Kind::Aaa).unwrap();

        let tokens: Vec<_> = t.lexer("aaa").unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], (&Kind::Aaa, "aaa"));

        let tokens: Vec<_> = t.lexer("aaaa").unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], (&Kind::Aaa, "aaa"));
        assert_eq!(tokens[1], (&Kind::A, "a"));

        let tokens: Vec<_> = t.lexer("aaaaa").unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], (&Kind::Aaa, "aaa"));
        assert_eq!(tokens[1], (&Kind::Aa, "aa"));
    }

    // ========================================================================
    // ERROR HANDLING
    // ========================================================================

    #[test]
    fn lexer_error_unknown_char_immediate() {
        // Il lexer restituisce errore immediatamente quando incontra
        // un carattere fuori dall'alfabeto, anche durante un match parziale
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Number,
        }

        let mut t = Table::new("0123456789".to_string());
        t.add("[0123456789]+", Kind::Number).unwrap();

        let mut iter = t.lexer("12@34").unwrap();

        // Errore immediato su '@', non restituisce "12" prima
        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(
            err,
            LexerError::UnknownChar {
                char: '@',
                position: 2
            }
        );
    }

    #[test]
    fn lexer_error_unknown_char_at_start() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Number,
        }

        let mut t = Table::new("0123456789".to_string());
        t.add("[0123456789]+", Kind::Number).unwrap();

        let mut iter = t.lexer("@123").unwrap();

        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(
            err,
            LexerError::UnknownChar {
                char: '@',
                position: 0
            }
        );
    }

    #[test]
    fn lexer_error_unknown_char_after_valid_token() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            A,
            B,
        }

        let mut t = Table::new("ab".to_string());
        t.add("a", Kind::A).unwrap();
        t.add("b", Kind::B).unwrap();

        let mut iter = t.lexer("a@b").unwrap();

        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(
            err,
            LexerError::UnknownChar {
                char: '@',
                position: 1
            }
        );
    }

    #[test]
    fn lexer_error_unexpected_end_no_pattern() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Abc,
        }

        let mut t = Table::new("abcdef".to_string());
        t.add("abc", Kind::Abc).unwrap();

        let mut iter = t.lexer("abcdef").unwrap();

        // "abc" matches
        let (kind, content) = iter.next().unwrap().unwrap();
        assert_eq!(kind, &Kind::Abc);
        assert_eq!(content, "abc");

        // "def" is in alphabet but no pattern matches
        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(err, LexerError::UnexpectedEnd { position: 3 });
    }

    #[test]
    fn lexer_error_unexpected_end_at_start() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Abc,
        }

        let mut t = Table::new("abcdef".to_string());
        t.add("abc", Kind::Abc).unwrap();

        let mut iter = t.lexer("def").unwrap();

        // "def" starts with 'd' which has no transition from root
        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(err, LexerError::UnexpectedEnd { position: 0 });
    }

    #[test]
    fn lexer_error_invalid_string_non_ascii() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Word,
        }

        let mut t = Table::new("abc".to_string());
        t.add("[abc]+", Kind::Word).unwrap();

        let result = t.lexer("héllo");
        assert!(matches!(result, Err(LexerError::InvalidString(_))));
    }

    #[test]
    fn lexer_error_invalid_string_emoji() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Word,
        }

        let mut t = Table::new("abc".to_string());
        t.add("[abc]+", Kind::Word).unwrap();

        let result = t.lexer("abc😀");
        assert!(matches!(result, Err(LexerError::InvalidString(_))));
    }

    // ========================================================================
    // WHITESPACE HANDLING
    // ========================================================================

    #[test]
    fn lexer_with_whitespace_in_alphabet() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Number,
            Space,
        }

        let mut t = Table::new("0123456789 ".to_string());
        t.add("[0123456789]+", Kind::Number).unwrap();
        t.add(" +", Kind::Space).unwrap();

        let tokens: Vec<_> = t
            .lexer("1 2  3")
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], (&Kind::Number, "1"));
        assert_eq!(tokens[1], (&Kind::Space, " "));
        assert_eq!(tokens[2], (&Kind::Number, "2"));
        assert_eq!(tokens[3], (&Kind::Space, "  "));
        assert_eq!(tokens[4], (&Kind::Number, "3"));
    }

    #[test]
    fn lexer_whitespace_not_in_alphabet_immediate_error() {
        // Errore immediato quando incontra spazio fuori alfabeto
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Number,
        }

        let mut t = Table::new("0123456789".to_string());
        t.add("[0123456789]+", Kind::Number).unwrap();

        let mut iter = t.lexer("1 2").unwrap();

        // Errore immediato, non restituisce "1" prima
        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(
            err,
            LexerError::UnknownChar {
                char: ' ',
                position: 1
            }
        );
    }

    // ========================================================================
    // COMPLEX PATTERNS
    // ========================================================================

    #[test]
    fn lexer_identifier_and_numbers_separate_alphabets() {
        // Usiamo alfabeti non sovrapposti per evitare ambiguità
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Ident,
            Number,
            Eq,
        }

        let mut t = Table::new("abcxyz0123456789=".to_string());
        t.add("[abcxyz]+", Kind::Ident).unwrap();
        t.add("[0123456789]+", Kind::Number).unwrap();
        t.add("=", Kind::Eq).unwrap();

        let tokens: Vec<_> = t.lexer("x=42").unwrap().collect::<Result<_, _>>().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], (&Kind::Ident, "x"));
        assert_eq!(tokens[1], (&Kind::Eq, "="));
        assert_eq!(tokens[2], (&Kind::Number, "42"));
    }

    #[test]
    fn lexer_multiple_operators() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Lt,
            Le,
            Gt,
            Ge,
            Eq,
            EqEq,
            Not,
            Ne,
        }

        let mut t = Table::new("<=!>".to_string());
        t.add("<", Kind::Lt).unwrap();
        t.add("<=", Kind::Le).unwrap();
        t.add(">", Kind::Gt).unwrap();
        t.add(">=", Kind::Ge).unwrap();
        t.add("=", Kind::Eq).unwrap();
        t.add("==", Kind::EqEq).unwrap();
        t.add("!", Kind::Not).unwrap();
        t.add("!=", Kind::Ne).unwrap();

        let tokens: Vec<_> = t
            .lexer("<=>=!===!=")
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], (&Kind::Le, "<="));
        assert_eq!(tokens[1], (&Kind::Ge, ">="));
        assert_eq!(tokens[2], (&Kind::Ne, "!="));
        assert_eq!(tokens[3], (&Kind::EqEq, "=="));
        assert_eq!(tokens[4], (&Kind::Ne, "!="));
    }

    #[test]
    fn lexer_hex_numbers() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Hex,
        }

        let mut t = Table::new("0123456789abcdef".to_string());
        t.add("[0123456789abcdef]+", Kind::Hex).unwrap();

        let tokens: Vec<_> = t
            .lexer("deadbeef")
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], (&Kind::Hex, "deadbeef"));
    }

    // ========================================================================
    // EDGE CASES
    // ========================================================================

    #[test]
    fn lexer_single_char_tokens() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            A,
        }

        let mut t = Table::new("a".to_string());
        t.add("a", Kind::A).unwrap();

        let tokens: Vec<_> = t.lexer("aaa").unwrap().collect::<Result<_, _>>().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], (&Kind::A, "a"));
        assert_eq!(tokens[1], (&Kind::A, "a"));
        assert_eq!(tokens[2], (&Kind::A, "a"));
    }

    #[test]
    fn lexer_very_long_token() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            A,
        }

        let mut t = Table::new("a".to_string());
        t.add("a+", Kind::A).unwrap();

        let long_input = "a".repeat(10000);
        let tokens: Vec<_> = t
            .lexer(&long_input)
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, &Kind::A);
        assert_eq!(tokens[0].1.len(), 10000);
    }

    #[test]
    fn lexer_alternating_tokens() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            A,
            B,
        }

        let mut t = Table::new("ab".to_string());
        t.add("a+", Kind::A).unwrap();
        t.add("b+", Kind::B).unwrap();

        let tokens: Vec<_> = t
            .lexer("aaabbbaaab")
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], (&Kind::A, "aaa"));
        assert_eq!(tokens[1], (&Kind::B, "bbb"));
        assert_eq!(tokens[2], (&Kind::A, "aaa"));
        assert_eq!(tokens[3], (&Kind::B, "b"));
    }

    #[test]
    fn lexer_partial_match_at_end() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Abc,
        }

        let mut t = Table::new("abc".to_string());
        t.add("abc", Kind::Abc).unwrap();

        // "abcab" - first "abc" matches, then "ab" is partial with no match
        let mut iter = t.lexer("abcab").unwrap();

        let (kind, content) = iter.next().unwrap().unwrap();
        assert_eq!(kind, &Kind::Abc);
        assert_eq!(content, "abc");

        // "ab" has no complete match
        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(err, LexerError::UnexpectedEnd { position: 3 });
    }

    #[test]
    fn lexer_empty_table() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {}

        let t: Table<Kind> = Table::new("abc".to_string());

        let mut iter = t.lexer("abc").unwrap();

        // No patterns defined, should fail immediately
        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(err, LexerError::UnexpectedEnd { position: 0 });
    }

    #[test]
    fn lexer_collect_result_success() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Num,
            Op,
        }

        let mut t = Table::new("0123456789+".to_string());
        t.add("[0123456789]+", Kind::Num).unwrap();
        t.add("+", Kind::Op).unwrap();

        let result: Result<Vec<_>, _> = t.lexer("1+2").unwrap().collect();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn lexer_collect_result_failure() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Num,
            Op,
        }

        let mut t = Table::new("0123456789+".to_string());
        t.add("[0123456789]+", Kind::Num).unwrap();
        t.add("+", Kind::Op).unwrap();

        let result: Result<Vec<_>, _> = t.lexer("1@2").unwrap().collect();
        assert!(result.is_err());
    }

    #[test]
    fn lexer_only_repetition() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Stars,
        }

        let mut t = Table::new("*".to_string());
        t.add("*+", Kind::Stars).unwrap();

        let tokens: Vec<_> = t.lexer("****").unwrap().collect::<Result<_, _>>().unwrap();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], (&Kind::Stars, "****"));
    }

    #[test]
    fn lexer_mixed_fixed_and_repetition() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Prefix,
            Suffix,
        }

        let mut t = Table::new("ab".to_string());
        t.add("a+b", Kind::Prefix).unwrap(); // one or more 'a' followed by 'b'

        let tokens: Vec<_> = t.lexer("aaab").unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], (&Kind::Prefix, "aaab"));

        let tokens: Vec<_> = t.lexer("ab").unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], (&Kind::Prefix, "ab"));
    }

    // ========================================================================
    // REAL-WORLD SCENARIOS
    // ========================================================================

    #[test]
    fn lexer_arithmetic_expression() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Num,
            Add,
            Sub,
            Mul,
            Div,
            LParen,
            RParen,
        }

        let mut t = Table::new("0123456789+-*/()".to_string());
        t.add("[0123456789]+", Kind::Num).unwrap();
        t.add("+", Kind::Add).unwrap();
        t.add("-", Kind::Sub).unwrap();
        t.add("*", Kind::Mul).unwrap();
        t.add("/", Kind::Div).unwrap();
        t.add("(", Kind::LParen).unwrap();
        t.add(")", Kind::RParen).unwrap();

        let tokens: Vec<_> = t
            .lexer("(1+2)*3")
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], (&Kind::LParen, "("));
        assert_eq!(tokens[1], (&Kind::Num, "1"));
        assert_eq!(tokens[2], (&Kind::Add, "+"));
        assert_eq!(tokens[3], (&Kind::Num, "2"));
        assert_eq!(tokens[4], (&Kind::RParen, ")"));
        assert_eq!(tokens[5], (&Kind::Mul, "*"));
        assert_eq!(tokens[6], (&Kind::Num, "3"));
    }

    #[test]
    fn lexer_complex_arithmetic() {
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            Num,
            Add,
            Sub,
            Mul,
            Div,
            LParen,
            RParen,
        }

        let mut t = Table::new("0123456789+-*/()".to_string());
        t.add("[0123456789]+", Kind::Num).unwrap();
        t.add("+", Kind::Add).unwrap();
        t.add("-", Kind::Sub).unwrap();
        t.add("*", Kind::Mul).unwrap();
        t.add("/", Kind::Div).unwrap();
        t.add("(", Kind::LParen).unwrap();
        t.add(")", Kind::RParen).unwrap();

        let tokens: Vec<_> = t
            .lexer("((10+20)*(30-5))/2")
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(tokens.len(), 15);
        assert_eq!(tokens[0], (&Kind::LParen, "("));
        assert_eq!(tokens[1], (&Kind::LParen, "("));
        assert_eq!(tokens[2], (&Kind::Num, "10"));
        assert_eq!(tokens[3], (&Kind::Add, "+"));
        assert_eq!(tokens[4], (&Kind::Num, "20"));
        assert_eq!(tokens[5], (&Kind::RParen, ")"));
        assert_eq!(tokens[6], (&Kind::Mul, "*"));
        assert_eq!(tokens[7], (&Kind::LParen, "("));
        assert_eq!(tokens[8], (&Kind::Num, "30"));
        assert_eq!(tokens[9], (&Kind::Sub, "-"));
        assert_eq!(tokens[10], (&Kind::Num, "5"));
        assert_eq!(tokens[11], (&Kind::RParen, ")"));
        assert_eq!(tokens[12], (&Kind::RParen, ")"));
        assert_eq!(tokens[13], (&Kind::Div, "/"));
        assert_eq!(tokens[14], (&Kind::Num, "2"));
    }

    #[test]
    fn lexer_simple_tokens_no_overlap() {
        // Semplice scenario senza ambiguità
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            LBrace,
            RBrace,
            Comma,
            Colon,
            Num,
        }

        let mut t = Table::new("{},:0123456789".to_string());
        t.add("{", Kind::LBrace).unwrap();
        t.add("}", Kind::RBrace).unwrap();
        t.add(",", Kind::Comma).unwrap();
        t.add(":", Kind::Colon).unwrap();
        t.add("[0123456789]+", Kind::Num).unwrap();

        let tokens: Vec<_> = t
            .lexer("{1:2,3:4}")
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[0], (&Kind::LBrace, "{"));
        assert_eq!(tokens[1], (&Kind::Num, "1"));
        assert_eq!(tokens[2], (&Kind::Colon, ":"));
        assert_eq!(tokens[3], (&Kind::Num, "2"));
        assert_eq!(tokens[4], (&Kind::Comma, ","));
        assert_eq!(tokens[5], (&Kind::Num, "3"));
        assert_eq!(tokens[6], (&Kind::Colon, ":"));
        assert_eq!(tokens[7], (&Kind::Num, "4"));
        assert_eq!(tokens[8], (&Kind::RBrace, "}"));
    }

    #[test]
    fn lexer_position_tracking() {
        // Verifica che le posizioni negli errori siano corrette
        #[derive(Debug, Clone, PartialEq)]
        enum Kind {
            A,
        }

        let mut t = Table::new("a".to_string());
        t.add("a", Kind::A).unwrap();

        // Errore a posizione 5
        let mut iter = t.lexer("aaaaa@").unwrap();

        for _ in 0..4 {
            assert!(iter.next().unwrap().is_ok());
        }

        let err = iter.next().unwrap().unwrap_err();
        assert_eq!(
            err,
            LexerError::UnknownChar {
                char: '@',
                position: 5
            }
        );
    }
}
