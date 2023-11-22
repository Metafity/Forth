use std::{collections::HashMap, sync::OnceState};

pub type Value = i32;
pub type Result = std::result::Result<(), Error>;

pub struct Forth {
    stack: Vec<i32>,
    vars: HashMap<String, Vec<String>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
}

pub enum TokenType {
    Word,
    Num,
}

pub enum WordReadState {
    NotReading,
    ToreadWord,
    ToreadDef,
}
impl Default for Forth {
    fn default() -> Self {
        Self::new()
    }
}

impl Forth {
    pub fn new() -> Forth {
        let mut vars = HashMap::new();
        vars.insert("+".to_string(), vec!["+".to_string()]);
        vars.insert("-".to_string(), vec!["-".to_string()]);
        vars.insert("*".to_string(), vec!["*".to_string()]);
        vars.insert("/".to_string(), vec!["/".to_string()]);
        vars.insert("DUP".to_string(), vec!["DUP".to_string()]);
        vars.insert("DROP".to_string(), vec!["DROP".to_string()]);
        vars.insert("SWAP".to_string(), vec!["SWAP".to_string()]);
        vars.insert("OVER".to_string(), vec!["OVER".to_string()]);

        Forth {
            stack: Vec::new(),
            vars,
        }
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    pub fn push_in_stack(&mut self, input: String, token_type: TokenType) -> Result {
        match token_type {
            TokenType::Word => {
                if let Some(second_operand) = self.stack.pop() {
                    match input.as_str() {
                        "DUP" => {
                            self.stack.push(second_operand);
                            self.stack.push(second_operand);
                            Ok(())
                        }
                        "DROP" => Ok(()),
                        input => {
                            if let Some(first_operand) = self.stack.pop() {
                                match input {
                                    "+" => {
                                        self.stack.push(first_operand + second_operand);
                                        Ok(())
                                    }
                                    "-" => {
                                        self.stack.push(first_operand - second_operand);
                                        Ok(())
                                    }
                                    "*" => {
                                        self.stack.push(first_operand * second_operand);
                                        Ok(())
                                    }
                                    "/" => {
                                        if second_operand == 0 {
                                            return Err(Error::DivisionByZero);
                                        }
                                        self.stack.push(first_operand / second_operand);
                                        Ok(())
                                    }
                                    "SWAP" => {
                                        self.stack.push(second_operand);
                                        self.stack.push(first_operand);
                                        Ok(())
                                    }
                                    "OVER" => {
                                        self.stack.push(first_operand);
                                        self.stack.push(second_operand);
                                        self.stack.push(first_operand);
                                        Ok(())
                                    }
                                    _ => Err(Error::InvalidWord),
                                }
                            } else {
                                Err(Error::StackUnderflow)
                            }
                        }
                    }
                } else {
                    Err(Error::StackUnderflow)
                }
            }
            TokenType::Num => {
                self.stack.push(input.parse::<i32>().unwrap());
                Ok(())
            }
        }
    }

    pub fn eval(&mut self, input: &str) -> Result {
        let tokens = input.split_whitespace();
        let mut state: WordReadState = WordReadState::NotReading;
        let mut temp_key: String = String::default();
        let mut temp_value: Vec<String> = Vec::default();

        for token in tokens {
            match (&state, Self::evaluate_token_type(token)) {
                (WordReadState::NotReading, TokenType::Word) => match token {
                    ":" => {
                        state = WordReadState::ToreadWord;
                    }
                    ";" => return Err(Error::InvalidWord),
                    word => {
                        let def = self.vars.get(word.to_ascii_uppercase().as_str()).cloned();
                        match def {
                            Some(items) => {
                                for item in items {
                                    match Self::evaluate_token_type(&item) {
                                        TokenType::Word => {
                                            match self.push_in_stack(item, TokenType::Word) {
                                                Ok(_) => {}
                                                Err(err) => return Err(err),
                                            }
                                        }
                                        TokenType::Num => {
                                            match self.push_in_stack(item, TokenType::Num) {
                                                Ok(_) => {}
                                                Err(err) => return Err(err),
                                            }
                                        }
                                    }
                                }
                            }
                            None => return Err(Error::UnknownWord),
                        }
                    }
                },
                (WordReadState::NotReading, TokenType::Num) => {
                    match self.push_in_stack(token.to_string(), TokenType::Num) {
                        Ok(_) => {}
                        Err(err) => return Err(err),
                    }
                }
                (WordReadState::ToreadWord, TokenType::Word) => match token {
                    ":" => return Err(Error::InvalidWord),
                    ";" => return Err(Error::InvalidWord),
                    word => {
                        state = WordReadState::ToreadDef;
                        temp_key = word.to_owned().to_ascii_uppercase();
                        temp_value.clear();
                    }
                },
                (WordReadState::ToreadWord, TokenType::Num) => return Err(Error::InvalidWord),
                (WordReadState::ToreadDef, TokenType::Word) => match token {
                    ";" => {
                        if temp_value.is_empty() {
                            return Err(Error::UnknownWord);
                        }
                        self.vars.insert(temp_key.clone(), temp_value.clone());
                        state = WordReadState::NotReading;
                    }
                    ":" => {
                        return Err(Error::InvalidWord);
                    }
                    word => match self.vars.get(word.to_ascii_uppercase().as_str()) {
                        Some(def) => {
                            for x in def {
                                temp_value.push(x.to_string().to_ascii_uppercase());
                            }
                        }
                        None => return Err(Error::UnknownWord),
                    },
                },
                (WordReadState::ToreadDef, TokenType::Num) => {
                    temp_value.push(token.to_owned());
                }
            }
        }

        match state {
            WordReadState::NotReading => Ok(()),
            WordReadState::ToreadWord => Err(Error::InvalidWord),
            WordReadState::ToreadDef => Err(Error::InvalidWord),
        }
    }

    pub fn evaluate_token_type(token: &str) -> TokenType {
        if token.parse::<i32>().is_ok() {
            TokenType::Num
        } else {
            TokenType::Word
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, Forth, Value};

    #[test]
    fn no_input_no_stack() {
        assert_eq!(Vec::<Value>::new(), Forth::new().stack());
    }
    #[test]

    fn numbers_just_get_pushed_onto_the_stack() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 3 4 5").is_ok());
        assert_eq!(vec![1, 2, 3, 4, 5], f.stack());
    }
    #[test]

    fn can_add_two_numbers() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 +").is_ok());
        assert_eq!(vec![3], f.stack());
    }
    #[test]

    fn addition_error() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::StackUnderflow), f.eval("1 +"));
        assert_eq!(Err(Error::StackUnderflow), f.eval("+"));
    }
    #[test]

    fn can_subtract_two_numbers() {
        let mut f = Forth::new();
        assert!(f.eval("3 4 -").is_ok());
        assert_eq!(vec![-1], f.stack());
    }
    #[test]

    fn subtraction_error() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::StackUnderflow), f.eval("1 -"));
        assert_eq!(Err(Error::StackUnderflow), f.eval("-"));
    }
    #[test]

    fn can_multiply_two_numbers() {
        let mut f = Forth::new();
        assert!(f.eval("2 4 *").is_ok());
        assert_eq!(vec![8], f.stack());
    }
    #[test]

    fn multiplication_error() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::StackUnderflow), f.eval("1 *"));
        assert_eq!(Err(Error::StackUnderflow), f.eval("*"));
    }
    #[test]

    fn can_divide_two_numbers() {
        let mut f = Forth::new();
        assert!(f.eval("12 3 /").is_ok());
        assert_eq!(vec![4], f.stack());
    }
    #[test]

    fn performs_integer_division() {
        let mut f = Forth::new();
        assert!(f.eval("8 3 /").is_ok());
        assert_eq!(vec![2], f.stack());
    }
    #[test]

    fn division_error() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::StackUnderflow), f.eval("1 /"));
        assert_eq!(Err(Error::StackUnderflow), f.eval("/"));
    }
    #[test]

    fn errors_if_dividing_by_zero() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::DivisionByZero), f.eval("4 0 /"));
    }
    #[test]

    fn addition_and_subtraction() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 + 4 -").is_ok());
        assert_eq!(vec![-1], f.stack());
    }
    #[test]

    fn multiplication_and_division() {
        let mut f = Forth::new();
        assert!(f.eval("2 4 * 3 /").is_ok());
        assert_eq!(vec![2], f.stack());
    }
    #[test]

    fn dup() {
        let mut f = Forth::new();
        assert!(f.eval("1 dup").is_ok());
        assert_eq!(vec![1, 1], f.stack());
    }
    #[test]

    fn dup_top_value_only() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 dup").is_ok());
        assert_eq!(vec![1, 2, 2], f.stack());
    }
    #[test]

    fn dup_case_insensitive() {
        let mut f = Forth::new();
        assert!(f.eval("1 DUP Dup dup").is_ok());
        assert_eq!(vec![1, 1, 1, 1], f.stack());
    }
    #[test]

    fn dup_error() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::StackUnderflow), f.eval("dup"));
    }
    #[test]

    fn drop() {
        let mut f = Forth::new();
        assert!(f.eval("1 drop").is_ok());
        assert_eq!(Vec::<Value>::new(), f.stack());
    }
    #[test]

    fn drop_with_two() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 drop").is_ok());
        assert_eq!(vec![1], f.stack());
    }
    #[test]

    fn drop_case_insensitive() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 3 4 DROP Drop drop").is_ok());
        assert_eq!(vec![1], f.stack());
    }
    #[test]

    fn drop_error() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::StackUnderflow), f.eval("drop"));
    }
    #[test]

    fn swap() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 swap").is_ok());
        assert_eq!(vec![2, 1], f.stack());
    }
    #[test]

    fn swap_with_three() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 3 swap").is_ok());
        assert_eq!(vec![1, 3, 2], f.stack());
    }
    #[test]

    fn swap_case_insensitive() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 SWAP 3 Swap 4 swap").is_ok());
        assert_eq!(vec![2, 3, 4, 1], f.stack());
    }
    #[test]

    fn swap_error() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::StackUnderflow), f.eval("1 swap"));
        assert_eq!(Err(Error::StackUnderflow), f.eval("swap"));
    }
    #[test]

    fn over() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 over").is_ok());
        assert_eq!(vec![1, 2, 1], f.stack());
    }
    #[test]

    fn over_with_three() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 3 over").is_ok());
        assert_eq!(vec![1, 2, 3, 2], f.stack());
    }
    #[test]

    fn over_case_insensitive() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 OVER Over over").is_ok());
        assert_eq!(vec![1, 2, 1, 2, 1], f.stack());
    }
    #[test]

    fn over_error() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::StackUnderflow), f.eval("1 over"));
        assert_eq!(Err(Error::StackUnderflow), f.eval("over"));
    }
    // User-defined words
    #[test]
    fn can_consist_of_built_in_words() {
        let mut f = Forth::new();
        assert!(f.eval(": dup-twice dup dup ;").is_ok());
        assert!(f.eval("1 dup-twice").is_ok());
        assert_eq!(vec![1, 1, 1], f.stack());
    }
    #[test]
    fn execute_in_the_right_order() {
        let mut f = Forth::new();
        assert!(f.eval(": countup 1 2 3 ;").is_ok());
        assert!(f.eval("countup").is_ok());
        assert_eq!(vec![1, 2, 3], f.stack());
    }
    #[test]
    fn redefining_an_existing_word() {
        let mut f = Forth::new();
        assert!(f.eval(": foo dup ;").is_ok());
        assert!(f.eval(": foo dup dup ;").is_ok());
        assert!(f.eval("1 foo").is_ok());
        assert_eq!(vec![1, 1, 1], f.stack());
    }
    #[test]
    fn redefining_an_existing_built_in_word() {
        let mut f = Forth::new();
        assert!(f.eval(": swap dup ;").is_ok());
        assert!(f.eval("1 swap").is_ok());
        assert_eq!(vec![1, 1], f.stack());
    }
    #[test]
    fn user_defined_words_are_case_insensitive() {
        let mut f = Forth::new();
        assert!(f.eval(": foo dup ;").is_ok());
        assert!(f.eval("1 FOO Foo foo").is_ok());
        assert_eq!(vec![1, 1, 1, 1], f.stack());
    }
    #[test]

    fn definitions_are_case_insensitive() {
        let mut f = Forth::new();
        assert!(f.eval(": SWAP DUP Dup dup ;").is_ok());
        assert!(f.eval("1 swap").is_ok());
        assert_eq!(vec![1, 1, 1, 1], f.stack());
    }
    #[test]

    fn redefining_a_built_in_operator() {
        let mut f = Forth::new();
        assert!(f.eval(": + * ;").is_ok());
        assert!(f.eval("3 4 +").is_ok());
        assert_eq!(vec![12], f.stack());
    }
    #[test]

    fn can_use_different_words_with_the_same_name() {
        let mut f = Forth::new();
        assert!(f.eval(": foo 5 ;").is_ok());
        assert!(f.eval(": bar foo ;").is_ok());
        assert!(f.eval(": foo 6 ;").is_ok());
        assert!(f.eval("bar foo").is_ok());
        assert_eq!(vec![5, 6], f.stack());
    }
    #[test]

    fn can_define_word_that_uses_word_with_the_same_name() {
        let mut f = Forth::new();
        assert!(f.eval(": foo 10 ;").is_ok());
        assert!(f.eval(": foo foo 1 + ;").is_ok());
        assert!(f.eval("foo").is_ok());
        assert_eq!(vec![11], f.stack());
    }
    #[test]

    fn defining_a_number() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::InvalidWord), f.eval(": 1 2 ;"));
    }
    #[test]

    fn malformed_word_definition() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::InvalidWord), f.eval(":"));
        assert_eq!(Err(Error::InvalidWord), f.eval(": foo"));
        assert_eq!(Err(Error::InvalidWord), f.eval(": foo 1"));
    }
    #[test]

    fn calling_non_existing_word() {
        let mut f = Forth::new();
        assert_eq!(Err(Error::UnknownWord), f.eval("1 foo"));
    }
    #[test]

    fn multiple_definitions() {
        let mut f = Forth::new();
        assert!(f.eval(": one 1 ; : two 2 ; one two +").is_ok());
        assert_eq!(vec![3], f.stack());
    }
    #[test]

    fn definitions_after_ops() {
        let mut f = Forth::new();
        assert!(f.eval("1 2 + : addone 1 + ; addone").is_ok());
        assert_eq!(vec![4], f.stack());
    }
    #[test]

    fn redefine_an_existing_word_with_another_existing_word() {
        let mut f = Forth::new();
        assert!(f.eval(": foo 5 ;").is_ok());
        assert!(f.eval(": bar foo ;").is_ok());
        assert!(f.eval(": foo 6 ;").is_ok());
        assert!(f.eval(": bar foo ;").is_ok());
        assert!(f.eval("bar foo").is_ok());
        assert_eq!(vec![6, 6], f.stack());
    }
}
