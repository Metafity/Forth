use std::{collections::HashMap, rc::Rc};

pub type Value = i32;
pub type Result = std::result::Result<(), Error>;

pub struct Forth {
    stack: Vec<i32>,
    vars: HashMap<String, Rc<Vec<Op>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
}
pub enum TokenType {
    Word(String),
    Num(i32),
}

pub enum Op{
    Word(String),
    Num(i32),
    Ref(Rc<Vec<Op>>)
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
        vars.insert("+".to_string(), Rc::new(vec![Op::Word("+".to_string())]));
        vars.insert("-".to_string(), Rc::new(vec![Op::Word("-".to_string())]));
        vars.insert("*".to_string(), Rc::new(vec![Op::Word("*".to_string())]));
        vars.insert("/".to_string(), Rc::new(vec![Op::Word("/".to_string())]));
        vars.insert("DUP".to_string(), Rc::new(vec![Op::Word("DUP".to_string())]));
        vars.insert("DROP".to_string(), Rc::new(vec![Op::Word("DROP".to_string())]));
        vars.insert("SWAP".to_string(), Rc::new(vec![Op::Word("SWAP".to_string())]));
        vars.insert("OVER".to_string(), Rc::new(vec![Op::Word("OVER".to_string())]));

        Forth {
            stack: Vec::new(),
            vars,
        }
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }
    pub fn evaluate_token_type(token: &str) -> TokenType {
        match token.parse::<i32>() {
            Ok(num) =>  TokenType::Num(num),
            _ => TokenType::Word(token.to_owned().to_ascii_uppercase())
        }   
    }

    pub fn push_in_stack(&mut self, token: &Op) -> Result {
        match token {
            Op::Word(input) => {
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
            Op::Num(num) => {
                self.stack.push(*num);
                Ok(())
            }
            Op::Ref(ops) => {
                for op in ops.iter() {
                    Self::push_in_stack(self, op)?;
                }
                Ok(())
            },
        }
    }

    pub fn eval(&mut self, input: &str) -> Result {
        let tokens = input.split_whitespace();
        let mut state: WordReadState = WordReadState::NotReading;
        let mut temp_key: String = String::default();
        let mut temp_value: Vec<Op> = Vec::new();

        for token in tokens {
            match (&state, Self::evaluate_token_type(token)) {
                (WordReadState::NotReading, TokenType::Word(word)) => match word.as_str() {
                    ":" => {
                        state = WordReadState::ToreadWord;
                    }
                    ";" => return Err(Error::InvalidWord),
                    word => {
                        let def = self.vars.get(word).cloned();
                        match def {
                            Some(items) => {
                                for i in items.iter() {
                                    match self.push_in_stack(i) {
                                        Ok(_) => (),
                                        Err(err) => {return Err(err)},
                                    }
                                }
                            }
                            None => return Err(Error::UnknownWord),
                        }
                    }
                },
                (WordReadState::NotReading, TokenType::Num(num)) => {
                    match self.push_in_stack(&Op::Num(num)) {
                        Ok(_) => {}
                        Err(err) => return Err(err),
                    }
                }
                (WordReadState::ToreadWord, TokenType::Word(_word)) => match token {
                    ":" => return Err(Error::InvalidWord),
                    ";" => return Err(Error::InvalidWord),
                    word => {
                        state = WordReadState::ToreadDef;
                        temp_key = word.to_ascii_uppercase();
                        
                    }
                },
                (WordReadState::ToreadWord, TokenType::Num(_num)) => return Err(Error::InvalidWord),
                (WordReadState::ToreadDef, TokenType::Word(word)) => match word.as_str() {
                    ";" => {
                        if temp_value.is_empty() {
                            return Err(Error::UnknownWord);
                        }
                        else {
                            self.vars.insert(temp_key.clone(), Rc::new(temp_value));
                            temp_value = Vec::new();
                            state = WordReadState::NotReading;
                        }
                    }
                    ":" => {
                        return Err(Error::InvalidWord);
                    }
                    word => match self.vars.get(word) {
                        Some(def) => {
                            temp_value.push(Op::Ref(Rc::clone(def)));
                        }
                        None => return Err(Error::UnknownWord),
                    },
                },
                (WordReadState::ToreadDef, TokenType::Num(num)) => {
                    temp_value.push(Op::Num(num));
                }
            }
        }

        match state {
            WordReadState::NotReading => Ok(()),
            WordReadState::ToreadWord => Err(Error::InvalidWord),
            WordReadState::ToreadDef => Err(Error::InvalidWord),
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
    #[test]

    fn foobar() {
        let mut f = Forth::new();
        assert!(f.eval(": bar 5 ;").is_ok());
        assert!(f.eval(": foo bar ;").is_ok());
        assert!(f.eval(": bar 7 ;").is_ok());
        assert!(f.eval("foo bar").is_ok());
        assert_eq!(vec![5, 7], f.stack());
    }

    #[test]
    #[ignore]
    fn alloc_attack() {
        let mut f = Forth::new();
        f.eval(": a 0 drop ;").unwrap();
        f.eval(": b a a ;").unwrap();
        f.eval(": c b b ;").unwrap();
        f.eval(": d c c ;").unwrap();
        f.eval(": e d d ;").unwrap();
        f.eval(": f e e ;").unwrap();
        f.eval(": g f f ;").unwrap();
        f.eval(": h g g ;").unwrap();
        f.eval(": i h h ;").unwrap();
        f.eval(": j i i ;").unwrap();
        f.eval(": k j j ;").unwrap();
        f.eval(": l k k ;").unwrap();
        f.eval(": m l l ;").unwrap();
        f.eval(": n m m ;").unwrap();
        f.eval(": o n n ;").unwrap();
        f.eval(": p o o ;").unwrap();
        f.eval(": q p p ;").unwrap();
        f.eval(": r q q ;").unwrap();
        f.eval(": s r r ;").unwrap();
        f.eval(": t s s ;").unwrap();
        f.eval(": u t t ;").unwrap();
        f.eval(": v u u ;").unwrap();
        f.eval(": w v v ;").unwrap();
        f.eval(": x w w ;").unwrap();
        f.eval(": y x x ;").unwrap();
        f.eval(": z y y ;").unwrap();
        // On an implementation with eager expansion of sub-custom-words,
        // `z`'s definition is 2**26 items long. Assuming the implementation
        // has compacted each instruction into a single byte, that takes up
        // over 50 Mb of memory. Less efficient implementations will require
        // more.
        //
        // This shouldn't crash or hang anyone's machine, but it's at least a
        // testable proposition. A good implementation shouldn't be doing eager
        // stack expansion, so it should require much less than that.
        // Sanity check--few implementations should fail here.
        assert!(f.stack().is_empty());
    }
}
