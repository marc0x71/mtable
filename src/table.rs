#![allow(unused)]

use crate::error::TableError;
use std::{collections::HashSet, fmt::Debug};

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

    fn set_children(&mut self, index: usize, child: usize) {
        if let Some(c) = self.children.get_mut(index) {
            *c = Some(child);
        }
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
}
