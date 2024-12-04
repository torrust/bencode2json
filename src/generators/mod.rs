pub mod json;
pub mod stack;

// todo: extract trait for generators when we implement a new one.

use derive_more::derive::Display;

#[derive(Debug, PartialEq, Display)]
pub enum BencodeType {
    Integer,
    String,
    List,
    Dict,
}
