// Selection query parser
// Grammar:
//   selection := term (bool_op term)*
//   term := keyword [comparison value] | "within" number "of" selection | "(" selection ")"
//   keyword := "all" | "element" | "resname" | "chain" | "resid" | "protein" | "water" | "backbone"
//   bool_op := "and" | "or" | "not"
//   comparison := "=" | "!="
//   value := string | number | range

use crate::errors::{AxiomError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionToken {
    // Keywords
    All,
    Element,
    Resname,
    Chain,
    Resid,
    Protein,
    Water,
    Backbone,
    Sidechain,

    // Boolean operators
    And,
    Or,
    Not,

    // Spatial operators
    Within,
    Of,

    // Comparisons
    Equals,
    NotEquals,

    // Values
    String(String),
    Number(f32),
    Range(u32, u32),

    // Structure
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone)]
pub enum SelectionAST {
    All,
    Element(String),     // Element symbol (O, H, C, etc.)
    Resname(String),     // Residue name (WAT, ALA, etc.)
    Chain(String),       // Chain ID (A, B, etc.)
    Resid(u32),          // Residue index
    ResidRange(u32, u32), // Residue range
    Protein,             // Built-in macro for protein atoms
    Water,               // Built-in macro for water
    Backbone,            // Built-in macro for backbone atoms
    Sidechain,           // Built-in macro for sidechain atoms
    Within(f32, Box<SelectionAST>), // Spatial query: within distance of selection
    And(Box<SelectionAST>, Box<SelectionAST>),
    Or(Box<SelectionAST>, Box<SelectionAST>),
    Not(Box<SelectionAST>),
}

/// Tokenize a selection string
fn tokenize(input: &str) -> Result<Vec<SelectionToken>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' => {
                chars.next();
            }
            '(' => {
                tokens.push(SelectionToken::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(SelectionToken::RightParen);
                chars.next();
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next(); // Skip second '=' if present
                }
                tokens.push(SelectionToken::Equals);
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(SelectionToken::NotEquals);
                } else {
                    return Err(AxiomError::SelectionError(format!("Unexpected character: !")));
                }
            }
            '0'..='9' | '-' => {
                // Parse number or range
                let mut num_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' || ch == '-' {
                        num_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                // Check for range (e.g., "10-20")
                if num_str.contains('-') && !num_str.starts_with('-') {
                    let parts: Vec<&str> = num_str.split('-').collect();
                    if parts.len() == 2 {
                        let start: u32 = parts[0].parse().map_err(|_| {
                            AxiomError::SelectionError(format!("Invalid range: {}", num_str))
                        })?;
                        let end: u32 = parts[1].parse().map_err(|_| {
                            AxiomError::SelectionError(format!("Invalid range: {}", num_str))
                        })?;
                        tokens.push(SelectionToken::Range(start, end));
                    } else {
                        return Err(AxiomError::SelectionError(format!("Invalid range: {}", num_str)));
                    }
                } else {
                    let num: f32 = num_str.parse().map_err(|_| {
                        AxiomError::SelectionError(format!("Invalid number: {}", num_str))
                    })?;
                    tokens.push(SelectionToken::Number(num));
                }
            }
            _ => {
                // Parse word
                let mut word = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        word.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let token = match word.to_lowercase().as_str() {
                    "all" => SelectionToken::All,
                    "element" => SelectionToken::Element,
                    "resname" => SelectionToken::Resname,
                    "chain" => SelectionToken::Chain,
                    "resid" => SelectionToken::Resid,
                    "protein" => SelectionToken::Protein,
                    "water" => SelectionToken::Water,
                    "backbone" => SelectionToken::Backbone,
                    "sidechain" => SelectionToken::Sidechain,
                    "and" => SelectionToken::And,
                    "or" => SelectionToken::Or,
                    "not" => SelectionToken::Not,
                    "within" => SelectionToken::Within,
                    "of" => SelectionToken::Of,
                    _ => SelectionToken::String(word),
                };
                tokens.push(token);
            }
        }
    }

    Ok(tokens)
}

/// Parse tokens into an AST
pub fn parse_selection(input: &str) -> Result<SelectionAST> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

struct Parser {
    tokens: Vec<SelectionToken>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<SelectionToken>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> Option<&SelectionToken> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: SelectionToken) -> Result<()> {
        if self.current() == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(AxiomError::SelectionError(format!(
                "Expected {:?}, got {:?}",
                expected,
                self.current()
            )))
        }
    }

    fn parse(&mut self) -> Result<SelectionAST> {
        self.parse_or()
    }

    // Precedence: or > and > not > term
    fn parse_or(&mut self) -> Result<SelectionAST> {
        let mut left = self.parse_and()?;

        while self.current() == Some(&SelectionToken::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = SelectionAST::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<SelectionAST> {
        let mut left = self.parse_not()?;

        while self.current() == Some(&SelectionToken::And) {
            self.advance();
            let right = self.parse_not()?;
            left = SelectionAST::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_not(&mut self) -> Result<SelectionAST> {
        if self.current() == Some(&SelectionToken::Not) {
            self.advance();
            let expr = self.parse_not()?;
            Ok(SelectionAST::Not(Box::new(expr)))
        } else {
            self.parse_term()
        }
    }

    fn parse_term(&mut self) -> Result<SelectionAST> {
        match self.current() {
            Some(SelectionToken::LeftParen) => {
                self.advance();
                let expr = self.parse()?;
                self.expect(SelectionToken::RightParen)?;
                Ok(expr)
            }
            Some(SelectionToken::All) => {
                self.advance();
                Ok(SelectionAST::All)
            }
            Some(SelectionToken::Protein) => {
                self.advance();
                Ok(SelectionAST::Protein)
            }
            Some(SelectionToken::Water) => {
                self.advance();
                Ok(SelectionAST::Water)
            }
            Some(SelectionToken::Backbone) => {
                self.advance();
                Ok(SelectionAST::Backbone)
            }
            Some(SelectionToken::Sidechain) => {
                self.advance();
                Ok(SelectionAST::Sidechain)
            }
            Some(SelectionToken::Element) => {
                self.advance();
                if let Some(SelectionToken::String(element)) = self.current() {
                    let element = element.clone();
                    self.advance();
                    Ok(SelectionAST::Element(element))
                } else {
                    Err(AxiomError::SelectionError("Expected element symbol after 'element'".to_string()))
                }
            }
            Some(SelectionToken::Resname) => {
                self.advance();
                if let Some(SelectionToken::String(resname)) = self.current() {
                    let resname = resname.clone();
                    self.advance();
                    Ok(SelectionAST::Resname(resname))
                } else {
                    Err(AxiomError::SelectionError("Expected residue name after 'resname'".to_string()))
                }
            }
            Some(SelectionToken::Chain) => {
                self.advance();
                if let Some(SelectionToken::String(chain)) = self.current() {
                    let chain = chain.clone();
                    self.advance();
                    Ok(SelectionAST::Chain(chain))
                } else {
                    Err(AxiomError::SelectionError("Expected chain ID after 'chain'".to_string()))
                }
            }
            Some(SelectionToken::Resid) => {
                self.advance();
                match self.current() {
                    Some(SelectionToken::Number(n)) => {
                        let resid = *n as u32;
                        self.advance();
                        Ok(SelectionAST::Resid(resid))
                    }
                    Some(SelectionToken::Range(start, end)) => {
                        let start = *start;
                        let end = *end;
                        self.advance();
                        Ok(SelectionAST::ResidRange(start, end))
                    }
                    _ => Err(AxiomError::SelectionError("Expected residue number or range after 'resid'".to_string())),
                }
            }
            Some(SelectionToken::Within) => {
                self.advance();
                if let Some(SelectionToken::Number(distance)) = self.current() {
                    let distance = *distance;
                    self.advance();
                    self.expect(SelectionToken::Of)?;
                    let selection = self.parse()?;
                    Ok(SelectionAST::Within(distance, Box::new(selection)))
                } else {
                    Err(AxiomError::SelectionError("Expected distance after 'within'".to_string()))
                }
            }
            _ => Err(AxiomError::SelectionError(format!("Unexpected token: {:?}", self.current()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("element O").unwrap();
        assert_eq!(tokens, vec![SelectionToken::Element, SelectionToken::String("O".to_string())]);
    }

    #[test]
    fn test_tokenize_complex() {
        let tokens = tokenize("element O and resname WAT").unwrap();
        assert_eq!(
            tokens,
            vec![
                SelectionToken::Element,
                SelectionToken::String("O".to_string()),
                SelectionToken::And,
                SelectionToken::Resname,
                SelectionToken::String("WAT".to_string())
            ]
        );
    }

    #[test]
    fn test_parse_all() {
        let ast = parse_selection("all").unwrap();
        matches!(ast, SelectionAST::All);
    }

    #[test]
    fn test_parse_element() {
        let ast = parse_selection("element O").unwrap();
        if let SelectionAST::Element(e) = ast {
            assert_eq!(e, "O");
        } else {
            panic!("Expected Element AST node");
        }
    }

    #[test]
    fn test_parse_and() {
        let ast = parse_selection("element O and resname WAT").unwrap();
        matches!(ast, SelectionAST::And(_, _));
    }

    #[test]
    fn test_parse_within() {
        let ast = parse_selection("within 5 of resname LIG").unwrap();
        if let SelectionAST::Within(dist, _) = ast {
            assert_eq!(dist, 5.0);
        } else {
            panic!("Expected Within AST node");
        }
    }
}
